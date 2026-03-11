use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::orchestrator::{OrchestrationError, TrackedTaskState};
use crate::sqlite_cli::SqliteCliDatabase;

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

#[derive(Clone)]
pub struct FileTaskRepository {
    tasks_dir: PathBuf,
    write_lock: Arc<Mutex<()>>,
}

impl FileTaskRepository {
    pub fn new(data_dir: impl Into<PathBuf>) -> Result<Self> {
        let tasks_dir = data_dir.into().join("tasks");
        fs::create_dir_all(&tasks_dir).with_context(|| {
            format!(
                "failed to create task repository directory: {}",
                tasks_dir.display()
            )
        })?;

        Ok(Self {
            tasks_dir,
            write_lock: Arc::new(Mutex::new(())),
        })
    }

    fn task_path(&self, task_id: Uuid) -> PathBuf {
        self.tasks_dir.join(format!("{task_id}.json"))
    }

    fn write_state(&self, state: &TrackedTaskState) -> std::result::Result<(), OrchestrationError> {
        let task_id = state.planned_task.task.id;
        let path = self.task_path(task_id);
        let temp_path = self.tasks_dir.join(format!("{task_id}.tmp"));
        let encoded = serde_json::to_vec_pretty(state).map_err(|error| {
            OrchestrationError::TaskRepository(format!("failed to encode task state: {error}"))
        })?;
        fs::write(&temp_path, encoded).map_err(|error| {
            OrchestrationError::TaskRepository(format!(
                "failed to write temp task state file {}: {error}",
                temp_path.display()
            ))
        })?;
        fs::rename(&temp_path, &path).map_err(|error| {
            OrchestrationError::TaskRepository(format!(
                "failed to move temp task state into place {}: {error}",
                path.display()
            ))
        })
    }
}

impl TaskRepository for FileTaskRepository {
    fn create(&self, state: TrackedTaskState) -> std::result::Result<(), OrchestrationError> {
        let _guard = self.write_lock.lock().map_err(|_| {
            OrchestrationError::TaskRepository("file repository lock poisoned".to_string())
        })?;
        let path = self.task_path(state.planned_task.task.id);
        if path.exists() {
            return Err(OrchestrationError::TaskAlreadyExists(
                state.planned_task.task.id,
            ));
        }

        self.write_state(&state)
    }

    fn get(
        &self,
        task_id: Uuid,
    ) -> std::result::Result<Option<TrackedTaskState>, OrchestrationError> {
        let path = self.task_path(task_id);
        if !path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(&path).map_err(|error| {
            OrchestrationError::TaskRepository(format!(
                "failed to read task state file {}: {error}",
                path.display()
            ))
        })?;
        let state = serde_json::from_slice(&bytes).map_err(|error| {
            OrchestrationError::TaskRepository(format!(
                "failed to decode task state file {}: {error}",
                path.display()
            ))
        })?;
        Ok(Some(state))
    }

    fn save(
        &self,
        state: TrackedTaskState,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError> {
        let _guard = self.write_lock.lock().map_err(|_| {
            OrchestrationError::TaskRepository("file repository lock poisoned".to_string())
        })?;
        let task_id = state.planned_task.task.id;
        let path = self.task_path(task_id);
        if !path.exists() {
            return Err(OrchestrationError::TaskNotFound(task_id));
        }

        self.write_state(&state)?;
        Ok(state)
    }
}

#[derive(Clone)]
pub struct SqliteTaskRepository {
    database: SqliteCliDatabase,
}

impl SqliteTaskRepository {
    pub fn new(db_path: impl Into<PathBuf>) -> Result<Self> {
        let database = SqliteCliDatabase::new(db_path)?;
        database.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                task_id TEXT PRIMARY KEY,
                correlation_id TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                payload_json TEXT NOT NULL
            );",
        )?;

