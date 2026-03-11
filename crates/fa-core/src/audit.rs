use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
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

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopAuditSink;

impl AuditSink for NoopAuditSink {
    fn record(&self, _event: AuditEvent) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryAuditSink {
    events: Arc<Mutex<Vec<AuditEvent>>>,
}

impl InMemoryAuditSink {
    pub fn snapshot(&self) -> Result<Vec<AuditEvent>> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
