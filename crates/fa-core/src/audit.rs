use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use fa_domain::ActorHandle;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventKind {
    TaskCreated,
    TaskPlanned,
    TaskStatusChanged,
    ApprovalRequested,
    ApprovalApproved,
    ApprovalRejected,
    ApprovalExpired,
    ConnectorRead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActor {
    Human(ActorHandle),
    Agent(String),
    System(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub correlation_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub kind: AuditEventKind,
    pub task_id: Option<Uuid>,
    pub approval_id: Option<Uuid>,
    pub actor: AuditActor,
    pub summary: String,
}

pub trait AuditSink: Send + Sync {
    fn record(&self, event: AuditEvent) -> Result<()>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEventQuery {
    pub task_id: Option<Uuid>,
    pub approval_id: Option<Uuid>,
    pub correlation_id: Option<String>,
    pub kind: Option<AuditEventKind>,
}

impl AuditEventQuery {
    pub fn matches(&self, event: &AuditEvent) -> bool {
        self.task_id
            .is_none_or(|task_id| event.task_id == Some(task_id))
            && self
                .approval_id
                .is_none_or(|approval_id| event.approval_id == Some(approval_id))
            && self
                .correlation_id
                .as_ref()
                .is_none_or(|correlation_id| event.correlation_id.as_ref() == Some(correlation_id))
            && self.kind.as_ref().is_none_or(|kind| &event.kind == kind)
    }
}

pub trait AuditStore: AuditSink {
    fn snapshot(&self) -> Result<Vec<AuditEvent>>;

    fn query(&self, query: &AuditEventQuery) -> Result<Vec<AuditEvent>> {
        self.snapshot().map(|events| {
            events
                .into_iter()
                .filter(|event| query.matches(event))
                .collect()
        })
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopAuditSink;

impl AuditSink for NoopAuditSink {
    fn record(&self, _event: AuditEvent) -> Result<()> {
        Ok(())
    }
}

impl AuditStore for NoopAuditSink {
    fn snapshot(&self) -> Result<Vec<AuditEvent>> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryAuditSink {
    events: Arc<Mutex<Vec<AuditEvent>>>,
}

impl AuditStore for InMemoryAuditSink {
    fn snapshot(&self) -> Result<Vec<AuditEvent>> {
        self.events
            .lock()
            .map(|events| events.clone())
            .map_err(|_| anyhow!("audit sink lock poisoned"))
    }
}

impl AuditSink for InMemoryAuditSink {
    fn record(&self, event: AuditEvent) -> Result<()> {
        self.events
            .lock()
            .map(|mut events| events.push(event))
            .map_err(|_| anyhow!("audit sink lock poisoned"))
    }
}

#[derive(Debug, Clone)]
pub struct FileAuditStore {
    path: PathBuf,
    write_lock: Arc<Mutex<()>>,
}

impl FileAuditStore {
    pub fn new(data_dir: impl Into<PathBuf>) -> Result<Self> {
        let data_dir = data_dir.into();
        fs::create_dir_all(&data_dir).with_context(|| {
            format!(
                "failed to create data directory for file audit store: {}",
                data_dir.display()
            )
        })?;
        let path = data_dir.join("audit-events.jsonl");
        if !path.exists() {
            fs::write(&path, "").with_context(|| {
                format!(
                    "failed to initialize audit store file at {}",
                    path.display()
                )
            })?;
        }

        Ok(Self {
            path,
            write_lock: Arc::new(Mutex::new(())),
        })
    }
}

impl AuditStore for FileAuditStore {
    fn snapshot(&self) -> Result<Vec<AuditEvent>> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_| anyhow!("file audit store lock poisoned"))?;
        let content = fs::read_to_string(&self.path).with_context(|| {
            format!("failed to read audit store file at {}", self.path.display())
        })?;

        content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<AuditEvent>(line)
                    .with_context(|| format!("failed to decode audit event from line: {line}"))
            })
            .collect()
    }
}

impl AuditSink for FileAuditStore {
    fn record(&self, event: AuditEvent) -> Result<()> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_| anyhow!("file audit store lock poisoned"))?;
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.path)
            .with_context(|| {
                format!("failed to open audit store file at {}", self.path.display())
            })?;
        let encoded = serde_json::to_string(&event).context("failed to encode audit event")?;
        writeln!(file, "{encoded}").context("failed to append audit event")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn temp_dir(prefix: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).expect("temp dir should create");
        path
    }

    #[test]
    fn in_memory_audit_sink_records_events() {
        let sink = InMemoryAuditSink::default();
        let event = AuditEvent {
            id: Uuid::new_v4(),
            correlation_id: Some("corr-1".to_string()),
            occurred_at: Utc::now(),
            kind: AuditEventKind::TaskCreated,
            task_id: None,
            approval_id: None,
            actor: AuditActor::System("test".to_string()),
            summary: "created".to_string(),
        };

        sink.record(event.clone()).expect("event should record");
        let snapshot = sink.snapshot().expect("snapshot should be readable");

        assert_eq!(snapshot, vec![event]);
    }

    #[test]
    fn audit_query_filters_by_task_correlation_and_kind() {
        let sink = InMemoryAuditSink::default();
        let task_id = Uuid::new_v4();
        let other_task_id = Uuid::new_v4();
        let first = AuditEvent {
            id: Uuid::new_v4(),
            correlation_id: Some("corr-3".to_string()),
            occurred_at: Utc::now(),
            kind: AuditEventKind::TaskCreated,
            task_id: Some(task_id),
            approval_id: None,
            actor: AuditActor::System("test".to_string()),
            summary: "created".to_string(),
        };
        let second = AuditEvent {
            id: Uuid::new_v4(),
            correlation_id: Some("corr-4".to_string()),
            occurred_at: Utc::now(),
            kind: AuditEventKind::ApprovalRequested,
            task_id: Some(task_id),
            approval_id: Some(Uuid::new_v4()),
            actor: AuditActor::System("test".to_string()),
            summary: "approval".to_string(),
        };
        let third = AuditEvent {
            id: Uuid::new_v4(),
            correlation_id: Some("corr-3".to_string()),
            occurred_at: Utc::now(),
            kind: AuditEventKind::TaskCreated,
            task_id: Some(other_task_id),
            approval_id: None,
            actor: AuditActor::System("test".to_string()),
            summary: "other".to_string(),
        };

        sink.record(first.clone()).expect("first should record");
        sink.record(second.clone()).expect("second should record");
        sink.record(third).expect("third should record");

        let task_events = sink
            .query(&AuditEventQuery {
                task_id: Some(task_id),
                ..AuditEventQuery::default()
            })
            .expect("task query should succeed");
        assert_eq!(task_events, vec![first.clone(), second.clone()]);

        let correlation_events = sink
            .query(&AuditEventQuery {
                correlation_id: Some("corr-4".to_string()),
                ..AuditEventQuery::default()
            })
            .expect("correlation query should succeed");
        assert_eq!(correlation_events, vec![second.clone()]);

        let kind_events = sink
            .query(&AuditEventQuery {
                task_id: Some(task_id),
                kind: Some(AuditEventKind::ApprovalRequested),
                ..AuditEventQuery::default()
            })
            .expect("kind query should succeed");
        assert_eq!(kind_events, vec![second]);
    }

    #[test]
    fn file_audit_store_persists_events_across_instances() {
        let dir = temp_dir("fa-audit-store-test");
        let sink = FileAuditStore::new(&dir).expect("file store should create");
        let event = AuditEvent {
            id: Uuid::new_v4(),
            correlation_id: Some("corr-2".to_string()),
            occurred_at: Utc::now(),
            kind: AuditEventKind::TaskCreated,
            task_id: None,
            approval_id: None,
            actor: AuditActor::System("test".to_string()),
            summary: "persisted".to_string(),
        };

        sink.record(event.clone()).expect("event should record");

        let reopened = FileAuditStore::new(&dir).expect("file store should reopen");
        let snapshot = reopened.snapshot().expect("snapshot should be readable");

        assert_eq!(snapshot, vec![event]);
        fs::remove_dir_all(dir).expect("temp dir should clean");
    }
}
