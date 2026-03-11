use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use fa_domain::{
    AgenticPattern, ApprovalPolicy, ApprovalRecord, ExecutionPlan, PlanOwner, PlannedStep,
    PlannedTaskBundle, TaskPriority, TaskRecord, TaskRequest, TaskRisk,
};

use crate::audit::{AuditActor, AuditEvent, AuditEventKind, AuditSink, InMemoryAuditSink};
use crate::blueprint::{bootstrap_blueprint, PlatformBlueprint};
use crate::connectors::{
    ConnectorReadRequest, ConnectorReadResult, ConnectorRecordKind, ConnectorRegistry,
    ConnectorSubject,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskIntakeResult {
    pub correlation_id: String,
    pub planned_task: PlannedTaskBundle,
    pub context_reads: Vec<ConnectorReadResult>,
}

#[derive(Clone)]
pub struct WorkOrchestrator {
    blueprint: PlatformBlueprint,
    connectors: ConnectorRegistry,
    audit_sink: Arc<InMemoryAuditSink>,
}

impl Default for WorkOrchestrator {
    fn default() -> Self {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        Self {
            blueprint: bootstrap_blueprint(),
            connectors: ConnectorRegistry::with_m1_defaults(),
            audit_sink,
        }
    }
}

impl WorkOrchestrator {
    pub fn new(blueprint: PlatformBlueprint) -> Self {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        Self {
            blueprint,
            connectors: ConnectorRegistry::with_m1_defaults(),
            audit_sink,
        }
    }

    pub fn with_m1_defaults(audit_sink: Arc<InMemoryAuditSink>) -> Self {
        Self {
            blueprint: bootstrap_blueprint(),
            connectors: ConnectorRegistry::with_m1_defaults(),
            audit_sink,
        }
    }

    pub fn blueprint(&self) -> &PlatformBlueprint {
        &self.blueprint
    }

    pub fn audit_sink(&self) -> &Arc<InMemoryAuditSink> {
        &self.audit_sink
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

    pub fn intake_task(&self, request: TaskRequest) -> Result<TaskIntakeResult> {
        self.intake_task_with_correlation(request, None)
    }

    pub fn intake_task_with_correlation(
        &self,
        request: TaskRequest,
        correlation_id: Option<String>,
    ) -> Result<TaskIntakeResult> {
        let correlation_id = correlation_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let context_reads = self.hydrate_context(&request, &correlation_id)?;
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

        Ok(TaskIntakeResult {
            correlation_id,
            planned_task: PlannedTaskBundle { task, approval },
            context_reads,
        })
    }

    fn hydrate_context(
        &self,
        request: &TaskRequest,
        correlation_id: &str,
    ) -> Result<Vec<ConnectorReadResult>> {
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
            let result = connector.read(&read_request)?;
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
    ) -> Result<()> {
        self.audit_sink.record(AuditEvent {
            id: Uuid::new_v4(),
            correlation_id,
            occurred_at: Utc::now(),
            kind,
            task_id,
            approval_id,
            actor,
            summary,
        })
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
    use crate::InMemoryAuditSink;

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
        assert!(audit_sink.snapshot().expect("snapshot should work").len() >= 4);
    }
}
