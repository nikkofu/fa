use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::workflow::{ActorHandle, ApprovalPolicy, ExecutionPlan, TaskRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Draft,
    Planned,
    AwaitingApproval,
    Approved,
    Executing,
    Completed,
    Failed,
}

impl TaskStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (TaskStatus::Draft, TaskStatus::Planned)
                | (TaskStatus::Planned, TaskStatus::AwaitingApproval)
                | (TaskStatus::Planned, TaskStatus::Approved)
                | (TaskStatus::Planned, TaskStatus::Failed)
                | (TaskStatus::AwaitingApproval, TaskStatus::Planned)
                | (TaskStatus::AwaitingApproval, TaskStatus::Approved)
                | (TaskStatus::AwaitingApproval, TaskStatus::Failed)
                | (TaskStatus::Approved, TaskStatus::Executing)
                | (TaskStatus::Approved, TaskStatus::Failed)
                | (TaskStatus::Executing, TaskStatus::Completed)
                | (TaskStatus::Executing, TaskStatus::Failed)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

impl ApprovalStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (ApprovalStatus::Pending, ApprovalStatus::Approved)
                | (ApprovalStatus::Pending, ApprovalStatus::Rejected)
                | (ApprovalStatus::Pending, ApprovalStatus::Expired)
        )
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleError {
    #[error("invalid task transition from {from:?} to {to:?}")]
    InvalidTaskTransition { from: TaskStatus, to: TaskStatus },
    #[error("invalid approval transition from {from:?} to {to:?}")]
    InvalidApprovalTransition {
        from: ApprovalStatus,
        to: ApprovalStatus,
    },
    #[error("task request id does not match execution plan request id")]
    RequestPlanMismatch,
    #[error("approval policy {0:?} does not require a human approval record")]
    ApprovalNotRequired(ApprovalPolicy),
    #[error("approval requires role '{required_role}', got '{actual_role}'")]
    ApprovalRoleMismatch {
        required_role: String,
        actual_role: String,
    },
    #[error("execution plan is required before requesting approval")]
    MissingExecutionPlan,
    #[error("execution plan is required before auto-approval")]
    MissingExecutionPlanForAutoApproval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: Uuid,
    pub request: TaskRequest,
    pub plan: Option<ExecutionPlan>,
    pub status: TaskStatus,
    pub approval_id: Option<Uuid>,
    pub latest_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TaskRecord {
    pub fn draft(request: TaskRequest) -> Self {
        let now = Utc::now();
        Self {
            id: request.id,
            request,
            plan: None,
            status: TaskStatus::Draft,
            approval_id: None,
            latest_error: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn apply_plan(&mut self, plan: ExecutionPlan) -> Result<(), LifecycleError> {
        if self.request.id != plan.request_id {
            return Err(LifecycleError::RequestPlanMismatch);
        }

        self.transition(TaskStatus::Planned)?;
        self.plan = Some(plan);
        self.latest_error = None;
        Ok(())
    }

    pub fn request_approval(&mut self, approval_id: Uuid) -> Result<(), LifecycleError> {
        let Some(plan) = &self.plan else {
            return Err(LifecycleError::MissingExecutionPlan);
        };

        if !plan.approval_policy.requires_human_approval() {
            return Err(LifecycleError::ApprovalNotRequired(plan.approval_policy));
        }

        self.transition(TaskStatus::AwaitingApproval)?;
        self.approval_id = Some(approval_id);
        Ok(())
    }

    pub fn auto_approve(&mut self) -> Result<(), LifecycleError> {
        let Some(plan) = &self.plan else {
            return Err(LifecycleError::MissingExecutionPlanForAutoApproval);
        };

        if plan.approval_policy.requires_human_approval() {
            return Err(LifecycleError::ApprovalNotRequired(plan.approval_policy));
        }

        self.transition(TaskStatus::Approved)
    }

    pub fn approve(&mut self) -> Result<(), LifecycleError> {
        self.transition(TaskStatus::Approved)
    }

    pub fn return_for_revision(&mut self) -> Result<(), LifecycleError> {
        self.transition(TaskStatus::Planned)
    }

    pub fn start_execution(&mut self) -> Result<(), LifecycleError> {
        self.transition(TaskStatus::Executing)
    }

    pub fn complete(&mut self) -> Result<(), LifecycleError> {
        self.transition(TaskStatus::Completed)
    }

    pub fn fail(&mut self, reason: impl Into<String>) -> Result<(), LifecycleError> {
        self.transition(TaskStatus::Failed)?;
        self.latest_error = Some(reason.into());
        Ok(())
    }

    fn transition(&mut self, next: TaskStatus) -> Result<(), LifecycleError> {
        let current = self.status;
        if !current.can_transition_to(next) {
            return Err(LifecycleError::InvalidTaskTransition {
                from: current,
                to: next,
            });
        }

        self.status = next;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub id: Uuid,
    pub task_id: Uuid,
    pub policy: ApprovalPolicy,
    pub required_role: String,
    pub status: ApprovalStatus,
    pub requested_by: ActorHandle,
    pub decided_by: Option<ActorHandle>,
    pub comment: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
}

impl ApprovalRecord {
    pub fn pending(
        task_id: Uuid,
        policy: ApprovalPolicy,
        requested_by: ActorHandle,
    ) -> Result<Self, LifecycleError> {
        if !policy.requires_human_approval() {
            return Err(LifecycleError::ApprovalNotRequired(policy));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            task_id,
            policy,
            required_role: policy.required_role().to_string(),
            status: ApprovalStatus::Pending,
            requested_by,
            decided_by: None,
            comment: None,
            requested_at: Utc::now(),
            decided_at: None,
        })
    }

    pub fn approve(
        &mut self,
        decided_by: ActorHandle,
        comment: impl Into<Option<String>>,
    ) -> Result<(), LifecycleError> {
        self.ensure_decider_role(&decided_by)?;
        self.transition(ApprovalStatus::Approved)?;
        self.decided_by = Some(decided_by);
        self.comment = comment.into();
        self.decided_at = Some(Utc::now());
        Ok(())
    }

    pub fn reject(
        &mut self,
        decided_by: ActorHandle,
        comment: impl Into<Option<String>>,
    ) -> Result<(), LifecycleError> {
        self.ensure_decider_role(&decided_by)?;
        self.transition(ApprovalStatus::Rejected)?;
        self.decided_by = Some(decided_by);
        self.comment = comment.into();
        self.decided_at = Some(Utc::now());
        Ok(())
    }

    pub fn expire(&mut self, comment: impl Into<Option<String>>) -> Result<(), LifecycleError> {
        self.transition(ApprovalStatus::Expired)?;
        self.comment = comment.into();
        self.decided_at = Some(Utc::now());
        Ok(())
    }

    fn transition(&mut self, next: ApprovalStatus) -> Result<(), LifecycleError> {
        let current = self.status;
        if !current.can_transition_to(next) {
            return Err(LifecycleError::InvalidApprovalTransition {
                from: current,
                to: next,
            });
        }

        self.status = next;
        Ok(())
    }

    fn ensure_decider_role(&self, decided_by: &ActorHandle) -> Result<(), LifecycleError> {
        let actual_role = decided_by.normalized_role();
        if actual_role != self.required_role {
            return Err(LifecycleError::ApprovalRoleMismatch {
                required_role: self.required_role.clone(),
                actual_role,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedTaskBundle {
    pub task: TaskRecord,
    pub approval: Option<ApprovalRecord>,
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        AgenticPattern, ApprovalPolicy, PlanOwner, PlannedStep, TaskPriority, TaskRisk,
        WorkflowGovernance,
    };

    use super::*;

    fn test_request() -> TaskRequest {
        TaskRequest {
            id: Uuid::new_v4(),
            title: "Investigate spindle drift".to_string(),
            description: "Diagnose spindle temperature drift.".to_string(),
            priority: TaskPriority::Expedited,
            risk: TaskRisk::Medium,
            initiator: ActorHandle {
                id: "worker_1001".to_string(),
                display_name: "Liu Supervisor".to_string(),
                role: "Production Supervisor".to_string(),
            },
            stakeholders: Vec::new(),
            equipment_ids: vec!["eq_cnc_01".to_string()],
            integrations: Vec::new(),
            desired_outcome: "Recover stable operation".to_string(),
            requires_human_approval: false,
            requires_diagnostic_loop: true,
        }
    }

    fn test_plan(request_id: Uuid, approval_policy: ApprovalPolicy) -> ExecutionPlan {
        ExecutionPlan {
            request_id,
            patterns: vec![AgenticPattern::Coordinator],
            rationale: vec!["test".to_string()],
            approval_policy,
            governance: WorkflowGovernance::default(),
            steps: vec![PlannedStep {
                sequence: 1,
                label: "Test".to_string(),
                owner: PlanOwner::System("workflow-engine".to_string()),
                expected_output: "ok".to_string(),
            }],
            created_at: Utc::now(),
        }
    }

    #[test]
    fn task_record_allows_happy_path_with_manual_approval() {
        let request = test_request();
        let request_id = request.id;
        let requested_by = request.initiator.clone();
        let mut task = TaskRecord::draft(request);
        let plan = test_plan(request_id, ApprovalPolicy::SafetyOfficer);
        let mut approval =
            ApprovalRecord::pending(request_id, ApprovalPolicy::SafetyOfficer, requested_by)
                .expect("approval required");

        task.apply_plan(plan).expect("plan should attach");
        task.request_approval(approval.id)
            .expect("approval should be requested");
        approval
            .approve(
                ActorHandle {
                    id: "worker_2001".to_string(),
                    display_name: "Wang Safety".to_string(),
                    role: "Safety Officer".to_string(),
                },
                Some("Proceed with diagnostic work".to_string()),
            )
            .expect("approval should succeed");
        task.approve().expect("task should be approved");
        task.start_execution().expect("task should execute");
        task.complete().expect("task should complete");

        assert_eq!(approval.status, ApprovalStatus::Approved);
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[test]
    fn task_record_rejects_invalid_transition() {
        let request = test_request();
        let mut task = TaskRecord::draft(request);

        let error = task.complete().expect_err("draft cannot complete");

        assert_eq!(
            error,
            LifecycleError::InvalidTaskTransition {
                from: TaskStatus::Draft,
                to: TaskStatus::Completed,
            }
        );
    }

    #[test]
    fn auto_approval_requires_auto_policy() {
        let request = test_request();
        let request_id = request.id;
        let mut task = TaskRecord::draft(request);

        task.apply_plan(test_plan(request_id, ApprovalPolicy::Auto))
            .expect("plan should attach");
        task.auto_approve().expect("auto policy should approve");

        assert_eq!(task.status, TaskStatus::Approved);
    }

    #[test]
    fn approval_policy_auto_cannot_create_manual_approval_record() {
        let request = test_request();

        let error = ApprovalRecord::pending(request.id, ApprovalPolicy::Auto, request.initiator)
            .expect_err("auto approval should not create manual record");

        assert_eq!(
            error,
            LifecycleError::ApprovalNotRequired(ApprovalPolicy::Auto)
        );
    }

    #[test]
    fn approval_record_rejects_decision_from_wrong_role() {
        let request = test_request();
        let mut approval =
            ApprovalRecord::pending(request.id, ApprovalPolicy::SafetyOfficer, request.initiator)
                .expect("approval required");

        let error = approval
            .approve(
                ActorHandle {
                    id: "worker_2001".to_string(),
                    display_name: "Chen QE".to_string(),
                    role: "Quality Engineer".to_string(),
                },
                Some("Proceed with diagnostic work".to_string()),
            )
            .expect_err("mismatched role should be rejected");

        assert_eq!(
            error,
            LifecycleError::ApprovalRoleMismatch {
                required_role: "safety_officer".to_string(),
                actual_role: "quality_engineer".to_string(),
            }
        );
        assert_eq!(approval.status, ApprovalStatus::Pending);
        assert!(approval.decided_by.is_none());
    }
}
