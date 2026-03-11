use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use uuid::Uuid;

use crate::orchestrator::{OrchestrationError, TrackedTaskState};

pub trait TaskRepository: Send + Sync {
    fn create(&self, state: TrackedTaskState) -> std::result::Result<(), OrchestrationError>;

    fn get(
        &self,
        task_id: Uuid,
    ) -> std::result::Result<Option<TrackedTaskState>, OrchestrationError>;

    fn save(
        &self,
        state: TrackedTaskState,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError>;
}

#[derive(Clone, Default)]
pub struct InMemoryTaskRepository {
    tasks: Arc<Mutex<HashMap<Uuid, TrackedTaskState>>>,
}

impl TaskRepository for InMemoryTaskRepository {
    fn create(&self, state: TrackedTaskState) -> std::result::Result<(), OrchestrationError> {
        let mut tasks = self.tasks.lock().map_err(|_| {
            OrchestrationError::TaskRepository("in-memory repository lock poisoned".to_string())
        })?;

        if tasks.contains_key(&state.planned_task.task.id) {
            return Err(OrchestrationError::TaskAlreadyExists(
                state.planned_task.task.id,
            ));
        }

        tasks.insert(state.planned_task.task.id, state);
        Ok(())
    }

    fn get(
        &self,
        task_id: Uuid,
    ) -> std::result::Result<Option<TrackedTaskState>, OrchestrationError> {
        self.tasks
            .lock()
            .map_err(|_| {
                OrchestrationError::TaskRepository("in-memory repository lock poisoned".to_string())
            })
            .map(|tasks| tasks.get(&task_id).cloned())
    }

    fn save(
        &self,
        state: TrackedTaskState,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError> {
        let task_id = state.planned_task.task.id;
        let mut tasks = self.tasks.lock().map_err(|_| {
            OrchestrationError::TaskRepository("in-memory repository lock poisoned".to_string())
        })?;

        if !tasks.contains_key(&task_id) {
            return Err(OrchestrationError::TaskNotFound(task_id));
        }

        tasks.insert(task_id, state.clone());
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use fa_domain::{
        ActorHandle, AgenticPattern, ApprovalPolicy, ExecutionPlan, PlanOwner, PlannedStep,
        PlannedTaskBundle, TaskPriority, TaskRecord, TaskRequest, TaskRisk,
    };
    use uuid::Uuid;

    use super::*;

    fn base_request() -> TaskRequest {
        TaskRequest {
            id: Uuid::new_v4(),
            title: "Inspect spindle cooling loop".to_string(),
            description: "Validate the spindle cooling loop before restart.".to_string(),
            priority: TaskPriority::Expedited,
            risk: TaskRisk::High,
            initiator: ActorHandle {
                id: "worker_1001".to_string(),
                display_name: "Liu Supervisor".to_string(),
                role: "Production Supervisor".to_string(),
            },
            stakeholders: Vec::new(),
            equipment_ids: vec!["eq_cnc_01".to_string()],
            integrations: Vec::new(),
            desired_outcome: "Recover stable spindle temperature".to_string(),
            requires_human_approval: true,
            requires_diagnostic_loop: true,
        }
    }

    fn tracked_state(task_id: Uuid) -> TrackedTaskState {
        let request = TaskRequest {
            id: task_id,
            ..base_request()
        };
        let mut task = TaskRecord::draft(request.clone());
        task.apply_plan(ExecutionPlan {
            request_id: request.id,
            patterns: vec![AgenticPattern::Coordinator],
            rationale: vec!["test".to_string()],
            approval_policy: ApprovalPolicy::SafetyOfficer,
            steps: vec![PlannedStep {
                sequence: 1,
                label: "Test".to_string(),
                owner: PlanOwner::System("workflow-engine".to_string()),
                expected_output: "ok".to_string(),
            }],
            created_at: Utc::now(),
        })
        .expect("plan should apply");

        TrackedTaskState {
            correlation_id: "repo-001".to_string(),
            planned_task: PlannedTaskBundle {
                task,
                approval: None,
            },
            context_reads: Vec::new(),
        }
    }

    #[test]
    fn in_memory_repository_creates_and_reads_tracked_state() {
        let repository = InMemoryTaskRepository::default();
        let task_id = Uuid::new_v4();
        let state = tracked_state(task_id);

        repository
            .create(state.clone())
            .expect("create should succeed");
        let stored = repository
            .get(task_id)
            .expect("get should succeed")
            .expect("task should exist");

        assert_eq!(stored, state);
    }

    #[test]
    fn in_memory_repository_saves_existing_task_state() {
        let repository = InMemoryTaskRepository::default();
        let task_id = Uuid::new_v4();
        let mut state = tracked_state(task_id);

        repository
            .create(state.clone())
            .expect("create should succeed");
        state.correlation_id = "repo-002".to_string();

        let saved = repository.save(state).expect("save should succeed");

        assert_eq!(saved.correlation_id, "repo-002");
    }
}