        Ok(Self { database })
    }

    fn exists(&self, task_id: Uuid) -> std::result::Result<bool, OrchestrationError> {
        let output = self
            .database
            .execute(&format!(
                "SELECT EXISTS(SELECT 1 FROM tasks WHERE task_id = {});",
                SqliteCliDatabase::quote(&task_id.to_string())
            ))
            .map_err(|error| OrchestrationError::TaskRepository(error.to_string()))?;
        Ok(output == "1")
    }

    fn persist_state(
        &self,
        state: &TrackedTaskState,
        is_insert: bool,
    ) -> std::result::Result<(), OrchestrationError> {
        let payload = serde_json::to_string(state).map_err(|error| {
            OrchestrationError::TaskRepository(format!(
                "failed to encode sqlite task state: {error}"
            ))
        })?;
        let payload_path = self
            .database
            .write_temp_json("task", &payload)
            .map_err(|error| OrchestrationError::TaskRepository(error.to_string()))?;
        let task_id = state.planned_task.task.id.to_string();
        let correlation_id = &state.correlation_id;
        let updated_at = state.planned_task.task.updated_at.to_rfc3339();
        let sql = if is_insert {
            format!(
                "INSERT INTO tasks(task_id, correlation_id, updated_at, payload_json) VALUES ({task_id}, {correlation_id}, {updated_at}, CAST(readfile({payload_path}) AS TEXT));",
                task_id = SqliteCliDatabase::quote(&task_id),
                correlation_id = SqliteCliDatabase::quote(correlation_id),
                updated_at = SqliteCliDatabase::quote(&updated_at),
                payload_path = SqliteCliDatabase::quote(&payload_path.display().to_string()),
            )
        } else {
            format!(
                "UPDATE tasks SET correlation_id = {correlation_id}, updated_at = {updated_at}, payload_json = CAST(readfile({payload_path}) AS TEXT) WHERE task_id = {task_id};",
                task_id = SqliteCliDatabase::quote(&task_id),
                correlation_id = SqliteCliDatabase::quote(correlation_id),
                updated_at = SqliteCliDatabase::quote(&updated_at),
                payload_path = SqliteCliDatabase::quote(&payload_path.display().to_string()),
            )
        };

        let result = self
            .database
            .execute(&sql)
            .map(|_| ())
            .map_err(|error| OrchestrationError::TaskRepository(error.to_string()));
        let _ = fs::remove_file(payload_path);
        result
    }
}

impl TaskRepository for SqliteTaskRepository {
    fn create(&self, state: TrackedTaskState) -> std::result::Result<(), OrchestrationError> {
        if self.exists(state.planned_task.task.id)? {
            return Err(OrchestrationError::TaskAlreadyExists(
                state.planned_task.task.id,
            ));
        }

        self.persist_state(&state, true)
    }

    fn get(
        &self,
        task_id: Uuid,
    ) -> std::result::Result<Option<TrackedTaskState>, OrchestrationError> {
        let output = self
            .database
            .execute(&format!(
                "SELECT payload_json FROM tasks WHERE task_id = {};",
                SqliteCliDatabase::quote(&task_id.to_string())
            ))
            .map_err(|error| OrchestrationError::TaskRepository(error.to_string()))?;

        if output.is_empty() {
            return Ok(None);
        }

        serde_json::from_str(&output).map(Some).map_err(|error| {
            OrchestrationError::TaskRepository(format!(
                "failed to decode sqlite task state for {task_id}: {error}"
            ))
        })
    }

    fn save(
        &self,
        state: TrackedTaskState,
    ) -> std::result::Result<TrackedTaskState, OrchestrationError> {
        let task_id = state.planned_task.task.id;
        if !self.exists(task_id)? {
            return Err(OrchestrationError::TaskNotFound(task_id));
        }

        self.persist_state(&state, false)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::Utc;
    use fa_domain::{
        ActorHandle, AgenticPattern, ApprovalPolicy, ExecutionPlan, PlanOwner, PlannedStep,
        PlannedTaskBundle, TaskPriority, TaskRecord, TaskRequest, TaskRisk, WorkflowGovernance,
    };
    use uuid::Uuid;

    use super::*;

    fn temp_dir(prefix: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).expect("temp dir should create");
        path
    }

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
            governance: WorkflowGovernance::default(),
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
            evidence: Vec::new(),
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

    #[test]
    fn file_repository_persists_tracked_state_across_instances() {
        let dir = temp_dir("fa-task-repository-test");
        let repository = FileTaskRepository::new(&dir).expect("file repository should create");
        let task_id = Uuid::new_v4();
        let state = tracked_state(task_id);

        repository
            .create(state.clone())
            .expect("create should succeed");

        let reopened = FileTaskRepository::new(&dir).expect("file repository should reopen");
        let stored = reopened
            .get(task_id)
            .expect("get should succeed")
            .expect("task should exist");

        assert_eq!(stored, state);
        fs::remove_dir_all(dir).expect("temp dir should clean");
    }

    #[test]
    fn sqlite_repository_persists_tracked_state_across_instances() {
        let dir = temp_dir("fa-sqlite-task-repository-test");
        let db_path = dir.join("fa.db");
        let repository =
            SqliteTaskRepository::new(&db_path).expect("sqlite repository should create");
        let task_id = Uuid::new_v4();
        let state = tracked_state(task_id);

        repository
            .create(state.clone())
            .expect("create should succeed");

        let reopened =
            SqliteTaskRepository::new(&db_path).expect("sqlite repository should reopen");
        let stored = reopened
            .get(task_id)
            .expect("get should succeed")
            .expect("task should exist");

        assert_eq!(stored, state);
        fs::remove_dir_all(dir).expect("temp dir should clean");
    }
}
