use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use fa_domain::{
    ActorHandle, AgenticPattern, ApprovalPolicy, ApprovalRecord, ExecutionPlan, LifecycleError,
    PlanOwner, PlannedStep, PlannedTaskBundle, TaskPriority, TaskRecord, TaskRequest, TaskRisk,
};

use crate::audit::{AuditActor, AuditEvent, AuditEventKind, AuditStore, InMemoryAuditSink};
use crate::blueprint::{bootstrap_blueprint, PlatformBlueprint};
use crate::connectors::{
    ConnectorReadRequest, ConnectorReadResult, ConnectorRecordKind, ConnectorRegistry,
    ConnectorSubject,
};
use crate::evidence::{evidence_from_context_reads, TaskEvidence};
use crate::repository::{InMemoryTaskRepository, TaskRepository};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackedTaskState {
    pub correlation_id: String,
    pub planned_task: PlannedTaskBundle,
    #[serde(default)]
    pub context_reads: Vec<ConnectorReadResult>,
    #[serde(default)]
    pub evidence: Vec<TaskEvidence>,
}

pub type TaskIntakeResult = TrackedTaskState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalActionRequest {
    pub decided_by: ActorHandle,
    pub approved: bool,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResubmitTaskRequest {
    pub requested_by: ActorHandle,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecuteTaskRequest {
    pub actor: ActorHandle,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompleteTaskRequest {
    pub actor: ActorHandle,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailTaskRequest {
    pub actor: ActorHandle,
    pub reason: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OrchestrationError {
    #[error(transparent)]
    Lifecycle(#[from] LifecycleError),
    #[error("task {0} already exists")]
    TaskAlreadyExists(Uuid),
    #[error("task {0} not found")]
    TaskNotFound(Uuid),
    #[error("approval record not found for task {0}")]
    ApprovalNotFound(Uuid),
    #[error("task repository error: {0}")]
    TaskRepository(String),
    #[error("connector error: {0}")]
    Connector(String),
    #[error("audit sink error: {0}")]
    Audit(String),
}

#[derive(Clone)]
pub struct WorkOrchestrator {
    blueprint: PlatformBlueprint,
    connectors: ConnectorRegistry,
    audit_sink: Arc<dyn AuditStore>,
    task_repository: Arc<dyn TaskRepository>,
}

impl Default for WorkOrchestrator {
    fn default() -> Self {
        Self::with_dependencies(
            bootstrap_blueprint(),
            ConnectorRegistry::with_m1_defaults(),
            Arc::new(InMemoryAuditSink::default()),
            Arc::new(InMemoryTaskRepository::default()),
        )
    }
}

impl WorkOrchestrator {
    pub fn new(blueprint: PlatformBlueprint) -> Self {
        Self::with_dependencies(
            blueprint,
            ConnectorRegistry::with_m1_defaults(),
            Arc::new(InMemoryAuditSink::default()),
            Arc::new(InMemoryTaskRepository::default()),
        )
    }

    pub fn with_m1_defaults(audit_sink: Arc<InMemoryAuditSink>) -> Self {
        Self::with_m1_defaults_and_repository(
            audit_sink,
            Arc::new(InMemoryTaskRepository::default()),
        )
    }

    pub fn with_m1_defaults_and_repository(
        audit_sink: Arc<dyn AuditStore>,
        task_repository: Arc<dyn TaskRepository>,
    ) -> Self {
        Self::with_dependencies(
            bootstrap_blueprint(),
            ConnectorRegistry::with_m1_defaults(),
            audit_sink,
            task_repository,
        )
    }

    pub fn with_dependencies(
        blueprint: PlatformBlueprint,
        connectors: ConnectorRegistry,
        audit_sink: Arc<dyn AuditStore>,
        task_repository: Arc<dyn TaskRepository>,
    ) -> Self {
        Self {
            blueprint,
            connectors,
            audit_sink,
            task_repository,
        }
    }

    pub fn blueprint(&self) -> &PlatformBlueprint {
        &self.blueprint
    }

    pub fn audit_sink(&self) -> &Arc<dyn AuditStore> {
        &self.audit_sink
    }

    pub fn get_task(
        &self,
        task_id: Uuid,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError> {
        self.task_repository
            .get(task_id)?
            .ok_or(OrchestrationError::TaskNotFound(task_id))
    }

    pub fn get_task_evidence(
        &self,
        task_id: Uuid,
    ) -> std::result::Result<Vec<TaskEvidence>, OrchestrationError> {
        Ok(self.get_task(task_id)?.evidence)
    }

    pub fn plan_task(&self, request: TaskRequest) -> ExecutionPlan {
        let patterns = select_patterns(&request);
        let approval_policy = select_approval_policy(&request);
        let steps = build_steps(&request, &patterns, approval_policy);
        let rationale = build_rationale(&request, &patterns, approval_policy);

        ExecutionPlan {
            request_id: request.id,
            patterns,
            rationale,
            approval_policy,
            steps,
            created_at: Utc::now(),
        }
    }

    pub fn intake_task(
        &self,
        request: TaskRequest,
    ) -> std::result::Result<TaskIntakeResult, OrchestrationError> {
        self.intake_task_with_correlation(request, None)
    }

    pub fn intake_task_with_correlation(
        &self,
        request: TaskRequest,
        correlation_id: Option<String>,
    ) -> std::result::Result<TaskIntakeResult, OrchestrationError> {
        let correlation_id = correlation_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let context_reads = self.hydrate_context(&request, &correlation_id)?;
        let evidence = evidence_from_context_reads(&context_reads);
        let plan = self.plan_task(request.clone());
        let approval_policy = plan.approval_policy;
        let mut task = TaskRecord::draft(request.clone());
        self.record_event(
            Some(correlation_id.clone()),
            AuditEventKind::TaskCreated,
            Some(task.id),
            None,
            AuditActor::Human(request.initiator.clone()),
            format!("Task '{}' accepted for intake", request.title),
        )?;
        task.apply_plan(plan.clone())?;
        self.record_event(
            Some(correlation_id.clone()),
            AuditEventKind::TaskPlanned,
            Some(task.id),
            None,
            AuditActor::System("workflow-engine".to_string()),
            format!(
                "Task '{}' planned with {:?} approval policy",
                request.title, approval_policy
            ),
        )?;

        let approval = if approval_policy.requires_human_approval() {
            let approval = ApprovalRecord::pending(task.id, approval_policy, request.initiator)?;
            task.request_approval(approval.id)?;
            self.record_event(
                Some(correlation_id.clone()),
                AuditEventKind::ApprovalRequested,
                Some(task.id),
                Some(approval.id),
                AuditActor::System("workflow-engine".to_string()),
                format!(
                    "Approval requested from role '{}' for task '{}'",
                    approval.required_role, task.request.title
                ),
            )?;
            self.record_event(
                Some(correlation_id.clone()),
                AuditEventKind::TaskStatusChanged,
                Some(task.id),
                Some(approval.id),
                AuditActor::System("workflow-engine".to_string()),
                format!("Task transitioned to {:?}", task.status),
            )?;
            Some(approval)
        } else {
            task.auto_approve()?;
            self.record_event(
                Some(correlation_id.clone()),
                AuditEventKind::TaskStatusChanged,
                Some(task.id),
                None,
                AuditActor::System("workflow-engine".to_string()),
                format!("Task auto-approved and transitioned to {:?}", task.status),
            )?;
            None
        };

        let tracked_state = TrackedTaskState {
            correlation_id,
            planned_task: PlannedTaskBundle { task, approval },
            context_reads,
            evidence,
        };
        self.task_repository.create(tracked_state.clone())?;

        Ok(tracked_state)
    }

    pub fn approve_task(
        &self,
        task_id: Uuid,
        action: ApprovalActionRequest,
        correlation_id: Option<String>,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError> {
        let decided_by = action.decided_by.clone();
        let approved = action.approved;
        let comment = action.comment.clone();
        let updated = self.update_task(task_id, correlation_id, |state| {
            let approval = state
                .planned_task
                .approval
                .as_mut()
                .ok_or(OrchestrationError::ApprovalNotFound(task_id))?;

            if approved {
                approval.approve(decided_by.clone(), comment.clone())?;
                state.planned_task.task.approve()?;
            } else {
                approval.reject(decided_by.clone(), comment.clone())?;
                state.planned_task.task.return_for_revision()?;
            }
            Ok(())
        })?;
        let correlation_id = updated.correlation_id.clone();
        let approval = updated
            .planned_task
            .approval
            .as_ref()
            .ok_or(OrchestrationError::ApprovalNotFound(task_id))?;
        let decision_event_kind = if approved {
            AuditEventKind::ApprovalApproved
        } else {
            AuditEventKind::ApprovalRejected
        };

        self.record_event(
            Some(correlation_id.clone()),
            decision_event_kind,
            Some(task_id),
            Some(approval.id),
            AuditActor::Human(decided_by),
            format!(
                "Approval decision for task '{}' is {:?}",
                updated.planned_task.task.request.title, approval.status
            ),
        )?;
        self.record_event(
            Some(correlation_id),
            AuditEventKind::TaskStatusChanged,
            Some(task_id),
            Some(approval.id),
            AuditActor::System("workflow-engine".to_string()),
            format!(
                "Task transitioned to {:?} after approval decision",
                updated.planned_task.task.status
            ),
        )?;

        Ok(updated)
    }

    pub fn resubmit_task(
        &self,
        task_id: Uuid,
        action: ResubmitTaskRequest,
        correlation_id: Option<String>,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError> {
        let requested_by = action.requested_by.clone();
        let comment = action.comment.clone();
        let updated = self.update_task(task_id, correlation_id, |state| {
            let plan = state
                .planned_task
                .task
                .plan
                .as_ref()
                .ok_or(LifecycleError::MissingExecutionPlan)?;
            let approval =
                ApprovalRecord::pending(task_id, plan.approval_policy, requested_by.clone())?;
            state.planned_task.task.request_approval(approval.id)?;
            state.planned_task.approval = Some(approval);
            Ok(())
        })?;
        let approval = updated
            .planned_task
            .approval
            .as_ref()
            .ok_or(OrchestrationError::ApprovalNotFound(task_id))?;

        self.record_event(
            Some(updated.correlation_id.clone()),
            AuditEventKind::ApprovalRequested,
            Some(task_id),
            Some(approval.id),
            AuditActor::Human(requested_by),
            comment.unwrap_or_else(|| {
                format!(
                    "Task '{}' resubmitted for approval to role '{}'",
                    updated.planned_task.task.request.title, approval.required_role
                )
            }),
        )?;
        self.record_event(
            Some(updated.correlation_id.clone()),
            AuditEventKind::TaskStatusChanged,
            Some(task_id),
            Some(approval.id),
            AuditActor::System("workflow-engine".to_string()),
            format!(
                "Task transitioned to {:?} after resubmission",
                updated.planned_task.task.status
            ),
        )?;

        Ok(updated)
    }

    pub fn start_execution(
        &self,
        task_id: Uuid,
        action: ExecuteTaskRequest,
        correlation_id: Option<String>,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError> {
        let actor = action.actor;
        let note = action.note;
        let updated = self.update_task(task_id, correlation_id, |state| {
            state.planned_task.task.start_execution()?;
            Ok(())
        })?;

        self.record_event(
            Some(updated.correlation_id.clone()),
            AuditEventKind::TaskStatusChanged,
            Some(task_id),
            updated
                .planned_task
                .approval
                .as_ref()
                .map(|approval| approval.id),
            AuditActor::Human(actor),
            note.unwrap_or_else(|| {
                format!(
                    "Task '{}' transitioned to {:?}",
                    updated.planned_task.task.request.title, updated.planned_task.task.status
                )
            }),
        )?;

        Ok(updated)
    }

    pub fn complete_task(
        &self,
        task_id: Uuid,
        action: CompleteTaskRequest,
        correlation_id: Option<String>,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError> {
        let actor = action.actor;
        let note = action.note;
        let updated = self.update_task(task_id, correlation_id, |state| {
            state.planned_task.task.complete()?;
            Ok(())
        })?;

        self.record_event(
            Some(updated.correlation_id.clone()),
            AuditEventKind::TaskStatusChanged,
            Some(task_id),
            updated
                .planned_task
                .approval
                .as_ref()
                .map(|approval| approval.id),
            AuditActor::Human(actor),
            note.unwrap_or_else(|| {
                format!(
                    "Task '{}' transitioned to {:?}",
                    updated.planned_task.task.request.title, updated.planned_task.task.status
                )
            }),
        )?;

        Ok(updated)
    }

    pub fn fail_task(
        &self,
        task_id: Uuid,
        action: FailTaskRequest,
        correlation_id: Option<String>,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError> {
        let actor = action.actor;
        let reason = action.reason;
        let updated = self.update_task(task_id, correlation_id, |state| {
            state.planned_task.task.fail(reason.clone())?;
            Ok(())
        })?;

        self.record_event(
            Some(updated.correlation_id.clone()),
            AuditEventKind::TaskStatusChanged,
            Some(task_id),
            updated
                .planned_task
                .approval
                .as_ref()
                .map(|approval| approval.id),
            AuditActor::Human(actor),
            format!(
                "Task '{}' failed: {}",
                updated.planned_task.task.request.title, reason
            ),
        )?;

        Ok(updated)
    }

    fn update_task<F>(
        &self,
        task_id: Uuid,
        correlation_id: Option<String>,
        mutate: F,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError>
    where
        F: FnOnce(&mut TrackedTaskState) -> std::result::Result<(), OrchestrationError>,
    {
        let mut state = self.get_task(task_id)?;
        mutate(&mut state)?;
        state.correlation_id = correlation_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        self.task_repository.save(state)
    }

    fn hydrate_context(
        &self,
        request: &TaskRequest,
        correlation_id: &str,
    ) -> std::result::Result<Vec<ConnectorReadResult>, OrchestrationError> {
        let mut results = Vec::new();
        let subject = primary_subject(request);

        for target in &request.integrations {
            let Some(kind) = ConnectorRegistry::kind_for_target(target) else {
                continue;
            };
            let Some(connector) = self.connectors.connector_for_kind(&kind) else {
                continue;
            };
            let read_request = ConnectorReadRequest {
                correlation_id: Some(correlation_id.to_string()),
                task_id: Some(request.id),
                subject: subject.clone(),
                requested_records: requested_record_kinds(target),
            };
            let result = connector
                .read(&read_request)
                .map_err(|error| OrchestrationError::Connector(error.to_string()))?;
            self.record_event(
                Some(correlation_id.to_string()),
                AuditEventKind::ConnectorRead,
                Some(request.id),
                None,
                AuditActor::System(format!("{:?}", result.connector)),
                format!(
                    "Read {} context records from {:?}",
                    result.records.len(),
                    result.connector
                ),
            )?;
            results.push(result);
        }

        Ok(results)
    }

    fn record_event(
        &self,
        correlation_id: Option<String>,
        kind: AuditEventKind,
        task_id: Option<Uuid>,
        approval_id: Option<Uuid>,
        actor: AuditActor,
        summary: String,
    ) -> std::result::Result<(), OrchestrationError> {
        self.audit_sink
            .record(AuditEvent {
                id: Uuid::new_v4(),
                correlation_id,
                occurred_at: Utc::now(),
                kind,
                task_id,
                approval_id,
                actor,
                summary,
            })
            .map_err(|error| OrchestrationError::Audit(error.to_string()))
    }
}

fn primary_subject(request: &TaskRequest) -> ConnectorSubject {
    request
        .equipment_ids
        .first()
        .cloned()
        .map(ConnectorSubject::Equipment)
        .unwrap_or(ConnectorSubject::Task(request.id))
}

fn requested_record_kinds(target: &fa_domain::IntegrationTarget) -> Vec<ConnectorRecordKind> {
    match target {
        fa_domain::IntegrationTarget::Mes => vec![
            ConnectorRecordKind::TaskContext,
            ConnectorRecordKind::EquipmentTelemetry,
        ],
        fa_domain::IntegrationTarget::Cmms => vec![
            ConnectorRecordKind::MaintenanceHistory,
            ConnectorRecordKind::WorkOrderContext,
        ],
        _ => Vec::new(),
    }
}

fn select_patterns(request: &TaskRequest) -> Vec<AgenticPattern> {
    let mut patterns = Vec::new();

    if matches!(request.risk, TaskRisk::Low)
        && request.integrations.len() <= 1
        && !request.requires_human_approval
        && !request.requires_diagnostic_loop
    {
        patterns.push(AgenticPattern::SingleAgent);
    } else {
        patterns.push(AgenticPattern::Coordinator);
    }

    patterns.push(AgenticPattern::DeterministicWorkflow);

    if request.requires_diagnostic_loop
        || matches!(request.priority, TaskPriority::Critical)
        || request.description.to_ascii_lowercase().contains("diagnos")
    {
        patterns.push(AgenticPattern::ReActLoop);
    }

    if request.requires_human_approval
        || matches!(request.risk, TaskRisk::High | TaskRisk::Critical)
    {
        patterns.push(AgenticPattern::HumanInTheLoop);
    }

    if request.integrations.len() > 1 || !request.equipment_ids.is_empty() {
        patterns.push(AgenticPattern::CustomBusinessLogic);
    }

    patterns
}

fn select_approval_policy(request: &TaskRequest) -> ApprovalPolicy {
    if matches!(request.risk, TaskRisk::Critical) {
        ApprovalPolicy::PlantManager
    } else if request.requires_human_approval || matches!(request.risk, TaskRisk::High) {
        ApprovalPolicy::SafetyOfficer
    } else if matches!(request.priority, TaskPriority::Critical) {
        ApprovalPolicy::OperationsSupervisor
    } else {
        ApprovalPolicy::Auto
    }
}

fn build_steps(
    request: &TaskRequest,
    patterns: &[AgenticPattern],
    approval_policy: ApprovalPolicy,
) -> Vec<PlannedStep> {
    let mut steps = vec![PlannedStep {
        sequence: 1,
        label: "Intake and context hydration".to_string(),
        owner: PlanOwner::System("workflow-engine".to_string()),
        expected_output: format!(
            "Validated request, enterprise context, and connector read plan for '{}'",
            request.title
        ),
    }];

    if patterns.contains(&AgenticPattern::SingleAgent) {
        steps.push(PlannedStep {
            sequence: 2,
            label: "Single-agent planning".to_string(),
            owner: PlanOwner::Agent("ops-copilot".to_string()),
            expected_output: "A concise action plan with evidence and confidence".to_string(),
        });
    }

    if patterns.contains(&AgenticPattern::Coordinator) {
        steps.push(PlannedStep {
            sequence: 2,
            label: "Coordinator task graph".to_string(),
            owner: PlanOwner::Agent("orchestrator".to_string()),
            expected_output: "Delegation map covering people, systems, and domain agents"
                .to_string(),
        });
    }

    if patterns.contains(&AgenticPattern::ReActLoop) {
        steps.push(PlannedStep {
            sequence: 3,
            label: "Evidence-driven reasoning loop".to_string(),
            owner: PlanOwner::Agent("reasoning-loop".to_string()),
            expected_output: "Hypotheses, evidence citations, and next-best-action recommendations"
                .to_string(),
        });
    }

    if patterns.contains(&AgenticPattern::CustomBusinessLogic) {
        steps.push(PlannedStep {
            sequence: 4,
            label: "Policy and connector orchestration".to_string(),
            owner: PlanOwner::System("policy-engine".to_string()),
            expected_output: "A deterministic connector sequence bounded by SOP and safety policy"
                .to_string(),
        });
    }

    if patterns.contains(&AgenticPattern::HumanInTheLoop) {
        steps.push(PlannedStep {
            sequence: 5,
            label: "Human approval checkpoint".to_string(),
            owner: PlanOwner::Human(format!("{approval_policy:?}")),
            expected_output: "Approved, rejected, or revised execution package".to_string(),
        });
    }

    steps.push(PlannedStep {
        sequence: 6,
        label: "Execution, audit, and close-out".to_string(),
        owner: PlanOwner::System("execution-runner".to_string()),
        expected_output: "Task outcome, audit trail, and follow-up actions".to_string(),
    });

    steps
}

fn build_rationale(
    request: &TaskRequest,
    patterns: &[AgenticPattern],
    approval_policy: ApprovalPolicy,
) -> Vec<String> {
    let mut rationale = vec![format!(
        "Task '{}' is classified as {:?} risk and {:?} priority.",
        request.title, request.risk, request.priority
    )];

    if patterns.contains(&AgenticPattern::Coordinator) {
        rationale.push(
            "Coordinator orchestration was selected because the task spans multiple actors, systems, or equipment."
                .to_string(),
        );
    }

    if patterns.contains(&AgenticPattern::SingleAgent) {
        rationale.push(
            "Single-agent handling is sufficient because the task is low-risk and operationally narrow."
                .to_string(),
        );
    }

    if patterns.contains(&AgenticPattern::ReActLoop) {
        rationale.push(
            "A ReAct-style loop is required to iterate over evidence before recommending action."
                .to_string(),
        );
    }

    if patterns.contains(&AgenticPattern::CustomBusinessLogic) {
        rationale.push(
            "Custom business logic is required to enforce deterministic SOP, connector sequencing, and safety boundaries."
                .to_string(),
        );
    }

    rationale.push(format!(
        "Approval policy is {:?}, aligning with governance for manufacturing change and safety.",
        approval_policy
    ));

    rationale
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use fa_domain::{ActorHandle, IntegrationTarget};

    use super::*;
    use crate::{InMemoryAuditSink, InMemoryTaskRepository, ResubmitTaskRequest, TaskRepository};

    fn approver() -> ActorHandle {
        ActorHandle {
            id: "worker_2001".to_string(),
            display_name: "Chen QE".to_string(),
            role: "Quality Engineer".to_string(),
        }
    }

    fn executor() -> ActorHandle {
        ActorHandle {
            id: "worker_3001".to_string(),
            display_name: "Wu Maint".to_string(),
            role: "Maintenance Technician".to_string(),
        }
    }

    fn base_request() -> TaskRequest {
        TaskRequest {
            id: Uuid::new_v4(),
            title: "Investigate spindle temperature drift".to_string(),
            description: "Diagnose repeated spindle temperature drift before the next shift."
                .to_string(),
            priority: TaskPriority::Expedited,
            risk: TaskRisk::Medium,
            initiator: ActorHandle {
                id: "worker_1001".to_string(),
                display_name: "Liu Supervisor".to_string(),
                role: "Production Supervisor".to_string(),
            },
            stakeholders: Vec::new(),
            equipment_ids: vec!["eq_cnc_01".to_string()],
            integrations: vec![IntegrationTarget::Mes, IntegrationTarget::Cmms],
            desired_outcome: "Recover stable spindle temperature within tolerance".to_string(),
            requires_human_approval: false,
            requires_diagnostic_loop: true,
        }
    }

    #[test]
    fn simple_low_risk_work_uses_single_agent() {
        let orchestrator = WorkOrchestrator::default();
        let mut request = base_request();
        request.title = "Summarize shift notes".to_string();
        request.description = "Summarize shift notes for morning handoff.".to_string();
        request.priority = TaskPriority::Routine;
        request.risk = TaskRisk::Low;
        request.integrations = vec![IntegrationTarget::Mes];
        request.equipment_ids.clear();
        request.requires_diagnostic_loop = false;

        let plan = orchestrator.plan_task(request);

        assert!(plan.patterns.contains(&AgenticPattern::SingleAgent));
        assert!(!plan.patterns.contains(&AgenticPattern::HumanInTheLoop));
        assert_eq!(plan.approval_policy, ApprovalPolicy::Auto);
    }

    #[test]
    fn critical_work_requires_human_governance() {
        let orchestrator = WorkOrchestrator::default();
        let mut request = base_request();
        request.priority = TaskPriority::Critical;
        request.risk = TaskRisk::Critical;
        request.requires_human_approval = true;

        let plan = orchestrator.plan_task(request);

        assert!(plan.patterns.contains(&AgenticPattern::Coordinator));
        assert!(plan.patterns.contains(&AgenticPattern::ReActLoop));
        assert!(plan.patterns.contains(&AgenticPattern::HumanInTheLoop));
        assert!(plan.patterns.contains(&AgenticPattern::CustomBusinessLogic));
        assert_eq!(plan.approval_policy, ApprovalPolicy::PlantManager);
    }

    #[test]
    fn intake_task_auto_approves_low_risk_work() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let mut request = base_request();
        request.title = "Summarize shift notes".to_string();
        request.description = "Summarize shift notes for morning handoff.".to_string();
        request.priority = TaskPriority::Routine;
        request.risk = TaskRisk::Low;
        request.integrations = vec![IntegrationTarget::Mes];
        request.equipment_ids.clear();
        request.requires_diagnostic_loop = false;

        let intake_result = orchestrator
            .intake_task(request)
            .expect("intake should succeed");

        assert_eq!(
            intake_result.planned_task.task.status,
            fa_domain::TaskStatus::Approved
        );
        assert!(intake_result.planned_task.approval.is_none());
        assert_eq!(intake_result.context_reads.len(), 1);
        assert_eq!(intake_result.evidence.len(), 2);
        assert!(!audit_sink
            .snapshot()
            .expect("snapshot should work")
            .is_empty());
    }

    #[test]
    fn intake_task_creates_manual_approval_for_high_risk_work() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let mut request = base_request();
        request.priority = TaskPriority::Critical;
        request.risk = TaskRisk::High;
        request.requires_human_approval = true;

        let intake_result = orchestrator
            .intake_task(request)
            .expect("intake should succeed");

        assert_eq!(
            intake_result.planned_task.task.status,
            fa_domain::TaskStatus::AwaitingApproval
        );
        assert_eq!(
            intake_result
                .planned_task
                .approval
                .as_ref()
                .map(|approval| approval.policy),
            Some(ApprovalPolicy::SafetyOfficer)
        );
        assert_eq!(intake_result.context_reads.len(), 2);
        assert_eq!(intake_result.evidence.len(), 4);
        assert!(audit_sink.snapshot().expect("snapshot should work").len() >= 4);
    }

    #[test]
    fn get_task_evidence_returns_structured_snapshots() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink);
        let request = base_request();
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");

        let evidence = orchestrator
            .get_task_evidence(task_id)
            .expect("evidence lookup should succeed");

        assert_eq!(evidence.len(), 4);
        assert!(evidence
            .iter()
            .any(|item| item.summary.contains("telemetry")));
        assert!(evidence
            .iter()
            .any(|item| item.summary.contains("recommended_action")
                || item.summary.contains("recommends")));
    }

    #[test]
    fn get_task_returns_stored_state_after_intake() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink);
        let request = base_request();
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should persist task");
        let stored = orchestrator.get_task(task_id).expect("task should exist");

        assert_eq!(stored.planned_task.task.id, task_id);
        assert_eq!(
            stored.planned_task.task.status,
            fa_domain::TaskStatus::Approved
        );
    }

    #[test]
    fn approve_task_transitions_to_approved() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let mut request = base_request();
        request.risk = TaskRisk::High;
        request.requires_human_approval = true;
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");
        let updated = orchestrator
            .approve_task(
                task_id,
                ApprovalActionRequest {
                    decided_by: approver(),
                    approved: true,
                    comment: Some("Proceed to execution".to_string()),
                },
                Some("approve-001".to_string()),
            )
            .expect("approval should succeed");

        assert_eq!(
            updated.planned_task.task.status,
            fa_domain::TaskStatus::Approved
        );
        assert_eq!(
            updated
                .planned_task
                .approval
                .as_ref()
                .map(|approval| approval.status),
            Some(fa_domain::ApprovalStatus::Approved)
        );
        assert_eq!(updated.correlation_id, "approve-001");
        assert!(audit_sink.snapshot().expect("audit snapshot").len() >= 6);
    }

    #[test]
    fn execute_task_transitions_approved_work_to_executing() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let mut request = base_request();
        request.risk = TaskRisk::High;
        request.requires_human_approval = true;
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");
        orchestrator
            .approve_task(
                task_id,
                ApprovalActionRequest {
                    decided_by: approver(),
                    approved: true,
                    comment: Some("Proceed to execution".to_string()),
                },
                Some("approve-002".to_string()),
            )
            .expect("approval should succeed");

        let executing = orchestrator
            .start_execution(
                task_id,
                ExecuteTaskRequest {
                    actor: executor(),
                    note: Some("Execution stub started".to_string()),
                },
                Some("execute-001".to_string()),
            )
            .expect("execution should start");

        assert_eq!(
            executing.planned_task.task.status,
            fa_domain::TaskStatus::Executing
        );
        assert_eq!(executing.correlation_id, "execute-001");
        assert!(audit_sink.snapshot().expect("audit snapshot").len() >= 7);
    }

    #[test]
    fn complete_task_transitions_executing_work_to_completed() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let mut request = base_request();
        request.risk = TaskRisk::High;
        request.requires_human_approval = true;
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");
        orchestrator
            .approve_task(
                task_id,
                ApprovalActionRequest {
                    decided_by: approver(),
                    approved: true,
                    comment: Some("Proceed to execution".to_string()),
                },
                Some("approve-003".to_string()),
            )
            .expect("approval should succeed");
        orchestrator
            .start_execution(
                task_id,
                ExecuteTaskRequest {
                    actor: executor(),
                    note: Some("Execution stub started".to_string()),
                },
                Some("execute-002".to_string()),
            )
            .expect("execution should start");

        let completed = orchestrator
            .complete_task(
                task_id,
                CompleteTaskRequest {
                    actor: executor(),
                    note: Some("Execution finished".to_string()),
                },
                Some("complete-001".to_string()),
            )
            .expect("completion should succeed");

        assert_eq!(
            completed.planned_task.task.status,
            fa_domain::TaskStatus::Completed
        );
        assert_eq!(completed.correlation_id, "complete-001");
    }

    #[test]
    fn fail_task_transitions_work_to_failed() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let mut request = base_request();
        request.risk = TaskRisk::High;
        request.requires_human_approval = true;
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");
        orchestrator
            .approve_task(
                task_id,
                ApprovalActionRequest {
                    decided_by: approver(),
                    approved: true,
                    comment: Some("Proceed to execution".to_string()),
                },
                Some("approve-004".to_string()),
            )
            .expect("approval should succeed");
        orchestrator
            .start_execution(
                task_id,
                ExecuteTaskRequest {
                    actor: executor(),
                    note: Some("Execution stub started".to_string()),
                },
                Some("execute-003".to_string()),
            )
            .expect("execution should start");

        let failed = orchestrator
            .fail_task(
                task_id,
                FailTaskRequest {
                    actor: executor(),
                    reason: "Cooling loop inspection failed".to_string(),
                },
                Some("fail-001".to_string()),
            )
            .expect("failure should be recorded");

        assert_eq!(
            failed.planned_task.task.status,
            fa_domain::TaskStatus::Failed
        );
        assert_eq!(
            failed.planned_task.task.latest_error.as_deref(),
            Some("Cooling loop inspection failed")
        );
        assert_eq!(failed.correlation_id, "fail-001");
    }

    #[test]
    fn resubmit_task_reopens_rejected_work_for_approval() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let mut request = base_request();
        request.risk = TaskRisk::High;
        request.requires_human_approval = true;
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");
        let rejected = orchestrator
            .approve_task(
                task_id,
                ApprovalActionRequest {
                    decided_by: approver(),
                    approved: false,
                    comment: Some("Need more evidence".to_string()),
                },
                Some("approve-reject-001".to_string()),
            )
            .expect("rejection should succeed");
        assert_eq!(
            rejected.planned_task.task.status,
            fa_domain::TaskStatus::Planned
        );
        assert_eq!(
            rejected
                .planned_task
                .approval
                .as_ref()
                .map(|approval| approval.status),
            Some(fa_domain::ApprovalStatus::Rejected)
        );

        let resubmitted = orchestrator
            .resubmit_task(
                task_id,
                ResubmitTaskRequest {
                    requested_by: ActorHandle {
                        id: "worker_1001".to_string(),
                        display_name: "Liu Supervisor".to_string(),
                        role: "Production Supervisor".to_string(),
                    },
                    comment: Some("Added vibration report and revised action plan".to_string()),
                },
                Some("resubmit-001".to_string()),
            )
            .expect("resubmission should succeed");

        assert_eq!(
            resubmitted.planned_task.task.status,
            fa_domain::TaskStatus::AwaitingApproval
        );
        assert_eq!(
            resubmitted
                .planned_task
                .approval
                .as_ref()
                .map(|approval| approval.status),
            Some(fa_domain::ApprovalStatus::Pending)
        );
        assert_eq!(resubmitted.correlation_id, "resubmit-001");
        assert!(audit_sink.snapshot().expect("audit snapshot").len() >= 9);
    }

    #[test]
    fn injected_repository_exposes_tracked_state_outside_orchestrator() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator =
            WorkOrchestrator::with_m1_defaults_and_repository(audit_sink, repository.clone());
        let request = base_request();
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should persist task");

        let stored = TaskRepository::get(repository.as_ref(), task_id)
            .expect("repository get should succeed")
            .expect("task should exist");

        assert_eq!(stored.planned_task.task.id, task_id);
        assert_eq!(
            stored.planned_task.task.status,
            fa_domain::TaskStatus::Approved
        );
    }
}
