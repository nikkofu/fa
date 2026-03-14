use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use fa_domain::{
    ActorHandle, AgenticPattern, ApprovalPolicy, ApprovalRecord, ApprovalStrategy, ExecutionPlan,
    GovernanceParticipation, LifecycleError, PlanOwner, PlannedStep, PlannedTaskBundle,
    ResponsibilityAssignment, TaskPriority, TaskRecord, TaskRequest, TaskRisk, TaskStatus,
    WorkflowGovernance,
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
    #[serde(default)]
    pub follow_up_items: Vec<FollowUpItemView>,
    #[serde(default)]
    pub follow_up_summary: FollowUpSummary,
    #[serde(default)]
    pub handoff_receipt: Option<HandoffReceiptView>,
    #[serde(default)]
    pub handoff_receipt_summary: HandoffReceiptSummary,
    #[serde(default)]
    pub alert_cluster_drafts: Vec<AlertClusterDraftView>,
    #[serde(default)]
    pub alert_triage_summary: AlertTriageSummary,
}

pub type TaskIntakeResult = TrackedTaskState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FollowUpItemView {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub source_kind: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub status: String,
    pub recommended_owner_role: Option<String>,
    pub accepted_owner_id: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub sla_status: String,
    pub blocked_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FollowUpSummary {
    pub total_items: usize,
    pub open_items: usize,
    pub blocked_items: usize,
    pub overdue_items: usize,
    pub escalated_items: usize,
    pub last_evaluated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FollowUpQueueQuery {
    pub task_id: Option<Uuid>,
    pub source_kind: Option<String>,
    pub status: Option<String>,
    pub owner_id: Option<String>,
    pub owner_role: Option<String>,
    pub overdue_only: bool,
    pub blocked_only: bool,
    pub escalation_required: bool,
    pub due_before: Option<DateTime<Utc>>,
    pub task_risk: Option<TaskRisk>,
    pub task_priority: Option<TaskPriority>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FollowUpQueueItemView {
    pub task_id: Uuid,
    pub correlation_id: String,
    pub task_title: String,
    pub task_priority: TaskPriority,
    pub task_risk: TaskRisk,
    pub task_status: TaskStatus,
    pub follow_up_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub source_kind: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub status: String,
    pub recommended_owner_role: Option<String>,
    pub accepted_owner_id: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub sla_status: String,
    pub effective_sla_status: String,
    pub overdue: bool,
    pub blocked_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FollowUpMonitoringBucket {
    pub key: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FollowUpMonitoringView {
    pub total_items: usize,
    pub open_items: usize,
    pub accepted_items: usize,
    pub unaccepted_items: usize,
    pub blocked_items: usize,
    pub overdue_items: usize,
    pub escalation_required_items: usize,
    pub next_due_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub source_kind_counts: Vec<FollowUpMonitoringBucket>,
    #[serde(default)]
    pub owner_role_counts: Vec<FollowUpMonitoringBucket>,
    #[serde(default)]
    pub sla_status_counts: Vec<FollowUpMonitoringBucket>,
    #[serde(default)]
    pub task_risk_counts: Vec<FollowUpMonitoringBucket>,
    #[serde(default)]
    pub task_priority_counts: Vec<FollowUpMonitoringBucket>,
    pub last_evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HandoffReceiptQueueQuery {
    pub task_id: Option<Uuid>,
    pub shift_id: Option<String>,
    pub receipt_status: Option<String>,
    pub receiving_role: Option<String>,
    pub receiving_actor_id: Option<String>,
    pub overdue_only: bool,
    pub has_exceptions: bool,
    pub escalated_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffReceiptView {
    pub id: String,
    pub handoff_task_id: Uuid,
    pub shift_id: String,
    pub sending_actor: ActorHandle,
    pub receiving_role: String,
    pub receiving_actor: Option<ActorHandle>,
    pub published_at: DateTime<Utc>,
    pub required_ack_by: Option<DateTime<Utc>>,
    pub status: String,
    #[serde(default)]
    pub follow_up_item_ids: Vec<String>,
    pub exception_note: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub escalation_state: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffReceiptSummary {
    pub status: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub required_ack_by: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub covered_follow_up_count: usize,
    pub unaccepted_follow_up_count: usize,
    pub exception_flag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffReceiptQueueItemView {
    pub task_id: Uuid,
    pub correlation_id: String,
    pub task_title: String,
    pub task_priority: TaskPriority,
    pub task_risk: TaskRisk,
    pub task_status: TaskStatus,
    pub receipt_id: String,
    pub shift_id: String,
    pub receipt_status: String,
    pub effective_status: String,
    pub sending_actor: ActorHandle,
    pub receiving_role: String,
    pub receiving_actor: Option<ActorHandle>,
    pub published_at: DateTime<Utc>,
    pub required_ack_by: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub follow_up_item_ids: Vec<String>,
    pub covered_follow_up_count: usize,
    pub unaccepted_follow_up_count: usize,
    pub has_exceptions: bool,
    pub exception_note: Option<String>,
    pub escalation_state: Option<String>,
    pub overdue: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffReceiptMonitoringBucket {
    pub key: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffReceiptMonitoringView {
    pub total_receipts: usize,
    pub open_receipts: usize,
    pub acknowledged_receipts: usize,
    pub unacknowledged_receipts: usize,
    pub overdue_receipts: usize,
    pub exception_receipts: usize,
    pub escalated_receipts: usize,
    pub next_ack_due_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub effective_status_counts: Vec<HandoffReceiptMonitoringBucket>,
    #[serde(default)]
    pub receiving_role_counts: Vec<HandoffReceiptMonitoringBucket>,
    #[serde(default)]
    pub ack_window_counts: Vec<HandoffReceiptMonitoringBucket>,
    #[serde(default)]
    pub task_risk_counts: Vec<HandoffReceiptMonitoringBucket>,
    #[serde(default)]
    pub task_priority_counts: Vec<HandoffReceiptMonitoringBucket>,
    pub last_evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertClusterDraftView {
    pub cluster_id: String,
    pub cluster_status: String,
    pub source_system: Option<String>,
    pub equipment_id: Option<String>,
    pub line_id: Option<String>,
    pub severity_band: String,
    #[serde(default)]
    pub source_event_refs: Vec<String>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub triage_label: Option<String>,
    pub recommended_owner_role: Option<String>,
    pub escalation_candidate: bool,
    pub rationale: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertTriageSummary {
    pub total_clusters: usize,
    pub open_clusters: usize,
    pub high_priority_clusters: usize,
    pub escalation_candidate_count: usize,
    pub last_clustered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AlertClusterQueueQuery {
    pub task_id: Option<Uuid>,
    pub cluster_status: Option<String>,
    pub source_system: Option<String>,
    pub equipment_id: Option<String>,
    pub line_id: Option<String>,
    pub severity_band: Option<String>,
    pub triage_label: Option<String>,
    pub recommended_owner_role: Option<String>,
    pub follow_up_owner_id: Option<String>,
    pub unaccepted_follow_up_only: bool,
    pub follow_up_escalation_required: bool,
    pub escalation_candidate: bool,
    pub window_from: Option<DateTime<Utc>>,
    pub window_to: Option<DateTime<Utc>>,
    pub open_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertClusterQueueItemView {
    pub task_id: Uuid,
    pub correlation_id: String,
    pub task_title: String,
    pub task_priority: TaskPriority,
    pub task_risk: TaskRisk,
    pub task_status: TaskStatus,
    pub cluster_id: String,
    pub cluster_status: String,
    pub source_system: Option<String>,
    pub equipment_id: Option<String>,
    pub line_id: Option<String>,
    pub severity_band: String,
    #[serde(default)]
    pub source_event_refs: Vec<String>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub triage_label: Option<String>,
    pub recommended_owner_role: Option<String>,
    pub escalation_candidate: bool,
    pub linked_follow_up: Option<AlertClusterLinkedFollowUpView>,
    pub rationale: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertClusterLinkedFollowUpView {
    pub total_items: usize,
    pub open_items: usize,
    pub accepted_items: usize,
    pub unaccepted_items: usize,
    #[serde(default)]
    pub follow_up_ids: Vec<String>,
    #[serde(default)]
    pub accepted_owner_ids: Vec<String>,
    pub worst_effective_sla_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertClusterMonitoringBucket {
    pub key: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertClusterMonitoringView {
    pub total_clusters: usize,
    pub open_clusters: usize,
    pub escalation_candidate_clusters: usize,
    pub high_severity_clusters: usize,
    pub active_window_clusters: usize,
    pub stale_window_clusters: usize,
    pub linked_follow_up_clusters: usize,
    pub unlinked_follow_up_clusters: usize,
    pub accepted_follow_up_clusters: usize,
    pub unaccepted_follow_up_clusters: usize,
    pub follow_up_escalation_clusters: usize,
    pub next_window_end_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub cluster_status_counts: Vec<AlertClusterMonitoringBucket>,
    #[serde(default)]
    pub source_system_counts: Vec<AlertClusterMonitoringBucket>,
    #[serde(default)]
    pub severity_band_counts: Vec<AlertClusterMonitoringBucket>,
    #[serde(default)]
    pub triage_label_counts: Vec<AlertClusterMonitoringBucket>,
    #[serde(default)]
    pub owner_role_counts: Vec<AlertClusterMonitoringBucket>,
    #[serde(default)]
    pub window_state_counts: Vec<AlertClusterMonitoringBucket>,
    #[serde(default)]
    pub follow_up_coverage_counts: Vec<AlertClusterMonitoringBucket>,
    #[serde(default)]
    pub follow_up_sla_status_counts: Vec<AlertClusterMonitoringBucket>,
    #[serde(default)]
    pub task_risk_counts: Vec<AlertClusterMonitoringBucket>,
    #[serde(default)]
    pub task_priority_counts: Vec<AlertClusterMonitoringBucket>,
    pub last_evaluated_at: DateTime<Utc>,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptFollowUpOwnerRequest {
    pub actor: ActorHandle,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcknowledgeHandoffReceiptRequest {
    pub actor: ActorHandle,
    pub exception_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscalateHandoffReceiptRequest {
    pub actor: ActorHandle,
    pub note: Option<String>,
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
    #[error("follow-up item '{follow_up_id}' not found for task {task_id}")]
    FollowUpItemNotFound { task_id: Uuid, follow_up_id: String },
    #[error(
        "follow-up item '{follow_up_id}' requires role '{required_role}', got '{actual_role}'"
    )]
    FollowUpRoleMismatch {
        follow_up_id: String,
        required_role: String,
        actual_role: String,
    },
    #[error(
        "follow-up item '{follow_up_id}' for task {task_id} cannot transition from status '{status}'"
    )]
    InvalidFollowUpItemState {
        task_id: Uuid,
        follow_up_id: String,
        status: String,
    },
    #[error("handoff receipt not found for task {0}")]
    HandoffReceiptNotFound(Uuid),
    #[error("handoff receipt requires role '{required_role}', got '{actual_role}'")]
    HandoffReceiptRoleMismatch {
        required_role: String,
        actual_role: String,
    },
    #[error("handoff receipt for task {task_id} cannot transition from status '{status}'")]
    InvalidHandoffReceiptState { task_id: Uuid, status: String },
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

    pub fn get_task_governance(
        &self,
        task_id: Uuid,
    ) -> std::result::Result<WorkflowGovernance, OrchestrationError> {
        self.get_task(task_id)?
            .planned_task
            .task
            .plan
            .map(|plan| plan.governance)
            .ok_or(OrchestrationError::Lifecycle(
                LifecycleError::MissingExecutionPlan,
            ))
    }

    pub fn list_follow_up_items(
        &self,
        query: &FollowUpQueueQuery,
    ) -> std::result::Result<Vec<FollowUpQueueItemView>, OrchestrationError> {
        let now = Utc::now();
        self.collect_follow_up_queue_items(query, now)
    }

    pub fn get_follow_up_monitoring(
        &self,
        query: &FollowUpQueueQuery,
    ) -> std::result::Result<FollowUpMonitoringView, OrchestrationError> {
        let now = Utc::now();
        let items = self.collect_follow_up_queue_items(query, now)?;
        Ok(summarize_follow_up_monitoring(&items, now))
    }

    pub fn list_handoff_receipts(
        &self,
        query: &HandoffReceiptQueueQuery,
    ) -> std::result::Result<Vec<HandoffReceiptQueueItemView>, OrchestrationError> {
        let now = Utc::now();
        self.collect_handoff_receipt_queue_items(query, now)
    }

    pub fn list_alert_clusters(
        &self,
        query: &AlertClusterQueueQuery,
    ) -> std::result::Result<Vec<AlertClusterQueueItemView>, OrchestrationError> {
        let now = Utc::now();
        self.collect_alert_cluster_queue_items(query, now)
    }

    pub fn get_alert_cluster_monitoring(
        &self,
        query: &AlertClusterQueueQuery,
    ) -> std::result::Result<AlertClusterMonitoringView, OrchestrationError> {
        let now = Utc::now();
        let items = self.collect_alert_cluster_queue_items(query, now)?;
        Ok(summarize_alert_cluster_monitoring(&items, now))
    }

    pub fn get_handoff_receipt_monitoring(
        &self,
        query: &HandoffReceiptQueueQuery,
    ) -> std::result::Result<HandoffReceiptMonitoringView, OrchestrationError> {
        let now = Utc::now();
        let items = self.collect_handoff_receipt_queue_items(query, now)?;
        Ok(summarize_handoff_receipt_monitoring(&items, now))
    }

    pub fn plan_task(&self, request: TaskRequest) -> ExecutionPlan {
        let patterns = select_patterns(&request);
        let approval_policy = select_approval_policy(&request);
        let governance = build_governance(&request, approval_policy);
        let steps = build_steps(&request, &patterns, approval_policy);
        let rationale = build_rationale(&request, &patterns, approval_policy);

        ExecutionPlan {
            request_id: request.id,
            patterns,
            rationale,
            approval_policy,
            governance,
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
        let follow_up_items = seed_follow_up_items(&task, &evidence);
        let follow_up_summary = summarize_follow_up_items(&follow_up_items, Utc::now());
        let handoff_receipt = seed_handoff_receipt(&task, &follow_up_items);
        let handoff_receipt_summary =
            summarize_handoff_receipt(handoff_receipt.as_ref(), &follow_up_items);
        let alert_cluster_drafts = seed_alert_cluster_drafts(&task, &evidence);
        let alert_triage_summary = summarize_alert_cluster_drafts(&alert_cluster_drafts);

        let tracked_state = TrackedTaskState {
            correlation_id,
            planned_task: PlannedTaskBundle { task, approval },
            context_reads,
            evidence,
            follow_up_items,
            follow_up_summary,
            handoff_receipt,
            handoff_receipt_summary,
            alert_cluster_drafts,
            alert_triage_summary,
        };
        self.task_repository.create(tracked_state.clone())?;
        if let Some(handoff_receipt) = tracked_state.handoff_receipt.as_ref() {
            self.record_event(
                Some(tracked_state.correlation_id.clone()),
                AuditEventKind::HandoffPublished,
                Some(tracked_state.planned_task.task.id),
                None,
                AuditActor::Human(handoff_receipt.sending_actor.clone()),
                format!(
                    "Handoff receipt '{}' published for task '{}'",
                    handoff_receipt.id, tracked_state.planned_task.task.request.title
                ),
            )?;
        }

        Ok(tracked_state)
    }

    fn collect_follow_up_queue_items(
        &self,
        query: &FollowUpQueueQuery,
        now: DateTime<Utc>,
    ) -> std::result::Result<Vec<FollowUpQueueItemView>, OrchestrationError> {
        let mut items: Vec<FollowUpQueueItemView> = self
            .task_repository
            .list()?
            .into_iter()
            .flat_map(|state| {
                state
                    .follow_up_items
                    .iter()
                    .map(|item| follow_up_queue_item(&state, item, now))
                    .collect::<Vec<_>>()
            })
            .filter(|item| follow_up_queue_matches(query, item))
            .collect();

        items.sort_by(compare_follow_up_queue_items);
        Ok(items)
    }

    fn collect_handoff_receipt_queue_items(
        &self,
        query: &HandoffReceiptQueueQuery,
        now: DateTime<Utc>,
    ) -> std::result::Result<Vec<HandoffReceiptQueueItemView>, OrchestrationError> {
        let mut items: Vec<HandoffReceiptQueueItemView> = self
            .task_repository
            .list()?
            .into_iter()
            .filter_map(|state| handoff_receipt_queue_item(&state, now))
            .filter(|item| handoff_receipt_queue_matches(query, item))
            .collect();

        items.sort_by(compare_handoff_receipt_queue_items);
        Ok(items)
    }

    fn collect_alert_cluster_queue_items(
        &self,
        query: &AlertClusterQueueQuery,
        now: DateTime<Utc>,
    ) -> std::result::Result<Vec<AlertClusterQueueItemView>, OrchestrationError> {
        let mut items: Vec<AlertClusterQueueItemView> = self
            .task_repository
            .list()?
            .into_iter()
            .flat_map(|state| {
                state
                    .alert_cluster_drafts
                    .iter()
                    .map(|cluster| alert_cluster_queue_item(&state, cluster, &now))
                    .collect::<Vec<_>>()
            })
            .filter(|item| alert_cluster_queue_matches(query, item))
            .collect();

        items.sort_by(|left, right| compare_alert_cluster_queue_items(left, right, now));
        Ok(items)
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

    pub fn accept_follow_up_owner(
        &self,
        task_id: Uuid,
        follow_up_id: String,
        action: AcceptFollowUpOwnerRequest,
        correlation_id: Option<String>,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError> {
        let actor = action.actor;
        let note = action
            .note
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let actor_for_update = actor.clone();
        let follow_up_id_for_update = follow_up_id.clone();

        let updated = self.update_task(task_id, correlation_id, move |state| {
            let item = state
                .follow_up_items
                .iter_mut()
                .find(|item| item.id == follow_up_id_for_update)
                .ok_or_else(|| OrchestrationError::FollowUpItemNotFound {
                    task_id,
                    follow_up_id: follow_up_id_for_update.clone(),
                })?;

            if item.status != "draft" || item.accepted_owner_id.is_some() {
                return Err(OrchestrationError::InvalidFollowUpItemState {
                    task_id,
                    follow_up_id: follow_up_id_for_update.clone(),
                    status: item.status.clone(),
                });
            }

            if let Some(required_role) = item.recommended_owner_role.clone() {
                let actual_role = actor_for_update.normalized_role();
                if actual_role != required_role {
                    return Err(OrchestrationError::FollowUpRoleMismatch {
                        follow_up_id: follow_up_id_for_update.clone(),
                        required_role,
                        actual_role,
                    });
                }
            }

            let accepted_at = Utc::now();
            item.accepted_owner_id = Some(actor_for_update.id.clone());
            item.status = "accepted".to_string();
            item.updated_at = accepted_at;
            state.follow_up_summary =
                summarize_follow_up_items(&state.follow_up_items, accepted_at);
            state.handoff_receipt_summary =
                summarize_handoff_receipt(state.handoff_receipt.as_ref(), &state.follow_up_items);
            Ok(())
        })?;

        let follow_up_title = updated
            .follow_up_items
            .iter()
            .find(|item| item.id == follow_up_id)
            .map(|item| item.title.clone())
            .unwrap_or_else(|| follow_up_id.clone());
        self.record_event(
            Some(updated.correlation_id.clone()),
            AuditEventKind::FollowUpOwnerAccepted,
            Some(task_id),
            None,
            AuditActor::Human(actor),
            note.unwrap_or_else(|| {
                format!(
                    "Follow-up '{}' accepted for task '{}'",
                    follow_up_title, updated.planned_task.task.request.title
                )
            }),
        )?;

        Ok(updated)
    }

    pub fn acknowledge_handoff_receipt(
        &self,
        task_id: Uuid,
        action: AcknowledgeHandoffReceiptRequest,
        correlation_id: Option<String>,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError> {
        let actor = action.actor;
        let exception_note = action
            .exception_note
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let actor_for_update = actor.clone();
        let exception_note_for_update = exception_note.clone();

        let updated = self.update_task(task_id, correlation_id, move |state| {
            let receipt = state
                .handoff_receipt
                .as_mut()
                .ok_or(OrchestrationError::HandoffReceiptNotFound(task_id))?;

            if receipt.status != "published" {
                return Err(OrchestrationError::InvalidHandoffReceiptState {
                    task_id,
                    status: receipt.status.clone(),
                });
            }

            let actual_role = actor_for_update.normalized_role();
            if actual_role != receipt.receiving_role {
                return Err(OrchestrationError::HandoffReceiptRoleMismatch {
                    required_role: receipt.receiving_role.clone(),
                    actual_role,
                });
            }

            let acknowledged_at = Utc::now();
            receipt.receiving_actor = Some(actor_for_update.clone());
            receipt.acknowledged_at = Some(acknowledged_at);
            receipt.exception_note = exception_note_for_update.clone();
            receipt.status = if receipt.exception_note.is_some() {
                "acknowledged_with_exceptions".to_string()
            } else {
                "acknowledged".to_string()
            };
            receipt.escalation_state = Some(if receipt.exception_note.is_some() {
                "review_requested".to_string()
            } else {
                "none".to_string()
            });
            receipt.updated_at = acknowledged_at;
            state.handoff_receipt_summary =
                summarize_handoff_receipt(state.handoff_receipt.as_ref(), &state.follow_up_items);
            Ok(())
        })?;

        let event_kind = if exception_note.is_some() {
            AuditEventKind::HandoffAcknowledgedWithExceptions
        } else {
            AuditEventKind::HandoffAcknowledged
        };
        self.record_event(
            Some(updated.correlation_id.clone()),
            event_kind,
            Some(task_id),
            None,
            AuditActor::Human(actor),
            exception_note.unwrap_or_else(|| {
                format!(
                    "Handoff receipt acknowledged for task '{}'",
                    updated.planned_task.task.request.title
                )
            }),
        )?;

        Ok(updated)
    }

    pub fn escalate_handoff_receipt(
        &self,
        task_id: Uuid,
        action: EscalateHandoffReceiptRequest,
        correlation_id: Option<String>,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError> {
        let actor = action.actor;
        let note = action
            .note
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let actor_for_update = actor.clone();

        let updated = self.update_task(task_id, correlation_id, move |state| {
            let receipt = state
                .handoff_receipt
                .as_mut()
                .ok_or(OrchestrationError::HandoffReceiptNotFound(task_id))?;

            if receipt.status != "acknowledged_with_exceptions" {
                return Err(OrchestrationError::InvalidHandoffReceiptState {
                    task_id,
                    status: receipt.status.clone(),
                });
            }

            let required_role = receipt.sending_actor.normalized_role();
            let actual_role = actor_for_update.normalized_role();
            if actual_role != required_role {
                return Err(OrchestrationError::HandoffReceiptRoleMismatch {
                    required_role,
                    actual_role,
                });
            }

            let escalated_at = Utc::now();
            receipt.status = "escalated".to_string();
            receipt.escalation_state = Some("escalated".to_string());
            receipt.updated_at = escalated_at;
            state.handoff_receipt_summary =
                summarize_handoff_receipt(state.handoff_receipt.as_ref(), &state.follow_up_items);
            Ok(())
        })?;

        self.record_event(
            Some(updated.correlation_id.clone()),
            AuditEventKind::HandoffReceiptEscalated,
            Some(task_id),
            None,
            AuditActor::Human(actor),
            note.unwrap_or_else(|| {
                format!(
                    "Handoff receipt escalated for task '{}'",
                    updated.planned_task.task.request.title
                )
            }),
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

fn build_governance(request: &TaskRequest, approval_policy: ApprovalPolicy) -> WorkflowGovernance {
    let mut responsibility_matrix = vec![
        ResponsibilityAssignment {
            role: request.initiator.normalized_role(),
            participation: GovernanceParticipation::Responsible,
            responsibilities: vec![
                "raise the anomaly and describe business impact".to_string(),
                "request revision or resubmission when more evidence is needed".to_string(),
            ],
        },
        ResponsibilityAssignment {
            role: "fa_orchestrator".to_string(),
            participation: GovernanceParticipation::Responsible,
            responsibilities: vec![
                "assemble cross-system evidence and workflow plan".to_string(),
                "record task state, approvals, and audit trail".to_string(),
            ],
        },
    ];

    if !request.equipment_ids.is_empty() {
        responsibility_matrix.push(ResponsibilityAssignment {
            role: "maintenance_engineer".to_string(),
            participation: GovernanceParticipation::Responsible,
            responsibilities: vec![
                "review diagnostic evidence and recommend maintenance action".to_string(),
                "lead execution once approval is granted".to_string(),
            ],
        });
    }

    if matches!(request.risk, TaskRisk::High | TaskRisk::Critical)
        || request
            .integrations
            .iter()
            .any(|target| matches!(target, fa_domain::IntegrationTarget::Quality))
    {
        responsibility_matrix.push(ResponsibilityAssignment {
            role: "quality_engineer".to_string(),
            participation: GovernanceParticipation::Consulted,
            responsibilities: vec![
                "review potential quality impact before execution".to_string(),
                "advise on containment or escalation when product risk is present".to_string(),
            ],
        });
    }

    responsibility_matrix.push(ResponsibilityAssignment {
        role: approval_policy.required_role().to_string(),
        participation: if approval_policy.requires_human_approval() {
            GovernanceParticipation::Accountable
        } else {
            GovernanceParticipation::Informed
        },
        responsibilities: if approval_policy.requires_human_approval() {
            vec!["approve, reject, or request revision before execution".to_string()]
        } else {
            vec!["monitor auto-approved execution path".to_string()]
        },
    });

    WorkflowGovernance {
        responsibility_matrix,
        approval_strategy: ApprovalStrategy {
            policy: approval_policy,
            manual_approval_required: approval_policy.requires_human_approval(),
            required_role: approval_policy.required_role().to_string(),
            escalation_role: approval_policy.escalation_role().map(str::to_string),
            decision_scope: approval_decision_scope(request, approval_policy),
            rationale: approval_rationale(request, approval_policy),
        },
        fallback_actions: vec![
            "record the blocking condition and preserve evidence for audit".to_string(),
            "return the workflow to a named human owner for next action".to_string(),
            "create a follow-up task instead of bypassing approval or policy gates".to_string(),
        ],
    }
}

fn seed_follow_up_items(task: &TaskRecord, evidence: &[TaskEvidence]) -> Vec<FollowUpItemView> {
    let mut items = Vec::new();
    items.extend(seed_shift_handoff_follow_up(task, evidence));
    items.extend(seed_alert_triage_follow_up(task, evidence));
    items
}

fn seed_shift_handoff_follow_up(
    task: &TaskRecord,
    evidence: &[TaskEvidence],
) -> Option<FollowUpItemView> {
    if !is_shift_handoff_request(&task.request) {
        return None;
    }

    let base_time = task.updated_at;
    let source_refs = follow_up_source_refs(task, evidence);

    Some(FollowUpItemView {
        id: format!("fu_{}_handoff_review", task.id.simple()),
        title: "Review outgoing shift handoff and accept remaining work".to_string(),
        summary: Some(
            "Incoming shift supervisor should review the handoff package and confirm the next owner for unresolved work."
                .to_string(),
        ),
        source_kind: "shift_handoff".to_string(),
        source_refs,
        status: "draft".to_string(),
        recommended_owner_role: Some("incoming_shift_supervisor".to_string()),
        accepted_owner_id: None,
        due_at: Some(base_time + chrono::Duration::minutes(30)),
        sla_status: "due_soon".to_string(),
        blocked_reason: None,
        created_at: base_time,
        updated_at: base_time,
    })
}

fn seed_alert_triage_follow_up(
    task: &TaskRecord,
    evidence: &[TaskEvidence],
) -> Option<FollowUpItemView> {
    if !is_alert_triage_request(&task.request) {
        return None;
    }

    let base_time = task.updated_at;
    let source_refs = follow_up_source_refs(task, evidence);
    let inference = infer_alert_triage_cluster(task, evidence);

    let (title, summary) = match inference.triage_label.as_str() {
        "sustained_threshold_review" => (
            "Review sustained threshold cluster and assign diagnostic owner".to_string(),
            Some(
                "Maintenance engineer should review the sustained threshold cluster, confirm equipment containment, and accept the first diagnostic action."
                    .to_string(),
            ),
        ),
        _ => (
            "Review clustered alerts and assign first response owner".to_string(),
            Some(
                "Production supervisor should review the clustered alert burst, confirm containment, and assign the first response owner before escalation."
                    .to_string(),
            ),
        ),
    };

    Some(FollowUpItemView {
        id: format!("fu_{}_alert_triage", task.id.simple()),
        title,
        summary,
        source_kind: "alert_triage".to_string(),
        source_refs,
        status: "draft".to_string(),
        recommended_owner_role: Some(inference.recommended_owner_role),
        accepted_owner_id: None,
        due_at: Some(base_time + chrono::Duration::minutes(15)),
        sla_status: "due_soon".to_string(),
        blocked_reason: None,
        created_at: base_time,
        updated_at: base_time,
    })
}

fn is_shift_handoff_request(request: &TaskRequest) -> bool {
    let haystack = request_text_haystack(request);

    haystack.contains("handoff")
        || haystack.contains("shift notes")
        || haystack.contains("shift summary")
}

fn request_text_haystack(request: &TaskRequest) -> String {
    format!(
        "{} {} {}",
        request.title, request.description, request.desired_outcome
    )
    .to_ascii_lowercase()
}

fn follow_up_source_refs(task: &TaskRecord, evidence: &[TaskEvidence]) -> Vec<String> {
    let refs: Vec<String> = evidence
        .iter()
        .map(|item| item.source_ref.clone())
        .take(3)
        .collect();

    if refs.is_empty() {
        vec![format!("task:{}", task.id)]
    } else {
        refs
    }
}

fn summarize_follow_up_items(
    follow_up_items: &[FollowUpItemView],
    evaluated_at: DateTime<Utc>,
) -> FollowUpSummary {
    if follow_up_items.is_empty() {
        return FollowUpSummary::default();
    }

    let open_items = follow_up_items
        .iter()
        .filter(|item| item.status != "completed")
        .count();
    let blocked_items = follow_up_items
        .iter()
        .filter(|item| item.status == "blocked")
        .count();
    let overdue_items = follow_up_items
        .iter()
        .filter(|item| item.sla_status == "overdue")
        .count();
    let escalated_items = follow_up_items
        .iter()
        .filter(|item| item.status == "escalated" || item.sla_status == "escalation_required")
        .count();

    FollowUpSummary {
        total_items: follow_up_items.len(),
        open_items,
        blocked_items,
        overdue_items,
        escalated_items,
        last_evaluated_at: Some(evaluated_at),
    }
}

fn seed_handoff_receipt(
    task: &TaskRecord,
    follow_up_items: &[FollowUpItemView],
) -> Option<HandoffReceiptView> {
    if !is_shift_handoff_request(&task.request) {
        return None;
    }

    let published_at = task.updated_at;

    Some(HandoffReceiptView {
        id: format!("hr_{}_published", task.id.simple()),
        handoff_task_id: task.id,
        shift_id: format!("shift_handoff_{}", task.id.simple()),
        sending_actor: task.request.initiator.clone(),
        receiving_role: "incoming_shift_supervisor".to_string(),
        receiving_actor: None,
        published_at,
        required_ack_by: Some(published_at + chrono::Duration::minutes(30)),
        status: "published".to_string(),
        follow_up_item_ids: follow_up_items.iter().map(|item| item.id.clone()).collect(),
        exception_note: None,
        acknowledged_at: None,
        escalation_state: Some("none".to_string()),
        created_at: published_at,
        updated_at: published_at,
    })
}

fn summarize_handoff_receipt(
    handoff_receipt: Option<&HandoffReceiptView>,
    follow_up_items: &[FollowUpItemView],
) -> HandoffReceiptSummary {
    let Some(handoff_receipt) = handoff_receipt else {
        return HandoffReceiptSummary::default();
    };
    let unaccepted_follow_up_count = follow_up_items
        .iter()
        .filter(|item| item.accepted_owner_id.is_none())
        .count();

    HandoffReceiptSummary {
        status: Some(handoff_receipt.status.clone()),
        published_at: Some(handoff_receipt.published_at),
        required_ack_by: handoff_receipt.required_ack_by,
        acknowledged_at: handoff_receipt.acknowledged_at,
        covered_follow_up_count: handoff_receipt.follow_up_item_ids.len(),
        unaccepted_follow_up_count,
        exception_flag: handoff_receipt.exception_note.is_some(),
    }
}

fn summarize_follow_up_monitoring(
    items: &[FollowUpQueueItemView],
    now: DateTime<Utc>,
) -> FollowUpMonitoringView {
    let open_items: Vec<&FollowUpQueueItemView> = items
        .iter()
        .filter(|item| follow_up_queue_item_is_open(item))
        .collect();
    let next_due_at = open_items.iter().filter_map(|item| item.due_at).min();

    FollowUpMonitoringView {
        total_items: items.len(),
        open_items: open_items.len(),
        accepted_items: open_items
            .iter()
            .filter(|item| item.accepted_owner_id.is_some())
            .count(),
        unaccepted_items: open_items
            .iter()
            .filter(|item| item.accepted_owner_id.is_none())
            .count(),
        blocked_items: open_items
            .iter()
            .filter(|item| follow_up_queue_item_is_blocked(item))
            .count(),
        overdue_items: open_items.iter().filter(|item| item.overdue).count(),
        escalation_required_items: open_items
            .iter()
            .filter(|item| follow_up_queue_item_requires_escalation(item))
            .count(),
        next_due_at,
        source_kind_counts: follow_up_monitoring_buckets(
            open_items.iter().map(|item| item.source_kind.clone()),
        ),
        owner_role_counts: follow_up_monitoring_buckets(open_items.iter().map(|item| {
            item.recommended_owner_role
                .clone()
                .unwrap_or_else(|| "unassigned".to_string())
        })),
        sla_status_counts: follow_up_monitoring_buckets(
            open_items
                .iter()
                .map(|item| item.effective_sla_status.clone()),
        ),
        task_risk_counts: follow_up_monitoring_buckets(
            open_items
                .iter()
                .map(|item| task_risk_label(item.task_risk).to_string()),
        ),
        task_priority_counts: follow_up_monitoring_buckets(
            open_items
                .iter()
                .map(|item| task_priority_label(item.task_priority).to_string()),
        ),
        last_evaluated_at: now,
    }
}

fn summarize_handoff_receipt_monitoring(
    items: &[HandoffReceiptQueueItemView],
    now: DateTime<Utc>,
) -> HandoffReceiptMonitoringView {
    let open_receipts: Vec<&HandoffReceiptQueueItemView> = items
        .iter()
        .filter(|item| handoff_receipt_queue_item_is_open(item))
        .collect();
    let unacknowledged_receipts: Vec<&HandoffReceiptQueueItemView> = items
        .iter()
        .filter(|item| handoff_receipt_queue_item_is_unacknowledged(item))
        .collect();
    let next_ack_due_at = unacknowledged_receipts
        .iter()
        .filter_map(|item| item.required_ack_by)
        .min();

    HandoffReceiptMonitoringView {
        total_receipts: items.len(),
        open_receipts: open_receipts.len(),
        acknowledged_receipts: items
            .iter()
            .filter(|item| handoff_receipt_queue_item_is_acknowledged(item))
            .count(),
        unacknowledged_receipts: unacknowledged_receipts.len(),
        overdue_receipts: open_receipts.iter().filter(|item| item.overdue).count(),
        exception_receipts: open_receipts
            .iter()
            .filter(|item| item.has_exceptions)
            .count(),
        escalated_receipts: open_receipts
            .iter()
            .filter(|item| item.effective_status == "escalated")
            .count(),
        next_ack_due_at,
        effective_status_counts: handoff_receipt_monitoring_buckets(
            open_receipts
                .iter()
                .map(|item| item.effective_status.clone()),
        ),
        receiving_role_counts: handoff_receipt_monitoring_buckets(
            open_receipts.iter().map(|item| item.receiving_role.clone()),
        ),
        ack_window_counts: handoff_receipt_monitoring_buckets(
            unacknowledged_receipts
                .iter()
                .map(|item| handoff_receipt_ack_window_key(item, now)),
        ),
        task_risk_counts: handoff_receipt_monitoring_buckets(
            open_receipts
                .iter()
                .map(|item| task_risk_label(item.task_risk).to_string()),
        ),
        task_priority_counts: handoff_receipt_monitoring_buckets(
            open_receipts
                .iter()
                .map(|item| task_priority_label(item.task_priority).to_string()),
        ),
        last_evaluated_at: now,
    }
}

fn summarize_alert_cluster_monitoring(
    items: &[AlertClusterQueueItemView],
    now: DateTime<Utc>,
) -> AlertClusterMonitoringView {
    let next_window_end_at = items.iter().map(|item| item.window_end).min();

    AlertClusterMonitoringView {
        total_clusters: items.len(),
        open_clusters: items
            .iter()
            .filter(|item| alert_cluster_queue_item_is_open(item))
            .count(),
        escalation_candidate_clusters: items
            .iter()
            .filter(|item| item.escalation_candidate)
            .count(),
        high_severity_clusters: items
            .iter()
            .filter(|item| alert_cluster_queue_item_has_high_severity(item))
            .count(),
        active_window_clusters: items
            .iter()
            .filter(|item| alert_cluster_window_state_key(item, now) == "active")
            .count(),
        stale_window_clusters: items
            .iter()
            .filter(|item| alert_cluster_window_state_key(item, now) == "stale")
            .count(),
        linked_follow_up_clusters: items
            .iter()
            .filter(|item| alert_cluster_queue_item_has_linked_follow_up(item))
            .count(),
        unlinked_follow_up_clusters: items
            .iter()
            .filter(|item| !alert_cluster_queue_item_has_linked_follow_up(item))
            .count(),
        accepted_follow_up_clusters: items
            .iter()
            .filter(|item| alert_cluster_queue_item_has_accepted_follow_up(item))
            .count(),
        unaccepted_follow_up_clusters: items
            .iter()
            .filter(|item| alert_cluster_queue_item_has_unaccepted_follow_up(item))
            .count(),
        follow_up_escalation_clusters: items
            .iter()
            .filter(|item| alert_cluster_queue_item_requires_follow_up_escalation(item))
            .count(),
        next_window_end_at,
        cluster_status_counts: alert_cluster_monitoring_buckets(
            items.iter().map(|item| item.cluster_status.clone()),
        ),
        source_system_counts: alert_cluster_monitoring_buckets(items.iter().map(|item| {
            item.source_system
                .clone()
                .unwrap_or_else(|| "unknown".to_string())
        })),
        severity_band_counts: alert_cluster_monitoring_buckets(
            items.iter().map(|item| item.severity_band.clone()),
        ),
        triage_label_counts: alert_cluster_monitoring_buckets(items.iter().map(|item| {
            item.triage_label
                .clone()
                .unwrap_or_else(|| "unclassified".to_string())
        })),
        owner_role_counts: alert_cluster_monitoring_buckets(items.iter().map(|item| {
            item.recommended_owner_role
                .clone()
                .unwrap_or_else(|| "unassigned".to_string())
        })),
        window_state_counts: alert_cluster_monitoring_buckets(
            items
                .iter()
                .map(|item| alert_cluster_window_state_key(item, now)),
        ),
        follow_up_coverage_counts: alert_cluster_monitoring_buckets(
            items
                .iter()
                .map(alert_cluster_queue_item_follow_up_coverage_key),
        ),
        follow_up_sla_status_counts: alert_cluster_monitoring_buckets(
            items.iter().map(alert_cluster_queue_item_follow_up_sla_key),
        ),
        task_risk_counts: alert_cluster_monitoring_buckets(
            items
                .iter()
                .map(|item| task_risk_label(item.task_risk).to_string()),
        ),
        task_priority_counts: alert_cluster_monitoring_buckets(
            items
                .iter()
                .map(|item| task_priority_label(item.task_priority).to_string()),
        ),
        last_evaluated_at: now,
    }
}

fn follow_up_queue_item(
    state: &TrackedTaskState,
    item: &FollowUpItemView,
    now: DateTime<Utc>,
) -> FollowUpQueueItemView {
    let effective_sla_status = effective_follow_up_sla_status(item, now);
    let overdue =
        effective_sla_status == "overdue" || effective_sla_status == "escalation_required";

    FollowUpQueueItemView {
        task_id: state.planned_task.task.id,
        correlation_id: state.correlation_id.clone(),
        task_title: state.planned_task.task.request.title.clone(),
        task_priority: state.planned_task.task.request.priority,
        task_risk: state.planned_task.task.request.risk,
        task_status: state.planned_task.task.status,
        follow_up_id: item.id.clone(),
        title: item.title.clone(),
        summary: item.summary.clone(),
        source_kind: item.source_kind.clone(),
        source_refs: item.source_refs.clone(),
        status: item.status.clone(),
        recommended_owner_role: item.recommended_owner_role.clone(),
        accepted_owner_id: item.accepted_owner_id.clone(),
        due_at: item.due_at,
        sla_status: item.sla_status.clone(),
        effective_sla_status,
        overdue,
        blocked_reason: item.blocked_reason.clone(),
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn effective_follow_up_sla_status(item: &FollowUpItemView, now: DateTime<Utc>) -> String {
    if item.status == "escalated" || item.sla_status == "escalation_required" {
        return "escalation_required".to_string();
    }

    if item.status != "completed" && item.due_at.is_some_and(|due_at| due_at < now) {
        return "overdue".to_string();
    }

    item.sla_status.clone()
}

fn handoff_receipt_queue_item(
    state: &TrackedTaskState,
    now: DateTime<Utc>,
) -> Option<HandoffReceiptQueueItemView> {
    let receipt = state.handoff_receipt.as_ref()?;
    let summary = summarize_handoff_receipt(Some(receipt), &state.follow_up_items);
    let effective_status = effective_handoff_receipt_status(receipt, now);
    let overdue = effective_status == "expired";

    Some(HandoffReceiptQueueItemView {
        task_id: state.planned_task.task.id,
        correlation_id: state.correlation_id.clone(),
        task_title: state.planned_task.task.request.title.clone(),
        task_priority: state.planned_task.task.request.priority,
        task_risk: state.planned_task.task.request.risk,
        task_status: state.planned_task.task.status,
        receipt_id: receipt.id.clone(),
        shift_id: receipt.shift_id.clone(),
        receipt_status: receipt.status.clone(),
        effective_status,
        sending_actor: receipt.sending_actor.clone(),
        receiving_role: receipt.receiving_role.clone(),
        receiving_actor: receipt.receiving_actor.clone(),
        published_at: receipt.published_at,
        required_ack_by: receipt.required_ack_by,
        acknowledged_at: receipt.acknowledged_at,
        follow_up_item_ids: receipt.follow_up_item_ids.clone(),
        covered_follow_up_count: summary.covered_follow_up_count,
        unaccepted_follow_up_count: summary.unaccepted_follow_up_count,
        has_exceptions: summary.exception_flag,
        exception_note: receipt.exception_note.clone(),
        escalation_state: receipt.escalation_state.clone(),
        overdue,
        created_at: receipt.created_at,
        updated_at: receipt.updated_at,
    })
}

fn alert_cluster_queue_item(
    state: &TrackedTaskState,
    cluster: &AlertClusterDraftView,
    now: &DateTime<Utc>,
) -> AlertClusterQueueItemView {
    AlertClusterQueueItemView {
        task_id: state.planned_task.task.id,
        correlation_id: state.correlation_id.clone(),
        task_title: state.planned_task.task.request.title.clone(),
        task_priority: state.planned_task.task.request.priority,
        task_risk: state.planned_task.task.request.risk,
        task_status: state.planned_task.task.status,
        cluster_id: cluster.cluster_id.clone(),
        cluster_status: cluster.cluster_status.clone(),
        source_system: cluster.source_system.clone(),
        equipment_id: cluster.equipment_id.clone(),
        line_id: cluster.line_id.clone(),
        severity_band: cluster.severity_band.clone(),
        source_event_refs: cluster.source_event_refs.clone(),
        window_start: cluster.window_start,
        window_end: cluster.window_end,
        triage_label: cluster.triage_label.clone(),
        recommended_owner_role: cluster.recommended_owner_role.clone(),
        escalation_candidate: cluster.escalation_candidate,
        linked_follow_up: summarize_alert_cluster_linked_follow_up(state, cluster, now),
        rationale: cluster.rationale.clone(),
        created_at: cluster.created_at,
        updated_at: cluster.updated_at,
    }
}

fn summarize_alert_cluster_linked_follow_up(
    state: &TrackedTaskState,
    cluster: &AlertClusterDraftView,
    now: &DateTime<Utc>,
) -> Option<AlertClusterLinkedFollowUpView> {
    let linked_items: Vec<&FollowUpItemView> = state
        .follow_up_items
        .iter()
        .filter(|item| alert_cluster_links_follow_up(state, cluster, item))
        .collect();

    if linked_items.is_empty() {
        return None;
    }

    let total_items = linked_items.len();
    let open_items = linked_items
        .iter()
        .filter(|item| item.status != "completed")
        .count();
    let accepted_items = linked_items
        .iter()
        .filter(|item| item.accepted_owner_id.is_some())
        .count();
    let mut follow_up_ids: Vec<String> = linked_items.iter().map(|item| item.id.clone()).collect();
    follow_up_ids.sort();

    let accepted_owner_ids = linked_items
        .iter()
        .filter_map(|item| item.accepted_owner_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let worst_effective_sla_status = linked_items
        .iter()
        .map(|item| effective_follow_up_sla_status(item, now.to_owned()))
        .min_by_key(|status| alert_cluster_follow_up_sla_rank(status));

    Some(AlertClusterLinkedFollowUpView {
        total_items,
        open_items,
        accepted_items,
        unaccepted_items: total_items - accepted_items,
        follow_up_ids,
        accepted_owner_ids,
        worst_effective_sla_status,
    })
}

fn alert_cluster_links_follow_up(
    state: &TrackedTaskState,
    cluster: &AlertClusterDraftView,
    item: &FollowUpItemView,
) -> bool {
    if !matches!(item.source_kind.as_str(), "alert_cluster" | "alert_triage") {
        return false;
    }

    follow_up_refs_link_alert_cluster(item, cluster) || state.alert_cluster_drafts.len() == 1
}

fn follow_up_refs_link_alert_cluster(
    item: &FollowUpItemView,
    cluster: &AlertClusterDraftView,
) -> bool {
    item.source_refs.iter().any(|source_ref| {
        source_ref == &cluster.cluster_id
            || cluster
                .source_event_refs
                .iter()
                .any(|cluster_ref| cluster_ref == source_ref)
    })
}

fn effective_handoff_receipt_status(receipt: &HandoffReceiptView, now: DateTime<Utc>) -> String {
    if receipt.status == "escalated" || receipt.escalation_state.as_deref() == Some("escalated") {
        return "escalated".to_string();
    }

    if receipt.status == "published"
        && receipt
            .required_ack_by
            .is_some_and(|required_ack_by| required_ack_by < now)
    {
        return "expired".to_string();
    }

    receipt.status.clone()
}

fn follow_up_queue_matches(query: &FollowUpQueueQuery, item: &FollowUpQueueItemView) -> bool {
    query.task_id.is_none_or(|task_id| item.task_id == task_id)
        && query
            .source_kind
            .as_ref()
            .is_none_or(|source_kind| item.source_kind == *source_kind)
        && query
            .status
            .as_ref()
            .is_none_or(|status| item.status == *status)
        && query
            .owner_id
            .as_ref()
            .is_none_or(|owner_id| item.accepted_owner_id.as_ref() == Some(owner_id))
        && query
            .owner_role
            .as_ref()
            .is_none_or(|owner_role| item.recommended_owner_role.as_ref() == Some(owner_role))
        && (!query.overdue_only || item.overdue)
        && (!query.blocked_only || follow_up_queue_item_is_blocked(item))
        && (!query.escalation_required || follow_up_queue_item_requires_escalation(item))
        && query
            .due_before
            .is_none_or(|due_before| item.due_at.is_some_and(|due_at| due_at <= due_before))
        && query
            .task_risk
            .is_none_or(|task_risk| item.task_risk == task_risk)
        && query
            .task_priority
            .is_none_or(|task_priority| item.task_priority == task_priority)
}

fn follow_up_queue_item_is_blocked(item: &FollowUpQueueItemView) -> bool {
    item.status == "blocked" || item.blocked_reason.is_some()
}

fn follow_up_queue_item_is_open(item: &FollowUpQueueItemView) -> bool {
    item.status != "completed"
}

fn follow_up_queue_item_requires_escalation(item: &FollowUpQueueItemView) -> bool {
    item.effective_sla_status == "escalation_required"
}

fn alert_cluster_queue_item_has_linked_follow_up(item: &AlertClusterQueueItemView) -> bool {
    item.linked_follow_up.is_some()
}

fn alert_cluster_queue_item_has_accepted_follow_up(item: &AlertClusterQueueItemView) -> bool {
    item.linked_follow_up
        .as_ref()
        .is_some_and(|linked_follow_up| linked_follow_up.accepted_items > 0)
}

fn alert_cluster_queue_item_has_follow_up_owner(
    item: &AlertClusterQueueItemView,
    owner_id: &str,
) -> bool {
    item.linked_follow_up
        .as_ref()
        .is_some_and(|linked_follow_up| {
            linked_follow_up
                .accepted_owner_ids
                .iter()
                .any(|accepted_owner_id| accepted_owner_id == owner_id)
        })
}

fn alert_cluster_queue_item_has_unaccepted_follow_up(item: &AlertClusterQueueItemView) -> bool {
    item.linked_follow_up
        .as_ref()
        .is_some_and(|linked_follow_up| linked_follow_up.unaccepted_items > 0)
}

fn alert_cluster_queue_item_requires_follow_up_escalation(
    item: &AlertClusterQueueItemView,
) -> bool {
    item.linked_follow_up
        .as_ref()
        .and_then(|linked_follow_up| linked_follow_up.worst_effective_sla_status.as_deref())
        == Some("escalation_required")
}

fn alert_cluster_queue_item_follow_up_coverage_key(item: &AlertClusterQueueItemView) -> String {
    if alert_cluster_queue_item_has_linked_follow_up(item) {
        "linked".to_string()
    } else {
        "no_follow_up".to_string()
    }
}

fn alert_cluster_queue_item_follow_up_sla_key(item: &AlertClusterQueueItemView) -> String {
    item.linked_follow_up
        .as_ref()
        .and_then(|linked_follow_up| linked_follow_up.worst_effective_sla_status.clone())
        .unwrap_or_else(|| "no_follow_up".to_string())
}

fn alert_cluster_follow_up_sla_rank(status: &str) -> u8 {
    match status {
        "escalation_required" => 0,
        "overdue" => 1,
        "due_soon" => 2,
        "on_track" => 3,
        _ => 4,
    }
}

fn handoff_receipt_queue_item_is_open(item: &HandoffReceiptQueueItemView) -> bool {
    item.effective_status != "acknowledged"
}

fn handoff_receipt_queue_item_is_acknowledged(item: &HandoffReceiptQueueItemView) -> bool {
    item.acknowledged_at.is_some()
}

fn handoff_receipt_queue_item_is_unacknowledged(item: &HandoffReceiptQueueItemView) -> bool {
    item.acknowledged_at.is_none()
}

fn handoff_receipt_ack_window_key(
    item: &HandoffReceiptQueueItemView,
    now: DateTime<Utc>,
) -> String {
    match item.required_ack_by {
        Some(required_ack_by) if required_ack_by < now => "overdue".to_string(),
        Some(required_ack_by) if required_ack_by <= now + chrono::Duration::minutes(30) => {
            "due_within_30m".to_string()
        }
        Some(required_ack_by) if required_ack_by <= now + chrono::Duration::hours(2) => {
            "due_within_2h".to_string()
        }
        Some(_) => "future".to_string(),
        None => "no_deadline".to_string(),
    }
}

fn handoff_receipt_queue_matches(
    query: &HandoffReceiptQueueQuery,
    item: &HandoffReceiptQueueItemView,
) -> bool {
    query.task_id.is_none_or(|task_id| item.task_id == task_id)
        && query
            .shift_id
            .as_ref()
            .is_none_or(|shift_id| item.shift_id == *shift_id)
        && query.receipt_status.as_ref().is_none_or(|receipt_status| {
            item.receipt_status == *receipt_status || item.effective_status == *receipt_status
        })
        && query
            .receiving_role
            .as_ref()
            .is_none_or(|receiving_role| item.receiving_role == *receiving_role)
        && query
            .receiving_actor_id
            .as_ref()
            .is_none_or(|receiving_actor_id| {
                item.receiving_actor.as_ref().map(|actor| actor.id.as_str())
                    == Some(receiving_actor_id.as_str())
            })
        && (!query.overdue_only || item.overdue)
        && (!query.has_exceptions || item.has_exceptions)
        && (!query.escalated_only || item.effective_status == "escalated")
}

fn alert_cluster_queue_matches(
    query: &AlertClusterQueueQuery,
    item: &AlertClusterQueueItemView,
) -> bool {
    query.task_id.is_none_or(|task_id| item.task_id == task_id)
        && query
            .cluster_status
            .as_ref()
            .is_none_or(|cluster_status| item.cluster_status == *cluster_status)
        && query
            .source_system
            .as_ref()
            .is_none_or(|source_system| item.source_system.as_ref() == Some(source_system))
        && query
            .equipment_id
            .as_ref()
            .is_none_or(|equipment_id| item.equipment_id.as_ref() == Some(equipment_id))
        && query
            .line_id
            .as_ref()
            .is_none_or(|line_id| item.line_id.as_ref() == Some(line_id))
        && query
            .severity_band
            .as_ref()
            .is_none_or(|severity_band| item.severity_band == *severity_band)
        && query
            .triage_label
            .as_ref()
            .is_none_or(|triage_label| item.triage_label.as_ref() == Some(triage_label))
        && query
            .recommended_owner_role
            .as_ref()
            .is_none_or(|owner_role| item.recommended_owner_role.as_ref() == Some(owner_role))
        && query
            .follow_up_owner_id
            .as_ref()
            .is_none_or(|owner_id| alert_cluster_queue_item_has_follow_up_owner(item, owner_id))
        && (!query.unaccepted_follow_up_only
            || alert_cluster_queue_item_has_unaccepted_follow_up(item))
        && (!query.follow_up_escalation_required
            || alert_cluster_queue_item_requires_follow_up_escalation(item))
        && (!query.escalation_candidate || item.escalation_candidate)
        && query
            .window_from
            .is_none_or(|window_from| item.window_end >= window_from)
        && query
            .window_to
            .is_none_or(|window_to| item.window_start <= window_to)
        && (!query.open_only || alert_cluster_queue_item_is_open(item))
}

fn compare_follow_up_queue_items(
    left: &FollowUpQueueItemView,
    right: &FollowUpQueueItemView,
) -> std::cmp::Ordering {
    follow_up_sla_rank(&left.effective_sla_status)
        .cmp(&follow_up_sla_rank(&right.effective_sla_status))
        .then_with(|| compare_optional_due_at(left.due_at, right.due_at))
        .then_with(|| {
            task_priority_rank(left.task_priority).cmp(&task_priority_rank(right.task_priority))
        })
        .then_with(|| right.updated_at.cmp(&left.updated_at))
        .then_with(|| left.follow_up_id.cmp(&right.follow_up_id))
}

fn compare_handoff_receipt_queue_items(
    left: &HandoffReceiptQueueItemView,
    right: &HandoffReceiptQueueItemView,
) -> std::cmp::Ordering {
    handoff_receipt_status_rank(&left.effective_status)
        .cmp(&handoff_receipt_status_rank(&right.effective_status))
        .then_with(|| compare_optional_due_at(left.required_ack_by, right.required_ack_by))
        .then_with(|| {
            task_priority_rank(left.task_priority).cmp(&task_priority_rank(right.task_priority))
        })
        .then_with(|| right.updated_at.cmp(&left.updated_at))
        .then_with(|| left.receipt_id.cmp(&right.receipt_id))
}

fn compare_alert_cluster_queue_items(
    left: &AlertClusterQueueItemView,
    right: &AlertClusterQueueItemView,
    now: DateTime<Utc>,
) -> std::cmp::Ordering {
    alert_cluster_escalation_rank(left.escalation_candidate)
        .cmp(&alert_cluster_escalation_rank(right.escalation_candidate))
        .then_with(|| {
            alert_cluster_status_rank(&left.cluster_status)
                .cmp(&alert_cluster_status_rank(&right.cluster_status))
        })
        .then_with(|| {
            alert_cluster_severity_rank(&left.severity_band)
                .cmp(&alert_cluster_severity_rank(&right.severity_band))
        })
        .then_with(|| {
            alert_cluster_window_rank(left, now).cmp(&alert_cluster_window_rank(right, now))
        })
        .then_with(|| right.window_end.cmp(&left.window_end))
        .then_with(|| right.updated_at.cmp(&left.updated_at))
        .then_with(|| left.cluster_id.cmp(&right.cluster_id))
}

fn compare_optional_due_at(
    left: Option<DateTime<Utc>>,
    right: Option<DateTime<Utc>>,
) -> std::cmp::Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn follow_up_sla_rank(status: &str) -> u8 {
    match status {
        "escalation_required" => 0,
        "overdue" => 1,
        "due_soon" => 2,
        "on_track" => 3,
        _ => 4,
    }
}

fn handoff_receipt_status_rank(status: &str) -> u8 {
    match status {
        "expired" => 0,
        "escalated" => 1,
        "acknowledged_with_exceptions" => 2,
        "published" => 3,
        "acknowledged" => 4,
        _ => 5,
    }
}

fn alert_cluster_queue_item_is_open(item: &AlertClusterQueueItemView) -> bool {
    item.cluster_status == "open"
}

fn alert_cluster_queue_item_has_high_severity(item: &AlertClusterQueueItemView) -> bool {
    matches!(item.severity_band.as_str(), "high" | "critical")
}

fn alert_cluster_window_state_key(item: &AlertClusterQueueItemView, now: DateTime<Utc>) -> String {
    if item.window_end < now {
        "stale".to_string()
    } else if item.window_start > now {
        "future".to_string()
    } else {
        "active".to_string()
    }
}

fn alert_cluster_escalation_rank(escalation_candidate: bool) -> u8 {
    if escalation_candidate {
        0
    } else {
        1
    }
}

fn alert_cluster_status_rank(status: &str) -> u8 {
    match status {
        "open" => 0,
        "investigating" => 1,
        "escalated" => 2,
        "closed" => 3,
        _ => 4,
    }
}

fn alert_cluster_severity_rank(severity_band: &str) -> u8 {
    match severity_band {
        "critical" => 0,
        "high" => 1,
        "medium" => 2,
        "low" => 3,
        _ => 4,
    }
}

fn alert_cluster_window_rank(item: &AlertClusterQueueItemView, now: DateTime<Utc>) -> u8 {
    if item.window_start <= now && item.window_end >= now {
        0
    } else if item.window_end > now {
        1
    } else {
        2
    }
}

fn task_priority_rank(priority: TaskPriority) -> u8 {
    match priority {
        TaskPriority::Critical => 0,
        TaskPriority::Expedited => 1,
        TaskPriority::Routine => 2,
    }
}

fn task_priority_label(priority: TaskPriority) -> &'static str {
    match priority {
        TaskPriority::Routine => "routine",
        TaskPriority::Expedited => "expedited",
        TaskPriority::Critical => "critical",
    }
}

fn task_risk_label(risk: TaskRisk) -> &'static str {
    match risk {
        TaskRisk::Low => "low",
        TaskRisk::Medium => "medium",
        TaskRisk::High => "high",
        TaskRisk::Critical => "critical",
    }
}

fn follow_up_monitoring_buckets<I>(keys: I) -> Vec<FollowUpMonitoringBucket>
where
    I: Iterator<Item = String>,
{
    bucket_counts(keys)
        .into_iter()
        .map(|(key, count)| FollowUpMonitoringBucket { key, count })
        .collect()
}

fn handoff_receipt_monitoring_buckets<I>(keys: I) -> Vec<HandoffReceiptMonitoringBucket>
where
    I: Iterator<Item = String>,
{
    bucket_counts(keys)
        .into_iter()
        .map(|(key, count)| HandoffReceiptMonitoringBucket { key, count })
        .collect()
}

fn alert_cluster_monitoring_buckets<I>(keys: I) -> Vec<AlertClusterMonitoringBucket>
where
    I: Iterator<Item = String>,
{
    bucket_counts(keys)
        .into_iter()
        .map(|(key, count)| AlertClusterMonitoringBucket { key, count })
        .collect()
}

fn bucket_counts<I>(keys: I) -> BTreeMap<String, usize>
where
    I: Iterator<Item = String>,
{
    let mut counts = BTreeMap::<String, usize>::new();

    for key in keys {
        *counts.entry(key).or_default() += 1;
    }

    counts
}

fn seed_alert_cluster_drafts(
    task: &TaskRecord,
    evidence: &[TaskEvidence],
) -> Vec<AlertClusterDraftView> {
    seed_alert_cluster_draft(task, evidence)
        .into_iter()
        .collect()
}

fn seed_alert_cluster_draft(
    task: &TaskRecord,
    evidence: &[TaskEvidence],
) -> Option<AlertClusterDraftView> {
    if !is_alert_triage_request(&task.request) {
        return None;
    }

    let base_time = task.updated_at;
    let inference = infer_alert_triage_cluster(task, evidence);
    let severity_band = severity_band(task.request.risk).to_string();
    let source_system = inference.source_system.clone();

    Some(AlertClusterDraftView {
        cluster_id: format!("ac_{}_triage", task.id.simple()),
        cluster_status: "open".to_string(),
        source_system,
        equipment_id: task.request.equipment_ids.first().cloned(),
        line_id: inference.line_id,
        severity_band,
        source_event_refs: alert_source_event_refs(
            task,
            evidence,
            inference.source_system.as_deref(),
        ),
        window_start: base_time,
        window_end: base_time + inference.window_duration,
        triage_label: Some(inference.triage_label.clone()),
        recommended_owner_role: Some(inference.recommended_owner_role.clone()),
        escalation_candidate: matches!(task.request.risk, TaskRisk::High | TaskRisk::Critical),
        rationale: Some(alert_cluster_rationale(&inference.triage_label).to_string()),
        created_at: base_time,
        updated_at: base_time,
    })
}

fn is_alert_triage_request(request: &TaskRequest) -> bool {
    let haystack = request_text_haystack(request);

    haystack.contains("alert") || haystack.contains("andon") || haystack.contains("triage")
}

struct AlertTriageClusterInference {
    source_system: Option<String>,
    line_id: Option<String>,
    triage_label: String,
    recommended_owner_role: String,
    window_duration: chrono::Duration,
}

fn infer_alert_triage_cluster(
    task: &TaskRecord,
    evidence: &[TaskEvidence],
) -> AlertTriageClusterInference {
    let triage_label = infer_alert_triage_label(&task.request);

    AlertTriageClusterInference {
        source_system: infer_alert_source_system(&task.request, evidence),
        line_id: infer_alert_line_id(&task.request),
        recommended_owner_role: alert_triage_recommended_owner_role(&triage_label).to_string(),
        window_duration: alert_cluster_window_duration(&triage_label),
        triage_label,
    }
}

fn infer_alert_triage_label(request: &TaskRequest) -> String {
    let haystack = request_text_haystack(request);

    if contains_any(
        &haystack,
        &[
            "threshold",
            "drift",
            "spike",
            "temperature",
            "vibration",
            "pressure",
        ],
    ) {
        "sustained_threshold_review".to_string()
    } else if contains_any(&haystack, &["repeated", "andon", "burst", "repeat"]) {
        "repeated_alert_review".to_string()
    } else {
        "first_response_review".to_string()
    }
}

fn alert_triage_recommended_owner_role(triage_label: &str) -> &'static str {
    match triage_label {
        "sustained_threshold_review" => "maintenance_engineer",
        _ => "production_supervisor",
    }
}

fn alert_cluster_window_duration(triage_label: &str) -> chrono::Duration {
    match triage_label {
        "sustained_threshold_review" => chrono::Duration::minutes(15),
        "first_response_review" => chrono::Duration::minutes(10),
        _ => chrono::Duration::minutes(5),
    }
}

fn infer_alert_source_system(request: &TaskRequest, evidence: &[TaskEvidence]) -> Option<String> {
    let haystack = request_text_haystack(request);
    if haystack.contains("andon") {
        return Some("andon".to_string());
    }
    if haystack.contains("scada") {
        return Some("scada".to_string());
    }
    if haystack.contains("incident log") || haystack.contains("incident_log") {
        return Some("incident_log".to_string());
    }

    evidence
        .iter()
        .find_map(|item| source_system_from_ref(&item.source_ref))
        .or_else(|| {
            request
                .integrations
                .first()
                .map(integration_target_name)
                .map(str::to_string)
        })
}

fn source_system_from_ref(source_ref: &str) -> Option<String> {
    let prefix = source_ref
        .split_once("://")
        .map(|(value, _)| value)
        .or_else(|| source_ref.split_once(':').map(|(value, _)| value))?;

    if prefix == "task" {
        None
    } else {
        Some(prefix.to_string())
    }
}

fn infer_alert_line_id(request: &TaskRequest) -> Option<String> {
    let haystack = request_text_haystack(request);
    if let Some(line_id) = parse_line_id_from_text(&haystack) {
        return Some(line_id);
    }

    request
        .equipment_ids
        .first()
        .and_then(|equipment_id| line_id_from_equipment_id(equipment_id))
}

fn parse_line_id_from_text(haystack: &str) -> Option<String> {
    let tokens: Vec<&str> = haystack
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect();

    for window in tokens.windows(3) {
        if window[1] != "line" {
            continue;
        }

        let area = normalize_line_area(window[0]);
        if area.is_empty() {
            continue;
        }

        let suffix = normalize_line_suffix(window[2]);
        return Some(match suffix {
            Some(suffix) => format!("line_{area}_{suffix}"),
            None => format!("line_{area}"),
        });
    }

    None
}

fn line_id_from_equipment_id(equipment_id: &str) -> Option<String> {
    let suffix = equipment_id.strip_prefix("eq_")?;
    let mut parts = suffix.split('_');
    let area = normalize_line_area(parts.next()?);
    if area.is_empty() {
        return None;
    }
    let number = parts.next().and_then(normalize_line_suffix);

    Some(match number {
        Some(number) => format!("line_{area}_{number}"),
        None => format!("line_{area}"),
    })
}

fn normalize_line_area(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

fn normalize_line_suffix(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    if value.chars().all(|ch| ch.is_ascii_digit()) {
        let number = value.parse::<u8>().ok()?;
        Some(format!("{number:02}"))
    } else {
        let normalized: String = value
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .map(|ch| ch.to_ascii_lowercase())
            .collect();
        (!normalized.is_empty()).then_some(normalized)
    }
}

fn integration_target_name(target: &fa_domain::IntegrationTarget) -> &'static str {
    match target {
        fa_domain::IntegrationTarget::Mes => "mes",
        fa_domain::IntegrationTarget::Erp => "erp",
        fa_domain::IntegrationTarget::Cmms => "cmms",
        fa_domain::IntegrationTarget::Quality => "quality",
        fa_domain::IntegrationTarget::Scada => "scada",
        fa_domain::IntegrationTarget::Warehouse => "warehouse",
        fa_domain::IntegrationTarget::Safety => "safety",
        fa_domain::IntegrationTarget::Custom(_) => "custom",
    }
}

fn alert_source_event_refs(
    task: &TaskRecord,
    evidence: &[TaskEvidence],
    source_system: Option<&str>,
) -> Vec<String> {
    let refs: Vec<String> = evidence
        .iter()
        .map(|item| item.source_ref.clone())
        .take(3)
        .collect();

    if refs.is_empty() {
        source_system
            .map(|source_system| format!("{source_system}://cluster/{}", task.id.simple()))
            .into_iter()
            .chain(std::iter::once(format!("task:{}", task.id)))
            .take(1)
            .collect()
    } else {
        refs
    }
}

fn alert_cluster_rationale(triage_label: &str) -> &'static str {
    match triage_label {
        "sustained_threshold_review" => {
            "Sustained threshold or drift signal should be clustered before maintenance-led triage and containment."
        }
        "first_response_review" => {
            "Alert-like signal should be normalized into a triage-ready cluster before first response routing."
        }
        _ => {
            "Repeated alert-like signal should be clustered before supervisor triage and owner assignment."
        }
    }
}

fn contains_any(haystack: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| haystack.contains(candidate))
}

fn severity_band(risk: TaskRisk) -> &'static str {
    match risk {
        TaskRisk::Low => "low",
        TaskRisk::Medium => "medium",
        TaskRisk::High => "high",
        TaskRisk::Critical => "critical",
    }
}

fn summarize_alert_cluster_drafts(
    alert_cluster_drafts: &[AlertClusterDraftView],
) -> AlertTriageSummary {
    if alert_cluster_drafts.is_empty() {
        return AlertTriageSummary::default();
    }

    let open_clusters = alert_cluster_drafts
        .iter()
        .filter(|cluster| cluster.cluster_status == "open")
        .count();
    let high_priority_clusters = alert_cluster_drafts
        .iter()
        .filter(|cluster| matches!(cluster.severity_band.as_str(), "high" | "critical"))
        .count();
    let escalation_candidate_count = alert_cluster_drafts
        .iter()
        .filter(|cluster| cluster.escalation_candidate)
        .count();
    let last_clustered_at = alert_cluster_drafts
        .iter()
        .map(|cluster| cluster.window_end)
        .max();

    AlertTriageSummary {
        total_clusters: alert_cluster_drafts.len(),
        open_clusters,
        high_priority_clusters,
        escalation_candidate_count,
        last_clustered_at,
    }
}

fn approval_decision_scope(request: &TaskRequest, approval_policy: ApprovalPolicy) -> Vec<String> {
    if !approval_policy.requires_human_approval() {
        return vec!["allow the task to proceed without manual approval".to_string()];
    }

    let mut scope = vec![
        "approve or reject execution before maintenance action begins".to_string(),
        "decide whether the evidence package is sufficient for the current workflow step"
            .to_string(),
    ];

    if !request.equipment_ids.is_empty() {
        scope.push(
            "confirm that equipment-affecting work stays inside policy boundaries".to_string(),
        );
    }

    if matches!(request.risk, TaskRisk::Critical) {
        scope.push(
            "escalate to plant-level governance if local approval is insufficient".to_string(),
        );
    }

    scope
}

fn approval_rationale(request: &TaskRequest, approval_policy: ApprovalPolicy) -> String {
    if !approval_policy.requires_human_approval() {
        return format!(
            "Task is {:?} risk and can proceed with automated approval under the current guardrails.",
            request.risk
        );
    }

    format!(
        "Manual approval by '{}' is required because the task is {:?} risk, {:?} priority, and may affect equipment, workflow continuity, or downstream quality decisions.",
        approval_policy.required_role(),
        request.risk,
        request.priority
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use fa_domain::{ActorHandle, IntegrationTarget};

    use super::*;
    use crate::{
        AuditStore, InMemoryAuditSink, InMemoryTaskRepository, ResubmitTaskRequest, TaskRepository,
    };

    fn safety_approver() -> ActorHandle {
        ActorHandle {
            id: "worker_2001".to_string(),
            display_name: "Wang Safety".to_string(),
            role: "Safety Officer".to_string(),
        }
    }

    fn mismatched_approver() -> ActorHandle {
        ActorHandle {
            id: "worker_2002".to_string(),
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

    fn incoming_shift_supervisor() -> ActorHandle {
        ActorHandle {
            id: "worker_1101".to_string(),
            display_name: "Zhang Incoming".to_string(),
            role: "Incoming Shift Supervisor".to_string(),
        }
    }

    fn production_supervisor() -> ActorHandle {
        ActorHandle {
            id: "worker_1001".to_string(),
            display_name: "Liu Supervisor".to_string(),
            role: "Production Supervisor".to_string(),
        }
    }

    fn monitoring_bucket_count(buckets: &[FollowUpMonitoringBucket], key: &str) -> usize {
        buckets
            .iter()
            .find(|bucket| bucket.key == key)
            .map(|bucket| bucket.count)
            .unwrap_or_default()
    }

    fn handoff_receipt_monitoring_bucket_count(
        buckets: &[HandoffReceiptMonitoringBucket],
        key: &str,
    ) -> usize {
        buckets
            .iter()
            .find(|bucket| bucket.key == key)
            .map(|bucket| bucket.count)
            .unwrap_or_default()
    }

    fn alert_cluster_monitoring_bucket_count(
        buckets: &[AlertClusterMonitoringBucket],
        key: &str,
    ) -> usize {
        buckets
            .iter()
            .find(|bucket| bucket.key == key)
            .map(|bucket| bucket.count)
            .unwrap_or_default()
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

    fn alert_triage_request() -> TaskRequest {
        let mut request = base_request();
        request.title = "Triage repeated andon alerts on pack line 4".to_string();
        request.description =
            "Review repeated alert burst and cluster similar signals before escalation."
                .to_string();
        request.priority = TaskPriority::Expedited;
        request.risk = TaskRisk::High;
        request.equipment_ids = vec!["eq_pack_04".to_string()];
        request.integrations = vec![IntegrationTarget::Mes];
        request.desired_outcome =
            "Create a triage-ready alert cluster and route it to the production supervisor."
                .to_string();
        request.requires_human_approval = false;
        request.requires_diagnostic_loop = false;
        request
    }

    fn scada_threshold_alert_request() -> TaskRequest {
        let mut request = base_request();
        request.title = "Triage sustained temperature alert on mix line 2".to_string();
        request.description =
            "Review sustained SCADA threshold breach and sensor drift on mix line 2 before escalation."
                .to_string();
        request.priority = TaskPriority::Expedited;
        request.risk = TaskRisk::Medium;
        request.equipment_ids = vec!["eq_mix_02".to_string()];
        request.integrations = vec![IntegrationTarget::Scada];
        request.desired_outcome =
            "Cluster sustained threshold signals and route first diagnostic review to maintenance."
                .to_string();
        request.requires_human_approval = false;
        request.requires_diagnostic_loop = false;
        request
    }

    fn shift_handoff_request() -> TaskRequest {
        let mut request = base_request();
        request.title = "Summarize shift notes".to_string();
        request.description = "Summarize shift notes for morning handoff.".to_string();
        request.priority = TaskPriority::Routine;
        request.risk = TaskRisk::Low;
        request.integrations = vec![IntegrationTarget::Mes];
        request.equipment_ids.clear();
        request.desired_outcome = "Publish a clean handoff summary with pending items".to_string();
        request.requires_human_approval = false;
        request.requires_diagnostic_loop = false;
        request
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
        assert!(!plan.governance.approval_strategy.manual_approval_required);
        assert_eq!(plan.governance.approval_strategy.required_role, "system");
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
        assert_eq!(
            plan.governance.approval_strategy.required_role,
            "plant_manager"
        );
        assert!(plan
            .governance
            .responsibility_matrix
            .iter()
            .any(|assignment| assignment.role == "plant_manager"
                && assignment.participation == GovernanceParticipation::Accountable));
    }

    #[test]
    fn intake_task_auto_approves_low_risk_work() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let request = shift_handoff_request();

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
        assert_eq!(intake_result.follow_up_items.len(), 1);
        assert_eq!(
            intake_result.follow_up_items[0].source_kind,
            "shift_handoff"
        );
        assert_eq!(
            intake_result.follow_up_items[0]
                .recommended_owner_role
                .as_deref(),
            Some("incoming_shift_supervisor")
        );
        assert_eq!(intake_result.follow_up_summary.total_items, 1);
        assert_eq!(intake_result.follow_up_summary.open_items, 1);
        assert!(intake_result.follow_up_summary.last_evaluated_at.is_some());
        assert_eq!(
            intake_result
                .handoff_receipt
                .as_ref()
                .map(|receipt| receipt.status.as_str()),
            Some("published")
        );
        assert_eq!(
            intake_result
                .handoff_receipt
                .as_ref()
                .expect("receipt should exist")
                .follow_up_item_ids
                .len(),
            1
        );
        assert_eq!(
            intake_result
                .handoff_receipt_summary
                .covered_follow_up_count,
            1
        );
        assert_eq!(
            intake_result
                .handoff_receipt_summary
                .unaccepted_follow_up_count,
            1
        );
        assert!(intake_result.alert_cluster_drafts.is_empty());
        assert_eq!(
            intake_result.alert_triage_summary,
            AlertTriageSummary::default()
        );
        assert!(!audit_sink
            .snapshot()
            .expect("snapshot should work")
            .is_empty());
    }

    #[test]
    fn accept_follow_up_owner_updates_shift_handoff_state() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let request = shift_handoff_request();
        let task_id = request.id;

        let intake_result = orchestrator
            .intake_task(request)
            .expect("intake should succeed");
        let follow_up_id = intake_result.follow_up_items[0].id.clone();

        let updated = orchestrator
            .accept_follow_up_owner(
                task_id,
                follow_up_id.clone(),
                AcceptFollowUpOwnerRequest {
                    actor: incoming_shift_supervisor(),
                    note: None,
                },
                Some("corr-follow-up-accept-001".to_string()),
            )
            .expect("follow-up owner acceptance should succeed");

        let follow_up_item = updated
            .follow_up_items
            .iter()
            .find(|item| item.id == follow_up_id)
            .expect("follow-up item should exist");
        assert_eq!(follow_up_item.status, "accepted");
        assert_eq!(
            follow_up_item.accepted_owner_id.as_deref(),
            Some("worker_1101")
        );
        assert_eq!(updated.follow_up_summary.total_items, 1);
        assert_eq!(updated.follow_up_summary.open_items, 1);
        assert!(updated.follow_up_summary.last_evaluated_at.is_some());
        assert_eq!(
            updated.handoff_receipt_summary.unaccepted_follow_up_count,
            0
        );
        assert_eq!(
            audit_sink
                .query(&crate::AuditEventQuery {
                    task_id: Some(task_id),
                    kind: Some(AuditEventKind::FollowUpOwnerAccepted),
                    ..crate::AuditEventQuery::default()
                })
                .expect("audit query should succeed")
                .len(),
            1
        );
    }

    #[test]
    fn accept_follow_up_owner_updates_alert_triage_state() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let request = alert_triage_request();
        let task_id = request.id;

        let intake_result = orchestrator
            .intake_task(request)
            .expect("intake should succeed");
        let follow_up_id = intake_result.follow_up_items[0].id.clone();

        let updated = orchestrator
            .accept_follow_up_owner(
                task_id,
                follow_up_id.clone(),
                AcceptFollowUpOwnerRequest {
                    actor: production_supervisor(),
                    note: Some("Production supervisor takes first response ownership.".to_string()),
                },
                Some("corr-follow-up-accept-002".to_string()),
            )
            .expect("follow-up owner acceptance should succeed");

        let follow_up_item = updated
            .follow_up_items
            .iter()
            .find(|item| item.id == follow_up_id)
            .expect("follow-up item should exist");
        assert_eq!(follow_up_item.status, "accepted");
        assert_eq!(
            follow_up_item.accepted_owner_id.as_deref(),
            Some("worker_1001")
        );
        assert_eq!(updated.follow_up_summary.total_items, 1);
        assert_eq!(updated.follow_up_summary.open_items, 1);
        assert!(updated.handoff_receipt.is_none());
        assert_eq!(updated.alert_cluster_drafts.len(), 1);
        assert_eq!(
            audit_sink
                .query(&crate::AuditEventQuery {
                    task_id: Some(task_id),
                    kind: Some(AuditEventKind::FollowUpOwnerAccepted),
                    ..crate::AuditEventQuery::default()
                })
                .expect("audit query should succeed")
                .len(),
            1
        );
    }

    #[test]
    fn accept_follow_up_owner_rejects_wrong_role() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink);
        let request = shift_handoff_request();
        let task_id = request.id;

        let intake_result = orchestrator
            .intake_task(request)
            .expect("intake should succeed");
        let follow_up_id = intake_result.follow_up_items[0].id.clone();

        let error = orchestrator
            .accept_follow_up_owner(
                task_id,
                follow_up_id.clone(),
                AcceptFollowUpOwnerRequest {
                    actor: production_supervisor(),
                    note: None,
                },
                Some("corr-follow-up-accept-003".to_string()),
            )
            .expect_err("follow-up owner acceptance should be rejected");

        assert_eq!(
            error,
            OrchestrationError::FollowUpRoleMismatch {
                follow_up_id,
                required_role: "incoming_shift_supervisor".to_string(),
                actual_role: "production_supervisor".to_string(),
            }
        );
    }

    #[test]
    fn accept_follow_up_owner_rejects_already_accepted_item() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink);
        let request = shift_handoff_request();
        let task_id = request.id;

        let intake_result = orchestrator
            .intake_task(request)
            .expect("intake should succeed");
        let follow_up_id = intake_result.follow_up_items[0].id.clone();

        orchestrator
            .accept_follow_up_owner(
                task_id,
                follow_up_id.clone(),
                AcceptFollowUpOwnerRequest {
                    actor: incoming_shift_supervisor(),
                    note: None,
                },
                Some("corr-follow-up-accept-004".to_string()),
            )
            .expect("first owner acceptance should succeed");

        let error = orchestrator
            .accept_follow_up_owner(
                task_id,
                follow_up_id.clone(),
                AcceptFollowUpOwnerRequest {
                    actor: incoming_shift_supervisor(),
                    note: None,
                },
                Some("corr-follow-up-accept-005".to_string()),
            )
            .expect_err("second owner acceptance should be rejected");

        assert_eq!(
            error,
            OrchestrationError::InvalidFollowUpItemState {
                task_id,
                follow_up_id,
                status: "accepted".to_string(),
            }
        );
    }

    #[test]
    fn acknowledge_handoff_receipt_updates_shift_handoff_state() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let request = shift_handoff_request();
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");

        let updated = orchestrator
            .acknowledge_handoff_receipt(
                task_id,
                AcknowledgeHandoffReceiptRequest {
                    actor: incoming_shift_supervisor(),
                    exception_note: None,
                },
                Some("corr-handoff-ack-001".to_string()),
            )
            .expect("handoff acknowledgement should succeed");

        assert_eq!(
            updated
                .handoff_receipt
                .as_ref()
                .map(|receipt| receipt.status.as_str()),
            Some("acknowledged")
        );
        assert_eq!(
            updated
                .handoff_receipt
                .as_ref()
                .and_then(|receipt| receipt.receiving_actor.as_ref())
                .map(|actor| actor.id.as_str()),
            Some("worker_1101")
        );
        assert!(updated
            .handoff_receipt
            .as_ref()
            .is_some_and(|receipt| receipt.acknowledged_at.is_some()));
        assert_eq!(
            updated.handoff_receipt_summary.status.as_deref(),
            Some("acknowledged")
        );
        assert!(updated.handoff_receipt_summary.acknowledged_at.is_some());
        assert_eq!(updated.handoff_receipt_summary.covered_follow_up_count, 1);
        assert_eq!(
            updated.handoff_receipt_summary.unaccepted_follow_up_count,
            1
        );
        assert_eq!(
            audit_sink
                .query(&crate::AuditEventQuery {
                    task_id: Some(task_id),
                    kind: Some(AuditEventKind::HandoffAcknowledged),
                    ..crate::AuditEventQuery::default()
                })
                .expect("audit query should succeed")
                .len(),
            1
        );
    }

    #[test]
    fn acknowledge_handoff_receipt_rejects_wrong_role() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink);
        let request = shift_handoff_request();
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");

        let error = orchestrator
            .acknowledge_handoff_receipt(
                task_id,
                AcknowledgeHandoffReceiptRequest {
                    actor: mismatched_approver(),
                    exception_note: None,
                },
                Some("corr-handoff-ack-002".to_string()),
            )
            .expect_err("handoff acknowledgement should be rejected");

        assert_eq!(
            error,
            OrchestrationError::HandoffReceiptRoleMismatch {
                required_role: "incoming_shift_supervisor".to_string(),
                actual_role: "quality_engineer".to_string(),
            }
        );
    }

    #[test]
    fn escalate_handoff_receipt_updates_exception_receipt_state() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let request = shift_handoff_request();
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");
        orchestrator
            .acknowledge_handoff_receipt(
                task_id,
                AcknowledgeHandoffReceiptRequest {
                    actor: incoming_shift_supervisor(),
                    exception_note: Some(
                        "Need clarification on packaging stop ownership before release."
                            .to_string(),
                    ),
                },
                Some("corr-handoff-ack-003".to_string()),
            )
            .expect("handoff acknowledgement with exception should succeed");

        let updated = orchestrator
            .escalate_handoff_receipt(
                task_id,
                EscalateHandoffReceiptRequest {
                    actor: production_supervisor(),
                    note: Some(
                        "Escalate to day-shift review before releasing startup decision."
                            .to_string(),
                    ),
                },
                Some("corr-handoff-escalate-001".to_string()),
            )
            .expect("handoff escalation should succeed");

        assert_eq!(
            updated
                .handoff_receipt
                .as_ref()
                .map(|receipt| receipt.status.as_str()),
            Some("escalated")
        );
        assert_eq!(
            updated
                .handoff_receipt
                .as_ref()
                .and_then(|receipt| receipt.escalation_state.as_deref()),
            Some("escalated")
        );
        assert_eq!(
            updated.handoff_receipt_summary.status.as_deref(),
            Some("escalated")
        );
        assert!(updated.handoff_receipt_summary.exception_flag);
        assert_eq!(
            audit_sink
                .query(&crate::AuditEventQuery {
                    task_id: Some(task_id),
                    kind: Some(AuditEventKind::HandoffReceiptEscalated),
                    ..crate::AuditEventQuery::default()
                })
                .expect("audit query should succeed")
                .len(),
            1
        );
    }

    #[test]
    fn escalate_handoff_receipt_rejects_non_exception_state() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink);
        let request = shift_handoff_request();
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");
        orchestrator
            .acknowledge_handoff_receipt(
                task_id,
                AcknowledgeHandoffReceiptRequest {
                    actor: incoming_shift_supervisor(),
                    exception_note: None,
                },
                Some("corr-handoff-ack-004".to_string()),
            )
            .expect("handoff acknowledgement should succeed");

        let error = orchestrator
            .escalate_handoff_receipt(
                task_id,
                EscalateHandoffReceiptRequest {
                    actor: production_supervisor(),
                    note: None,
                },
                Some("corr-handoff-escalate-002".to_string()),
            )
            .expect_err("handoff escalation should be rejected");

        assert_eq!(
            error,
            OrchestrationError::InvalidHandoffReceiptState {
                task_id,
                status: "acknowledged".to_string(),
            }
        );
    }

    #[test]
    fn escalate_handoff_receipt_rejects_wrong_role() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink);
        let request = shift_handoff_request();
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");
        orchestrator
            .acknowledge_handoff_receipt(
                task_id,
                AcknowledgeHandoffReceiptRequest {
                    actor: incoming_shift_supervisor(),
                    exception_note: Some(
                        "Need clarification on packaging stop ownership before release."
                            .to_string(),
                    ),
                },
                Some("corr-handoff-ack-005".to_string()),
            )
            .expect("handoff acknowledgement with exception should succeed");

        let error = orchestrator
            .escalate_handoff_receipt(
                task_id,
                EscalateHandoffReceiptRequest {
                    actor: incoming_shift_supervisor(),
                    note: None,
                },
                Some("corr-handoff-escalate-003".to_string()),
            )
            .expect_err("handoff escalation should be rejected");

        assert_eq!(
            error,
            OrchestrationError::HandoffReceiptRoleMismatch {
                required_role: "production_supervisor".to_string(),
                actual_role: "incoming_shift_supervisor".to_string(),
            }
        );
    }

    #[test]
    fn list_follow_up_items_returns_cross_task_items_sorted_by_due_at() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink);
        let shift_request = shift_handoff_request();
        let shift_task_id = shift_request.id;
        let alert_request = alert_triage_request();
        let alert_task_id = alert_request.id;

        orchestrator
            .intake_task(shift_request)
            .expect("shift handoff intake should succeed");
        orchestrator
            .intake_task(alert_request)
            .expect("alert triage intake should succeed");

        let items = orchestrator
            .list_follow_up_items(&FollowUpQueueQuery::default())
            .expect("follow-up queue query should succeed");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].task_id, alert_task_id);
        assert_eq!(items[0].source_kind, "alert_triage");
        assert_eq!(items[0].task_status, TaskStatus::AwaitingApproval);
        assert_eq!(items[0].effective_sla_status, "due_soon");
        assert!(!items[0].overdue);
        assert_eq!(items[1].task_id, shift_task_id);
        assert_eq!(items[1].source_kind, "shift_handoff");
        assert_eq!(items[1].task_status, TaskStatus::Approved);
        assert_eq!(items[1].effective_sla_status, "due_soon");
        assert!(!items[1].overdue);
    }

    #[test]
    fn list_follow_up_items_filters_by_owner_and_overdue_state() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator =
            WorkOrchestrator::with_m1_defaults_and_repository(audit_sink, repository.clone());
        let request = shift_handoff_request();
        let task_id = request.id;

        let intake_result = orchestrator
            .intake_task(request)
            .expect("shift handoff intake should succeed");
        let follow_up_id = intake_result.follow_up_items[0].id.clone();

        orchestrator
            .accept_follow_up_owner(
                task_id,
                follow_up_id,
                AcceptFollowUpOwnerRequest {
                    actor: incoming_shift_supervisor(),
                    note: None,
                },
                Some("corr-follow-up-queue-001".to_string()),
            )
            .expect("follow-up owner acceptance should succeed");

        let owner_items = orchestrator
            .list_follow_up_items(&FollowUpQueueQuery {
                owner_id: Some("worker_1101".to_string()),
                ..FollowUpQueueQuery::default()
            })
            .expect("owner queue query should succeed");

        assert_eq!(owner_items.len(), 1);
        assert_eq!(owner_items[0].task_id, task_id);
        assert_eq!(owner_items[0].status, "accepted");
        assert_eq!(
            owner_items[0].accepted_owner_id.as_deref(),
            Some("worker_1101")
        );

        let mut stored = repository
            .get(task_id)
            .expect("task lookup should succeed")
            .expect("task should exist");
        stored.follow_up_items[0].due_at = Some(Utc::now() - Duration::minutes(5));
        stored.follow_up_items[0].updated_at = Utc::now();
        repository
            .save(stored)
            .expect("repository save should succeed");

        let overdue_items = orchestrator
            .list_follow_up_items(&FollowUpQueueQuery {
                overdue_only: true,
                ..FollowUpQueueQuery::default()
            })
            .expect("overdue queue query should succeed");

        assert_eq!(overdue_items.len(), 1);
        assert_eq!(overdue_items[0].task_id, task_id);
        assert_eq!(overdue_items[0].effective_sla_status, "overdue");
        assert!(overdue_items[0].overdue);
    }

    #[test]
    fn list_follow_up_items_filters_by_triage_dimensions() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator =
            WorkOrchestrator::with_m1_defaults_and_repository(audit_sink, repository.clone());
        let shift_request = shift_handoff_request();
        let shift_task_id = shift_request.id;
        let alert_request = alert_triage_request();
        let alert_task_id = alert_request.id;

        orchestrator
            .intake_task(shift_request)
            .expect("shift handoff intake should succeed");
        orchestrator
            .intake_task(alert_request)
            .expect("alert triage intake should succeed");

        let now = Utc::now();

        let mut shift_state = repository
            .get(shift_task_id)
            .expect("shift task lookup should succeed")
            .expect("shift task should exist");
        shift_state.follow_up_items[0].status = "blocked".to_string();
        shift_state.follow_up_items[0].blocked_reason =
            Some("Awaiting outgoing shift clarification.".to_string());
        shift_state.follow_up_items[0].due_at = Some(now + Duration::minutes(25));
        shift_state.follow_up_items[0].updated_at = now + Duration::minutes(1);
        repository
            .save(shift_state)
            .expect("shift task save should succeed");

        let mut alert_state = repository
            .get(alert_task_id)
            .expect("alert task lookup should succeed")
            .expect("alert task should exist");
        alert_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        alert_state.follow_up_items[0].due_at = Some(now + Duration::minutes(10));
        alert_state.follow_up_items[0].updated_at = now + Duration::minutes(2);
        repository
            .save(alert_state)
            .expect("alert task save should succeed");

        let blocked_items = orchestrator
            .list_follow_up_items(&FollowUpQueueQuery {
                blocked_only: true,
                ..FollowUpQueueQuery::default()
            })
            .expect("blocked queue query should succeed");

        assert_eq!(blocked_items.len(), 1);
        assert_eq!(blocked_items[0].task_id, shift_task_id);
        assert_eq!(blocked_items[0].status, "blocked");
        assert!(blocked_items[0].blocked_reason.is_some());

        let escalated_items = orchestrator
            .list_follow_up_items(&FollowUpQueueQuery {
                escalation_required: true,
                ..FollowUpQueueQuery::default()
            })
            .expect("escalation queue query should succeed");

        assert_eq!(escalated_items.len(), 1);
        assert_eq!(escalated_items[0].task_id, alert_task_id);
        assert_eq!(
            escalated_items[0].effective_sla_status,
            "escalation_required"
        );
        assert!(escalated_items[0].overdue);

        let priority_and_risk_items = orchestrator
            .list_follow_up_items(&FollowUpQueueQuery {
                task_risk: Some(TaskRisk::High),
                task_priority: Some(TaskPriority::Expedited),
                ..FollowUpQueueQuery::default()
            })
            .expect("priority and risk queue query should succeed");

        assert_eq!(priority_and_risk_items.len(), 1);
        assert_eq!(priority_and_risk_items[0].task_id, alert_task_id);

        let due_before_items = orchestrator
            .list_follow_up_items(&FollowUpQueueQuery {
                due_before: Some(now + Duration::minutes(15)),
                ..FollowUpQueueQuery::default()
            })
            .expect("due_before queue query should succeed");

        assert_eq!(due_before_items.len(), 1);
        assert_eq!(due_before_items[0].task_id, alert_task_id);
    }

    #[test]
    fn get_follow_up_monitoring_summarizes_filtered_queue_items() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator =
            WorkOrchestrator::with_m1_defaults_and_repository(audit_sink, repository.clone());
        let shift_request = shift_handoff_request();
        let shift_task_id = shift_request.id;
        let alert_request = alert_triage_request();
        let alert_task_id = alert_request.id;

        let intake_result = orchestrator
            .intake_task(shift_request)
            .expect("shift handoff intake should succeed");
        let follow_up_id = intake_result.follow_up_items[0].id.clone();
        orchestrator
            .intake_task(alert_request)
            .expect("alert triage intake should succeed");

        orchestrator
            .accept_follow_up_owner(
                shift_task_id,
                follow_up_id,
                AcceptFollowUpOwnerRequest {
                    actor: incoming_shift_supervisor(),
                    note: None,
                },
                Some("corr-follow-up-monitoring-001".to_string()),
            )
            .expect("follow-up owner acceptance should succeed");

        let now = Utc::now();

        let mut shift_state = repository
            .get(shift_task_id)
            .expect("shift task lookup should succeed")
            .expect("shift task should exist");
        shift_state.follow_up_items[0].status = "blocked".to_string();
        shift_state.follow_up_items[0].blocked_reason =
            Some("Awaiting outgoing shift clarification.".to_string());
        shift_state.follow_up_items[0].due_at = Some(now + Duration::minutes(25));
        shift_state.follow_up_items[0].updated_at = now + Duration::minutes(1);
        repository
            .save(shift_state)
            .expect("shift task save should succeed");

        let mut alert_state = repository
            .get(alert_task_id)
            .expect("alert task lookup should succeed")
            .expect("alert task should exist");
        alert_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        alert_state.follow_up_items[0].due_at = Some(now + Duration::minutes(10));
        alert_state.follow_up_items[0].updated_at = now + Duration::minutes(2);
        repository
            .save(alert_state)
            .expect("alert task save should succeed");

        let monitoring = orchestrator
            .get_follow_up_monitoring(&FollowUpQueueQuery::default())
            .expect("follow-up monitoring query should succeed");

        assert_eq!(monitoring.total_items, 2);
        assert_eq!(monitoring.open_items, 2);
        assert_eq!(monitoring.accepted_items, 1);
        assert_eq!(monitoring.unaccepted_items, 1);
        assert_eq!(monitoring.blocked_items, 1);
        assert_eq!(monitoring.overdue_items, 1);
        assert_eq!(monitoring.escalation_required_items, 1);
        assert_eq!(monitoring.next_due_at, Some(now + Duration::minutes(10)));
        assert_eq!(
            monitoring_bucket_count(&monitoring.source_kind_counts, "alert_triage"),
            1
        );
        assert_eq!(
            monitoring_bucket_count(&monitoring.source_kind_counts, "shift_handoff"),
            1
        );
        assert_eq!(
            monitoring_bucket_count(&monitoring.owner_role_counts, "incoming_shift_supervisor"),
            1
        );
        assert_eq!(
            monitoring_bucket_count(&monitoring.owner_role_counts, "production_supervisor"),
            1
        );
        assert_eq!(
            monitoring_bucket_count(&monitoring.sla_status_counts, "due_soon"),
            1
        );
        assert_eq!(
            monitoring_bucket_count(&monitoring.sla_status_counts, "escalation_required"),
            1
        );
        assert_eq!(
            monitoring_bucket_count(&monitoring.task_risk_counts, "low"),
            1
        );
        assert_eq!(
            monitoring_bucket_count(&monitoring.task_risk_counts, "high"),
            1
        );
        assert_eq!(
            monitoring_bucket_count(&monitoring.task_priority_counts, "routine"),
            1
        );
        assert_eq!(
            monitoring_bucket_count(&monitoring.task_priority_counts, "expedited"),
            1
        );
        assert!(monitoring.last_evaluated_at >= now);

        let filtered_monitoring = orchestrator
            .get_follow_up_monitoring(&FollowUpQueueQuery {
                source_kind: Some("alert_triage".to_string()),
                ..FollowUpQueueQuery::default()
            })
            .expect("filtered follow-up monitoring query should succeed");

        assert_eq!(filtered_monitoring.total_items, 1);
        assert_eq!(filtered_monitoring.open_items, 1);
        assert_eq!(filtered_monitoring.accepted_items, 0);
        assert_eq!(filtered_monitoring.unaccepted_items, 1);
        assert_eq!(filtered_monitoring.blocked_items, 0);
        assert_eq!(filtered_monitoring.overdue_items, 1);
        assert_eq!(filtered_monitoring.escalation_required_items, 1);
        assert_eq!(
            filtered_monitoring.next_due_at,
            Some(now + Duration::minutes(10))
        );
        assert_eq!(
            monitoring_bucket_count(&filtered_monitoring.source_kind_counts, "alert_triage"),
            1
        );
    }

    #[test]
    fn list_handoff_receipts_returns_cross_shift_items_sorted_by_urgency() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator =
            WorkOrchestrator::with_m1_defaults_and_repository(audit_sink, repository.clone());
        let first_request = shift_handoff_request();
        let first_task_id = first_request.id;
        let second_request = shift_handoff_request();
        let second_task_id = second_request.id;

        orchestrator
            .intake_task(first_request)
            .expect("first handoff intake should succeed");
        orchestrator
            .intake_task(second_request)
            .expect("second handoff intake should succeed");

        let now = Utc::now();

        let mut first_state = repository
            .get(first_task_id)
            .expect("first task lookup should succeed")
            .expect("first task should exist");
        first_state
            .handoff_receipt
            .as_mut()
            .expect("handoff receipt should exist")
            .required_ack_by = Some(now - Duration::minutes(5));
        repository
            .save(first_state)
            .expect("expired receipt state should save");

        orchestrator
            .acknowledge_handoff_receipt(
                second_task_id,
                AcknowledgeHandoffReceiptRequest {
                    actor: incoming_shift_supervisor(),
                    exception_note: Some("Need clarification on one unresolved stop.".to_string()),
                },
                Some("corr-handoff-receipt-queue-001".to_string()),
            )
            .expect("receipt acknowledgement should succeed");

        let items = orchestrator
            .list_handoff_receipts(&HandoffReceiptQueueQuery::default())
            .expect("handoff receipt queue query should succeed");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].task_id, first_task_id);
        assert_eq!(items[0].effective_status, "expired");
        assert!(items[0].overdue);
        assert_eq!(items[1].task_id, second_task_id);
        assert_eq!(items[1].effective_status, "acknowledged_with_exceptions");
        assert!(items[1].has_exceptions);
    }

    #[test]
    fn list_handoff_receipts_filters_by_queue_dimensions() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator =
            WorkOrchestrator::with_m1_defaults_and_repository(audit_sink, repository.clone());
        let first_request = shift_handoff_request();
        let first_task_id = first_request.id;
        let second_request = shift_handoff_request();
        let second_task_id = second_request.id;

        orchestrator
            .intake_task(first_request)
            .expect("first handoff intake should succeed");
        orchestrator
            .intake_task(second_request)
            .expect("second handoff intake should succeed");

        let now = Utc::now();

        let mut first_state = repository
            .get(first_task_id)
            .expect("first task lookup should succeed")
            .expect("first task should exist");
        let first_shift_id = first_state
            .handoff_receipt
            .as_ref()
            .expect("handoff receipt should exist")
            .shift_id
            .clone();
        first_state
            .handoff_receipt
            .as_mut()
            .expect("handoff receipt should exist")
            .required_ack_by = Some(now - Duration::minutes(5));
        repository
            .save(first_state)
            .expect("expired receipt state should save");

        orchestrator
            .acknowledge_handoff_receipt(
                second_task_id,
                AcknowledgeHandoffReceiptRequest {
                    actor: incoming_shift_supervisor(),
                    exception_note: Some("Need clarification on one unresolved stop.".to_string()),
                },
                Some("corr-handoff-receipt-queue-002".to_string()),
            )
            .expect("receipt acknowledgement should succeed");
        orchestrator
            .escalate_handoff_receipt(
                second_task_id,
                EscalateHandoffReceiptRequest {
                    actor: production_supervisor(),
                    note: Some("Escalate to day shift review.".to_string()),
                },
                Some("corr-handoff-receipt-queue-003".to_string()),
            )
            .expect("receipt escalation should succeed");

        let overdue_items = orchestrator
            .list_handoff_receipts(&HandoffReceiptQueueQuery {
                overdue_only: true,
                ..HandoffReceiptQueueQuery::default()
            })
            .expect("overdue receipt queue query should succeed");
        assert_eq!(overdue_items.len(), 1);
        assert_eq!(overdue_items[0].task_id, first_task_id);
        assert_eq!(overdue_items[0].effective_status, "expired");

        let escalated_items = orchestrator
            .list_handoff_receipts(&HandoffReceiptQueueQuery {
                escalated_only: true,
                ..HandoffReceiptQueueQuery::default()
            })
            .expect("escalated receipt queue query should succeed");
        assert_eq!(escalated_items.len(), 1);
        assert_eq!(escalated_items[0].task_id, second_task_id);
        assert_eq!(escalated_items[0].effective_status, "escalated");
        assert!(escalated_items[0].has_exceptions);

        let exceptions_items = orchestrator
            .list_handoff_receipts(&HandoffReceiptQueueQuery {
                has_exceptions: true,
                ..HandoffReceiptQueueQuery::default()
            })
            .expect("exception receipt queue query should succeed");
        assert_eq!(exceptions_items.len(), 1);
        assert_eq!(exceptions_items[0].task_id, second_task_id);

        let shift_items = orchestrator
            .list_handoff_receipts(&HandoffReceiptQueueQuery {
                shift_id: Some(first_shift_id),
                ..HandoffReceiptQueueQuery::default()
            })
            .expect("shift-specific receipt queue query should succeed");
        assert_eq!(shift_items.len(), 1);
        assert_eq!(shift_items[0].task_id, first_task_id);

        let actor_items = orchestrator
            .list_handoff_receipts(&HandoffReceiptQueueQuery {
                receiving_actor_id: Some("worker_1101".to_string()),
                ..HandoffReceiptQueueQuery::default()
            })
            .expect("receiving actor queue query should succeed");
        assert_eq!(actor_items.len(), 1);
        assert_eq!(actor_items[0].task_id, second_task_id);

        let expired_status_items = orchestrator
            .list_handoff_receipts(&HandoffReceiptQueueQuery {
                receipt_status: Some("expired".to_string()),
                ..HandoffReceiptQueueQuery::default()
            })
            .expect("expired status queue query should succeed");
        assert_eq!(expired_status_items.len(), 1);
        assert_eq!(expired_status_items[0].task_id, first_task_id);
    }

    #[test]
    fn list_alert_clusters_returns_cross_task_items_sorted_by_escalation_and_severity() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink);
        let andon_request = alert_triage_request();
        let andon_task_id = andon_request.id;
        let scada_request = scada_threshold_alert_request();
        let scada_task_id = scada_request.id;

        orchestrator
            .intake_task(andon_request)
            .expect("andon alert intake should succeed");
        orchestrator
            .intake_task(scada_request)
            .expect("scada alert intake should succeed");

        let items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery::default())
            .expect("alert cluster queue query should succeed");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].task_id, andon_task_id);
        assert_eq!(items[0].source_system.as_deref(), Some("andon"));
        assert_eq!(items[0].severity_band, "high");
        assert!(items[0].escalation_candidate);
        assert_eq!(items[1].task_id, scada_task_id);
        assert_eq!(items[1].source_system.as_deref(), Some("scada"));
        assert_eq!(items[1].severity_band, "medium");
        assert!(!items[1].escalation_candidate);
    }

    #[test]
    fn list_alert_clusters_filters_by_queue_dimensions() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator =
            WorkOrchestrator::with_m1_defaults_and_repository(audit_sink, repository.clone());
        let andon_request = alert_triage_request();
        let andon_task_id = andon_request.id;
        let scada_request = scada_threshold_alert_request();
        let scada_task_id = scada_request.id;

        orchestrator
            .intake_task(andon_request)
            .expect("andon alert intake should succeed");
        orchestrator
            .intake_task(scada_request)
            .expect("scada alert intake should succeed");

        let now = Utc::now();

        let mut andon_state = repository
            .get(andon_task_id)
            .expect("andon task lookup should succeed")
            .expect("andon task should exist");
        andon_state.alert_cluster_drafts[0].window_start = now - Duration::minutes(30);
        andon_state.alert_cluster_drafts[0].window_end = now - Duration::minutes(25);
        andon_state.alert_cluster_drafts[0].updated_at = now - Duration::minutes(20);
        repository
            .save(andon_state)
            .expect("andon task save should succeed");

        let mut scada_state = repository
            .get(scada_task_id)
            .expect("scada task lookup should succeed")
            .expect("scada task should exist");
        scada_state.alert_cluster_drafts[0].cluster_status = "closed".to_string();
        scada_state.alert_cluster_drafts[0].window_start = now - Duration::minutes(10);
        scada_state.alert_cluster_drafts[0].window_end = now - Duration::minutes(5);
        scada_state.alert_cluster_drafts[0].updated_at = now - Duration::minutes(2);
        repository
            .save(scada_state)
            .expect("scada task save should succeed");

        let task_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                task_id: Some(scada_task_id),
                ..AlertClusterQueueQuery::default()
            })
            .expect("task-scoped alert cluster queue query should succeed");
        assert_eq!(task_items.len(), 1);
        assert_eq!(task_items[0].task_id, scada_task_id);

        let status_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                cluster_status: Some("closed".to_string()),
                ..AlertClusterQueueQuery::default()
            })
            .expect("status-filtered alert cluster queue query should succeed");
        assert_eq!(status_items.len(), 1);
        assert_eq!(status_items[0].task_id, scada_task_id);

        let source_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                source_system: Some("andon".to_string()),
                ..AlertClusterQueueQuery::default()
            })
            .expect("source-filtered alert cluster queue query should succeed");
        assert_eq!(source_items.len(), 1);
        assert_eq!(source_items[0].task_id, andon_task_id);

        let equipment_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                equipment_id: Some("eq_pack_04".to_string()),
                ..AlertClusterQueueQuery::default()
            })
            .expect("equipment-filtered alert cluster queue query should succeed");
        assert_eq!(equipment_items.len(), 1);
        assert_eq!(equipment_items[0].task_id, andon_task_id);

        let line_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                line_id: Some("line_mix_02".to_string()),
                ..AlertClusterQueueQuery::default()
            })
            .expect("line-filtered alert cluster queue query should succeed");
        assert_eq!(line_items.len(), 1);
        assert_eq!(line_items[0].task_id, scada_task_id);

        let severity_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                severity_band: Some("high".to_string()),
                ..AlertClusterQueueQuery::default()
            })
            .expect("severity-filtered alert cluster queue query should succeed");
        assert_eq!(severity_items.len(), 1);
        assert_eq!(severity_items[0].task_id, andon_task_id);

        let label_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                triage_label: Some("sustained_threshold_review".to_string()),
                ..AlertClusterQueueQuery::default()
            })
            .expect("label-filtered alert cluster queue query should succeed");
        assert_eq!(label_items.len(), 1);
        assert_eq!(label_items[0].task_id, scada_task_id);

        let owner_role_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                recommended_owner_role: Some("maintenance_engineer".to_string()),
                ..AlertClusterQueueQuery::default()
            })
            .expect("owner-role-filtered alert cluster queue query should succeed");
        assert_eq!(owner_role_items.len(), 1);
        assert_eq!(owner_role_items[0].task_id, scada_task_id);

        let escalation_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                escalation_candidate: true,
                ..AlertClusterQueueQuery::default()
            })
            .expect("escalation-filtered alert cluster queue query should succeed");
        assert_eq!(escalation_items.len(), 1);
        assert_eq!(escalation_items[0].task_id, andon_task_id);

        let window_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                window_from: Some(now - Duration::minutes(12)),
                window_to: Some(now - Duration::minutes(2)),
                ..AlertClusterQueueQuery::default()
            })
            .expect("window-filtered alert cluster queue query should succeed");
        assert_eq!(window_items.len(), 1);
        assert_eq!(window_items[0].task_id, scada_task_id);

        let open_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                open_only: true,
                ..AlertClusterQueueQuery::default()
            })
            .expect("open-only alert cluster queue query should succeed");
        assert_eq!(open_items.len(), 1);
        assert_eq!(open_items[0].task_id, andon_task_id);
    }

    #[test]
    fn list_alert_clusters_includes_linked_follow_up_summary() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator =
            WorkOrchestrator::with_m1_defaults_and_repository(audit_sink, repository.clone());
        let request = alert_triage_request();
        let task_id = request.id;

        let intake_result = orchestrator
            .intake_task(request)
            .expect("alert triage intake should succeed");
        let follow_up_id = intake_result.follow_up_items[0].id.clone();
        let cluster_id = intake_result.alert_cluster_drafts[0].cluster_id.clone();

        let initial_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery::default())
            .expect("initial alert cluster queue query should succeed");
        assert_eq!(initial_items.len(), 1);
        let initial_linked_follow_up = initial_items[0]
            .linked_follow_up
            .as_ref()
            .expect("linked follow-up summary should exist");
        assert_eq!(initial_linked_follow_up.total_items, 1);
        assert_eq!(initial_linked_follow_up.open_items, 1);
        assert_eq!(initial_linked_follow_up.accepted_items, 0);
        assert_eq!(initial_linked_follow_up.unaccepted_items, 1);
        assert_eq!(
            initial_linked_follow_up.follow_up_ids,
            vec![follow_up_id.clone()]
        );
        assert!(initial_linked_follow_up.accepted_owner_ids.is_empty());
        assert_eq!(
            initial_linked_follow_up
                .worst_effective_sla_status
                .as_deref(),
            Some("due_soon")
        );

        orchestrator
            .accept_follow_up_owner(
                task_id,
                follow_up_id,
                AcceptFollowUpOwnerRequest {
                    actor: production_supervisor(),
                    note: Some("Production supervisor takes first response ownership.".to_string()),
                },
                Some("corr-alert-cluster-follow-up-link-001".to_string()),
            )
            .expect("follow-up owner acceptance should succeed");

        let mut stored = repository
            .get(task_id)
            .expect("task lookup should succeed")
            .expect("task should exist");
        stored.follow_up_items[0].source_kind = "alert_cluster".to_string();
        stored.follow_up_items[0].source_refs = vec![cluster_id];
        stored.follow_up_items[0].sla_status = "escalation_required".to_string();
        stored.follow_up_items[0].updated_at = Utc::now();
        repository.save(stored).expect("task save should succeed");

        let linked_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery::default())
            .expect("linked alert cluster queue query should succeed");
        assert_eq!(linked_items.len(), 1);
        let linked_follow_up = linked_items[0]
            .linked_follow_up
            .as_ref()
            .expect("linked follow-up summary should exist");
        assert_eq!(linked_follow_up.total_items, 1);
        assert_eq!(linked_follow_up.open_items, 1);
        assert_eq!(linked_follow_up.accepted_items, 1);
        assert_eq!(linked_follow_up.unaccepted_items, 0);
        assert_eq!(
            linked_follow_up.accepted_owner_ids,
            vec!["worker_1001".to_string()]
        );
        assert_eq!(
            linked_follow_up.worst_effective_sla_status.as_deref(),
            Some("escalation_required")
        );
    }

    #[test]
    fn list_alert_clusters_filters_by_linked_follow_up_dimensions() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator =
            WorkOrchestrator::with_m1_defaults_and_repository(audit_sink, repository.clone());
        let andon_request = alert_triage_request();
        let andon_task_id = andon_request.id;
        let scada_request = scada_threshold_alert_request();
        let scada_task_id = scada_request.id;

        let andon_intake = orchestrator
            .intake_task(andon_request)
            .expect("andon alert intake should succeed");
        let andon_follow_up_id = andon_intake.follow_up_items[0].id.clone();
        let andon_cluster_id = andon_intake.alert_cluster_drafts[0].cluster_id.clone();
        orchestrator
            .intake_task(scada_request)
            .expect("scada alert intake should succeed");

        orchestrator
            .accept_follow_up_owner(
                andon_task_id,
                andon_follow_up_id,
                AcceptFollowUpOwnerRequest {
                    actor: production_supervisor(),
                    note: Some("Production supervisor takes first response ownership.".to_string()),
                },
                Some("corr-alert-cluster-link-filters-001".to_string()),
            )
            .expect("follow-up owner acceptance should succeed");

        let mut andon_state = repository
            .get(andon_task_id)
            .expect("andon task lookup should succeed")
            .expect("andon task should exist");
        andon_state.follow_up_items[0].source_kind = "alert_cluster".to_string();
        andon_state.follow_up_items[0].source_refs = vec![andon_cluster_id];
        andon_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        andon_state.follow_up_items[0].updated_at = Utc::now();
        repository
            .save(andon_state)
            .expect("andon task save should succeed");

        let owner_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                follow_up_owner_id: Some("worker_1001".to_string()),
                ..AlertClusterQueueQuery::default()
            })
            .expect("owner-filtered alert cluster queue query should succeed");
        assert_eq!(owner_items.len(), 1);
        assert_eq!(owner_items[0].task_id, andon_task_id);

        let unaccepted_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                unaccepted_follow_up_only: true,
                ..AlertClusterQueueQuery::default()
            })
            .expect("unaccepted alert cluster queue query should succeed");
        assert_eq!(unaccepted_items.len(), 1);
        assert_eq!(unaccepted_items[0].task_id, scada_task_id);

        let escalation_items = orchestrator
            .list_alert_clusters(&AlertClusterQueueQuery {
                follow_up_escalation_required: true,
                ..AlertClusterQueueQuery::default()
            })
            .expect("follow-up escalation alert cluster queue query should succeed");
        assert_eq!(escalation_items.len(), 1);
        assert_eq!(escalation_items[0].task_id, andon_task_id);
    }

    #[test]
    fn get_alert_cluster_monitoring_summarizes_filtered_cluster_backlog() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator =
            WorkOrchestrator::with_m1_defaults_and_repository(audit_sink, repository.clone());
        let andon_request = alert_triage_request();
        let andon_task_id = andon_request.id;
        let scada_request = scada_threshold_alert_request();
        let scada_task_id = scada_request.id;

        let mut future_request = scada_threshold_alert_request();
        future_request.id = Uuid::new_v4();
        future_request.title = "Triage reserve threshold alert on mix line 3".to_string();
        future_request.description =
            "Review reserve SCADA threshold breach on mix line 3 for the next planned batch."
                .to_string();
        future_request.equipment_ids = vec!["eq_mix_03".to_string()];
        let future_task_id = future_request.id;

        let andon_intake = orchestrator
            .intake_task(andon_request)
            .expect("andon alert intake should succeed");
        let andon_follow_up_id = andon_intake.follow_up_items[0].id.clone();
        let andon_cluster_id = andon_intake.alert_cluster_drafts[0].cluster_id.clone();
        orchestrator
            .intake_task(scada_request)
            .expect("scada alert intake should succeed");
        orchestrator
            .intake_task(future_request)
            .expect("future scada alert intake should succeed");

        orchestrator
            .accept_follow_up_owner(
                andon_task_id,
                andon_follow_up_id,
                AcceptFollowUpOwnerRequest {
                    actor: production_supervisor(),
                    note: Some("Production supervisor takes first response ownership.".to_string()),
                },
                Some("corr-alert-cluster-monitoring-linked-follow-up-001".to_string()),
            )
            .expect("follow-up owner acceptance should succeed");

        let now = Utc::now();
        let expected_next_window_end = now - Duration::minutes(25);

        let mut andon_state = repository
            .get(andon_task_id)
            .expect("andon task lookup should succeed")
            .expect("andon task should exist");
        andon_state.alert_cluster_drafts[0].window_start = now - Duration::minutes(30);
        andon_state.alert_cluster_drafts[0].window_end = expected_next_window_end;
        andon_state.follow_up_items[0].source_kind = "alert_cluster".to_string();
        andon_state.follow_up_items[0].source_refs = vec![andon_cluster_id];
        andon_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        repository
            .save(andon_state)
            .expect("andon task save should succeed");

        let mut scada_state = repository
            .get(scada_task_id)
            .expect("scada task lookup should succeed")
            .expect("scada task should exist");
        scada_state.alert_cluster_drafts[0].window_start = now - Duration::minutes(5);
        scada_state.alert_cluster_drafts[0].window_end = now + Duration::minutes(10);
        repository
            .save(scada_state)
            .expect("scada task save should succeed");

        let mut future_state = repository
            .get(future_task_id)
            .expect("future task lookup should succeed")
            .expect("future task should exist");
        future_state.alert_cluster_drafts[0].cluster_status = "closed".to_string();
        future_state.alert_cluster_drafts[0].window_start = now + Duration::minutes(20);
        future_state.alert_cluster_drafts[0].window_end = now + Duration::minutes(35);
        repository
            .save(future_state)
            .expect("future task save should succeed");

        let monitoring = orchestrator
            .get_alert_cluster_monitoring(&AlertClusterQueueQuery::default())
            .expect("alert cluster monitoring query should succeed");

        assert_eq!(monitoring.total_clusters, 3);
        assert_eq!(monitoring.open_clusters, 2);
        assert_eq!(monitoring.escalation_candidate_clusters, 1);
        assert_eq!(monitoring.high_severity_clusters, 1);
        assert_eq!(monitoring.active_window_clusters, 1);
        assert_eq!(monitoring.stale_window_clusters, 1);
        assert_eq!(monitoring.linked_follow_up_clusters, 3);
        assert_eq!(monitoring.unlinked_follow_up_clusters, 0);
        assert_eq!(monitoring.accepted_follow_up_clusters, 1);
        assert_eq!(monitoring.unaccepted_follow_up_clusters, 2);
        assert_eq!(monitoring.follow_up_escalation_clusters, 1);
        assert_eq!(
            monitoring.next_window_end_at,
            Some(expected_next_window_end)
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.cluster_status_counts, "open"),
            2
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.cluster_status_counts, "closed"),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.source_system_counts, "andon"),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.source_system_counts, "scada"),
            2
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.severity_band_counts, "high"),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.severity_band_counts, "medium"),
            2
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &monitoring.triage_label_counts,
                "repeated_alert_review"
            ),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &monitoring.triage_label_counts,
                "sustained_threshold_review"
            ),
            2
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &monitoring.owner_role_counts,
                "production_supervisor"
            ),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &monitoring.owner_role_counts,
                "maintenance_engineer"
            ),
            2
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.window_state_counts, "stale"),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.window_state_counts, "active"),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.window_state_counts, "future"),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.follow_up_coverage_counts, "linked"),
            3
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &monitoring.follow_up_sla_status_counts,
                "escalation_required"
            ),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &monitoring.follow_up_sla_status_counts,
                "due_soon"
            ),
            2
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.task_risk_counts, "high"),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.task_risk_counts, "medium"),
            2
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(&monitoring.task_priority_counts, "expedited"),
            3
        );
        assert!(monitoring.last_evaluated_at >= now);

        let filtered_monitoring = orchestrator
            .get_alert_cluster_monitoring(&AlertClusterQueueQuery {
                source_system: Some("scada".to_string()),
                ..AlertClusterQueueQuery::default()
            })
            .expect("filtered alert cluster monitoring query should succeed");

        assert_eq!(filtered_monitoring.total_clusters, 2);
        assert_eq!(filtered_monitoring.open_clusters, 1);
        assert_eq!(filtered_monitoring.escalation_candidate_clusters, 0);
        assert_eq!(filtered_monitoring.high_severity_clusters, 0);
        assert_eq!(filtered_monitoring.active_window_clusters, 1);
        assert_eq!(filtered_monitoring.stale_window_clusters, 0);
        assert_eq!(filtered_monitoring.linked_follow_up_clusters, 2);
        assert_eq!(filtered_monitoring.unlinked_follow_up_clusters, 0);
        assert_eq!(filtered_monitoring.accepted_follow_up_clusters, 0);
        assert_eq!(filtered_monitoring.unaccepted_follow_up_clusters, 2);
        assert_eq!(filtered_monitoring.follow_up_escalation_clusters, 0);
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &filtered_monitoring.cluster_status_counts,
                "closed"
            ),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &filtered_monitoring.cluster_status_counts,
                "open"
            ),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &filtered_monitoring.source_system_counts,
                "scada"
            ),
            2
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &filtered_monitoring.window_state_counts,
                "active"
            ),
            1
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &filtered_monitoring.follow_up_coverage_counts,
                "linked"
            ),
            2
        );
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &filtered_monitoring.follow_up_sla_status_counts,
                "due_soon"
            ),
            2
        );
    }

    #[test]
    fn get_alert_cluster_monitoring_filters_by_linked_follow_up_dimensions() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator =
            WorkOrchestrator::with_m1_defaults_and_repository(audit_sink, repository.clone());
        let andon_request = alert_triage_request();
        let andon_task_id = andon_request.id;
        let scada_request = scada_threshold_alert_request();

        let andon_intake = orchestrator
            .intake_task(andon_request)
            .expect("andon alert intake should succeed");
        let andon_follow_up_id = andon_intake.follow_up_items[0].id.clone();
        let andon_cluster_id = andon_intake.alert_cluster_drafts[0].cluster_id.clone();
        orchestrator
            .intake_task(scada_request)
            .expect("scada alert intake should succeed");

        orchestrator
            .accept_follow_up_owner(
                andon_task_id,
                andon_follow_up_id,
                AcceptFollowUpOwnerRequest {
                    actor: production_supervisor(),
                    note: None,
                },
                Some("corr-alert-cluster-link-filters-002".to_string()),
            )
            .expect("follow-up owner acceptance should succeed");

        let mut andon_state = repository
            .get(andon_task_id)
            .expect("andon task lookup should succeed")
            .expect("andon task should exist");
        andon_state.follow_up_items[0].source_kind = "alert_cluster".to_string();
        andon_state.follow_up_items[0].source_refs = vec![andon_cluster_id];
        andon_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        repository
            .save(andon_state)
            .expect("andon task save should succeed");

        let escalation_monitoring = orchestrator
            .get_alert_cluster_monitoring(&AlertClusterQueueQuery {
                follow_up_escalation_required: true,
                ..AlertClusterQueueQuery::default()
            })
            .expect("follow-up escalation alert cluster monitoring query should succeed");
        assert_eq!(escalation_monitoring.total_clusters, 1);
        assert_eq!(escalation_monitoring.open_clusters, 1);
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &escalation_monitoring.source_system_counts,
                "andon"
            ),
            1
        );

        let unaccepted_monitoring = orchestrator
            .get_alert_cluster_monitoring(&AlertClusterQueueQuery {
                unaccepted_follow_up_only: true,
                ..AlertClusterQueueQuery::default()
            })
            .expect("unaccepted alert cluster monitoring query should succeed");
        assert_eq!(unaccepted_monitoring.total_clusters, 1);
        assert_eq!(
            alert_cluster_monitoring_bucket_count(
                &unaccepted_monitoring.source_system_counts,
                "scada"
            ),
            1
        );
    }

    #[test]
    fn get_handoff_receipt_monitoring_summarizes_filtered_receipt_backlog() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator =
            WorkOrchestrator::with_m1_defaults_and_repository(audit_sink, repository.clone());
        let first_request = shift_handoff_request();
        let first_task_id = first_request.id;

        let mut second_request = shift_handoff_request();
        second_request.id = Uuid::new_v4();
        second_request.title = "Summarize packaging handoff notes".to_string();
        second_request.description =
            "Summarize packaging line handoff notes for the next shift.".to_string();
        let second_task_id = second_request.id;

        let mut third_request = shift_handoff_request();
        third_request.id = Uuid::new_v4();
        third_request.title = "Summarize assembly handoff notes".to_string();
        third_request.description =
            "Summarize assembly line handoff notes for the next shift.".to_string();
        let third_task_id = third_request.id;

        let mut fourth_request = shift_handoff_request();
        fourth_request.id = Uuid::new_v4();
        fourth_request.title = "Summarize paint handoff notes".to_string();
        fourth_request.description =
            "Summarize paint line handoff notes for the next shift.".to_string();
        let fourth_task_id = fourth_request.id;

        orchestrator
            .intake_task(first_request)
            .expect("first handoff intake should succeed");
        orchestrator
            .intake_task(second_request)
            .expect("second handoff intake should succeed");
        orchestrator
            .intake_task(third_request)
            .expect("third handoff intake should succeed");
        orchestrator
            .intake_task(fourth_request)
            .expect("fourth handoff intake should succeed");

        let now = Utc::now();

        let mut first_state = repository
            .get(first_task_id)
            .expect("first task lookup should succeed")
            .expect("first task should exist");
        first_state
            .handoff_receipt
            .as_mut()
            .expect("handoff receipt should exist")
            .required_ack_by = Some(now - Duration::minutes(5));
        repository
            .save(first_state)
            .expect("first task save should succeed");

        let mut second_state = repository
            .get(second_task_id)
            .expect("second task lookup should succeed")
            .expect("second task should exist");
        second_state
            .handoff_receipt
            .as_mut()
            .expect("handoff receipt should exist")
            .required_ack_by = Some(now + Duration::minutes(20));
        repository
            .save(second_state)
            .expect("second task save should succeed");

        orchestrator
            .acknowledge_handoff_receipt(
                third_task_id,
                AcknowledgeHandoffReceiptRequest {
                    actor: incoming_shift_supervisor(),
                    exception_note: Some(
                        "Need clarification on packaging stop ownership before release."
                            .to_string(),
                    ),
                },
                Some("corr-handoff-monitoring-001".to_string()),
            )
            .expect("third receipt acknowledgement should succeed");

        orchestrator
            .acknowledge_handoff_receipt(
                fourth_task_id,
                AcknowledgeHandoffReceiptRequest {
                    actor: incoming_shift_supervisor(),
                    exception_note: Some(
                        "Need clarification on paint line release notes.".to_string(),
                    ),
                },
                Some("corr-handoff-monitoring-002".to_string()),
            )
            .expect("fourth receipt acknowledgement should succeed");
        orchestrator
            .escalate_handoff_receipt(
                fourth_task_id,
                EscalateHandoffReceiptRequest {
                    actor: production_supervisor(),
                    note: Some("Escalate paint handoff to supervisor review.".to_string()),
                },
                Some("corr-handoff-monitoring-003".to_string()),
            )
            .expect("fourth receipt escalation should succeed");

        let monitoring = orchestrator
            .get_handoff_receipt_monitoring(&HandoffReceiptQueueQuery::default())
            .expect("handoff receipt monitoring query should succeed");

        assert_eq!(monitoring.total_receipts, 4);
        assert_eq!(monitoring.open_receipts, 4);
        assert_eq!(monitoring.acknowledged_receipts, 2);
        assert_eq!(monitoring.unacknowledged_receipts, 2);
        assert_eq!(monitoring.overdue_receipts, 1);
        assert_eq!(monitoring.exception_receipts, 2);
        assert_eq!(monitoring.escalated_receipts, 1);
        assert_eq!(monitoring.next_ack_due_at, Some(now - Duration::minutes(5)));
        assert_eq!(
            handoff_receipt_monitoring_bucket_count(&monitoring.effective_status_counts, "expired"),
            1
        );
        assert_eq!(
            handoff_receipt_monitoring_bucket_count(
                &monitoring.effective_status_counts,
                "published"
            ),
            1
        );
        assert_eq!(
            handoff_receipt_monitoring_bucket_count(
                &monitoring.effective_status_counts,
                "acknowledged_with_exceptions"
            ),
            1
        );
        assert_eq!(
            handoff_receipt_monitoring_bucket_count(
                &monitoring.effective_status_counts,
                "escalated"
            ),
            1
        );
        assert_eq!(
            handoff_receipt_monitoring_bucket_count(
                &monitoring.receiving_role_counts,
                "incoming_shift_supervisor"
            ),
            4
        );
        assert_eq!(
            handoff_receipt_monitoring_bucket_count(&monitoring.ack_window_counts, "overdue"),
            1
        );
        assert_eq!(
            handoff_receipt_monitoring_bucket_count(
                &monitoring.ack_window_counts,
                "due_within_30m"
            ),
            1
        );
        assert_eq!(
            handoff_receipt_monitoring_bucket_count(&monitoring.task_risk_counts, "low"),
            4
        );
        assert_eq!(
            handoff_receipt_monitoring_bucket_count(&monitoring.task_priority_counts, "routine"),
            4
        );
        assert!(monitoring.last_evaluated_at >= now);

        let filtered_monitoring = orchestrator
            .get_handoff_receipt_monitoring(&HandoffReceiptQueueQuery {
                escalated_only: true,
                ..HandoffReceiptQueueQuery::default()
            })
            .expect("filtered handoff receipt monitoring query should succeed");

        assert_eq!(filtered_monitoring.total_receipts, 1);
        assert_eq!(filtered_monitoring.open_receipts, 1);
        assert_eq!(filtered_monitoring.acknowledged_receipts, 1);
        assert_eq!(filtered_monitoring.unacknowledged_receipts, 0);
        assert_eq!(filtered_monitoring.overdue_receipts, 0);
        assert_eq!(filtered_monitoring.exception_receipts, 1);
        assert_eq!(filtered_monitoring.escalated_receipts, 1);
        assert_eq!(filtered_monitoring.next_ack_due_at, None);
        assert_eq!(
            handoff_receipt_monitoring_bucket_count(
                &filtered_monitoring.effective_status_counts,
                "escalated"
            ),
            1
        );
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
        assert!(intake_result.follow_up_items.is_empty());
        assert_eq!(intake_result.follow_up_summary.total_items, 0);
        assert!(intake_result.handoff_receipt.is_none());
        assert_eq!(
            intake_result.handoff_receipt_summary,
            HandoffReceiptSummary::default()
        );
        assert!(intake_result.alert_cluster_drafts.is_empty());
        assert_eq!(
            intake_result.alert_triage_summary,
            AlertTriageSummary::default()
        );
        assert!(audit_sink.snapshot().expect("snapshot should work").len() >= 4);
    }

    #[test]
    fn intake_task_seeds_follow_up_and_alert_cluster_for_alert_triage_work() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());

        let intake_result = orchestrator
            .intake_task(alert_triage_request())
            .expect("intake should succeed");

        assert_eq!(
            intake_result.planned_task.task.status,
            fa_domain::TaskStatus::AwaitingApproval
        );
        assert_eq!(intake_result.context_reads.len(), 1);
        assert_eq!(intake_result.evidence.len(), 2);
        assert_eq!(intake_result.follow_up_items.len(), 1);
        assert_eq!(intake_result.follow_up_items[0].source_kind, "alert_triage");
        assert_eq!(
            intake_result.follow_up_items[0]
                .recommended_owner_role
                .as_deref(),
            Some("production_supervisor")
        );
        assert_eq!(intake_result.follow_up_summary.total_items, 1);
        assert_eq!(intake_result.follow_up_summary.open_items, 1);
        assert!(intake_result.handoff_receipt.is_none());
        assert_eq!(intake_result.alert_cluster_drafts.len(), 1);
        assert_eq!(intake_result.alert_cluster_drafts[0].cluster_status, "open");
        assert_eq!(
            intake_result.alert_cluster_drafts[0]
                .source_system
                .as_deref(),
            Some("andon")
        );
        assert_eq!(
            intake_result.alert_cluster_drafts[0]
                .equipment_id
                .as_deref(),
            Some("eq_pack_04")
        );
        assert_eq!(
            intake_result.alert_cluster_drafts[0].line_id.as_deref(),
            Some("line_pack_04")
        );
        assert_eq!(intake_result.alert_cluster_drafts[0].severity_band, "high");
        assert_eq!(
            intake_result.alert_cluster_drafts[0]
                .triage_label
                .as_deref(),
            Some("repeated_alert_review")
        );
        assert_eq!(
            intake_result.alert_cluster_drafts[0].window_end
                - intake_result.alert_cluster_drafts[0].window_start,
            Duration::minutes(5)
        );
        assert_eq!(
            intake_result.alert_cluster_drafts[0]
                .recommended_owner_role
                .as_deref(),
            Some("production_supervisor")
        );
        assert!(intake_result.alert_cluster_drafts[0].escalation_candidate);
        assert_eq!(intake_result.alert_triage_summary.total_clusters, 1);
        assert_eq!(intake_result.alert_triage_summary.open_clusters, 1);
        assert_eq!(intake_result.alert_triage_summary.high_priority_clusters, 1);
        assert_eq!(
            intake_result
                .alert_triage_summary
                .escalation_candidate_count,
            1
        );
        assert!(intake_result
            .alert_triage_summary
            .last_clustered_at
            .is_some());
        assert!(audit_sink.snapshot().expect("snapshot should work").len() >= 3);
    }

    #[test]
    fn intake_task_infers_scada_threshold_alert_cluster_shape() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink);

        let intake_result = orchestrator
            .intake_task(scada_threshold_alert_request())
            .expect("intake should succeed");

        assert_eq!(
            intake_result.planned_task.task.status,
            fa_domain::TaskStatus::Approved
        );
        assert!(intake_result.context_reads.is_empty());
        assert!(intake_result.evidence.is_empty());
        assert_eq!(intake_result.follow_up_items.len(), 1);
        assert_eq!(
            intake_result.follow_up_items[0]
                .recommended_owner_role
                .as_deref(),
            Some("maintenance_engineer")
        );
        assert_eq!(intake_result.alert_cluster_drafts.len(), 1);
        assert_eq!(
            intake_result.alert_cluster_drafts[0]
                .source_system
                .as_deref(),
            Some("scada")
        );
        assert_eq!(
            intake_result.alert_cluster_drafts[0].line_id.as_deref(),
            Some("line_mix_02")
        );
        assert_eq!(
            intake_result.alert_cluster_drafts[0]
                .triage_label
                .as_deref(),
            Some("sustained_threshold_review")
        );
        assert_eq!(
            intake_result.alert_cluster_drafts[0]
                .recommended_owner_role
                .as_deref(),
            Some("maintenance_engineer")
        );
        assert_eq!(
            intake_result.alert_cluster_drafts[0].source_event_refs,
            vec![format!(
                "scada://cluster/{}",
                intake_result.planned_task.task.id.simple()
            )]
        );
        assert_eq!(
            intake_result.alert_cluster_drafts[0].window_end
                - intake_result.alert_cluster_drafts[0].window_start,
            Duration::minutes(15)
        );
        assert_eq!(
            intake_result.alert_cluster_drafts[0].severity_band,
            "medium"
        );
        assert!(!intake_result.alert_cluster_drafts[0].escalation_candidate);
        assert_eq!(intake_result.alert_triage_summary.total_clusters, 1);
        assert_eq!(intake_result.alert_triage_summary.open_clusters, 1);
        assert_eq!(intake_result.alert_triage_summary.high_priority_clusters, 0);
        assert_eq!(
            intake_result
                .alert_triage_summary
                .escalation_candidate_count,
            0
        );
    }

    #[test]
    fn tracked_task_state_decodes_without_follow_up_fields() {
        let now = Utc::now();
        let state = TrackedTaskState {
            correlation_id: "compat-001".to_string(),
            planned_task: PlannedTaskBundle {
                task: TaskRecord::draft(base_request()),
                approval: None,
            },
            context_reads: Vec::new(),
            evidence: Vec::new(),
            follow_up_items: vec![FollowUpItemView {
                id: "fu_001".to_string(),
                title: "Confirm inspection result".to_string(),
                summary: Some("Need maintenance confirmation before restart.".to_string()),
                source_kind: "anomaly".to_string(),
                source_refs: vec!["cmms:inspection-88".to_string()],
                status: "draft".to_string(),
                recommended_owner_role: Some("maintenance_supervisor".to_string()),
                accepted_owner_id: None,
                due_at: Some(now + Duration::hours(2)),
                sla_status: "due_soon".to_string(),
                blocked_reason: None,
                created_at: now,
                updated_at: now,
            }],
            follow_up_summary: FollowUpSummary {
                total_items: 1,
                open_items: 1,
                blocked_items: 0,
                overdue_items: 0,
                escalated_items: 0,
                last_evaluated_at: Some(now),
            },
            handoff_receipt: Some(HandoffReceiptView {
                id: "hr_001".to_string(),
                handoff_task_id: Uuid::new_v4(),
                shift_id: "shift_b_2026_03_12".to_string(),
                sending_actor: ActorHandle {
                    id: "worker_1001".to_string(),
                    display_name: "Liu Supervisor".to_string(),
                    role: "Production Supervisor".to_string(),
                },
                receiving_role: "incoming_shift_supervisor".to_string(),
                receiving_actor: None,
                published_at: now,
                required_ack_by: Some(now + Duration::minutes(30)),
                status: "published".to_string(),
                follow_up_item_ids: vec!["fu_001".to_string()],
                exception_note: None,
                acknowledged_at: None,
                escalation_state: Some("none".to_string()),
                created_at: now,
                updated_at: now,
            }),
            handoff_receipt_summary: HandoffReceiptSummary {
                status: Some("published".to_string()),
                published_at: Some(now),
                required_ack_by: Some(now + Duration::minutes(30)),
                acknowledged_at: None,
                covered_follow_up_count: 1,
                unaccepted_follow_up_count: 1,
                exception_flag: false,
            },
            alert_cluster_drafts: vec![AlertClusterDraftView {
                cluster_id: "ac_001".to_string(),
                cluster_status: "open".to_string(),
                source_system: Some("andon".to_string()),
                equipment_id: Some("eq_pack_04".to_string()),
                line_id: Some("line_pack_a".to_string()),
                severity_band: "high".to_string(),
                source_event_refs: vec!["andon:evt-101".to_string(), "andon:evt-102".to_string()],
                window_start: now,
                window_end: now + Duration::minutes(6),
                triage_label: Some("repeat_temperature_alarm".to_string()),
                recommended_owner_role: Some("production_supervisor".to_string()),
                escalation_candidate: true,
                rationale: Some(
                    "Repeated alarm burst within short window on same station.".to_string(),
                ),
                created_at: now,
                updated_at: now,
            }],
            alert_triage_summary: AlertTriageSummary {
                total_clusters: 1,
                open_clusters: 1,
                high_priority_clusters: 1,
                escalation_candidate_count: 1,
                last_clustered_at: Some(now + Duration::minutes(6)),
            },
        };

        let mut encoded =
            serde_json::to_value(&state).expect("tracked task state should encode to json");
        let object = encoded
            .as_object_mut()
            .expect("tracked task state should encode to object");
        object.remove("follow_up_items");
        object.remove("follow_up_summary");
        object.remove("handoff_receipt");
        object.remove("handoff_receipt_summary");
        object.remove("alert_cluster_drafts");
        object.remove("alert_triage_summary");

        let decoded: TrackedTaskState =
            serde_json::from_value(encoded).expect("legacy json should still decode");

        assert!(decoded.follow_up_items.is_empty());
        assert_eq!(decoded.follow_up_summary, FollowUpSummary::default());
        assert!(decoded.handoff_receipt.is_none());
        assert_eq!(
            decoded.handoff_receipt_summary,
            HandoffReceiptSummary::default()
        );
        assert!(decoded.alert_cluster_drafts.is_empty());
        assert_eq!(decoded.alert_triage_summary, AlertTriageSummary::default());
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
    fn get_task_governance_returns_matrix_and_strategy() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink);
        let mut request = base_request();
        request.risk = TaskRisk::High;
        request.requires_human_approval = true;
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");

        let governance = orchestrator
            .get_task_governance(task_id)
            .expect("governance lookup should succeed");

        assert_eq!(governance.approval_strategy.required_role, "safety_officer");
        assert!(governance.approval_strategy.manual_approval_required);
        assert!(governance
            .responsibility_matrix
            .iter()
            .any(|assignment| assignment.role == "maintenance_engineer"
                && assignment.participation == GovernanceParticipation::Responsible));
        assert!(governance
            .responsibility_matrix
            .iter()
            .any(|assignment| assignment.role == "quality_engineer"
                && assignment.participation == GovernanceParticipation::Consulted));
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
                    decided_by: safety_approver(),
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
                    decided_by: safety_approver(),
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
                    decided_by: safety_approver(),
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
                    decided_by: safety_approver(),
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
                    decided_by: safety_approver(),
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

    #[test]
    fn approve_task_rejects_mismatched_approval_role() {
        let audit_sink = Arc::new(InMemoryAuditSink::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults(audit_sink.clone());
        let mut request = base_request();
        request.risk = TaskRisk::High;
        request.requires_human_approval = true;
        let task_id = request.id;

        orchestrator
            .intake_task(request)
            .expect("intake should succeed");

        let error = orchestrator
            .approve_task(
                task_id,
                ApprovalActionRequest {
                    decided_by: mismatched_approver(),
                    approved: true,
                    comment: Some("Proceed to execution".to_string()),
                },
                Some("approve-mismatch-001".to_string()),
            )
            .expect_err("mismatched role should fail");

        assert_eq!(
            error,
            OrchestrationError::Lifecycle(LifecycleError::ApprovalRoleMismatch {
                required_role: "safety_officer".to_string(),
                actual_role: "quality_engineer".to_string(),
            })
        );

        let state = orchestrator
            .get_task(task_id)
            .expect("task should remain readable");
        assert_eq!(
            state.planned_task.task.status,
            fa_domain::TaskStatus::AwaitingApproval
        );
        assert_eq!(
            state
                .planned_task
                .approval
                .as_ref()
                .map(|approval| approval.status),
            Some(fa_domain::ApprovalStatus::Pending)
        );
        let audit_events = audit_sink.snapshot().expect("audit snapshot");
        assert_eq!(audit_events.len(), 6);
        assert!(audit_events
            .iter()
            .all(|event| event.correlation_id.as_deref() != Some("approve-mismatch-001")));
    }
}
