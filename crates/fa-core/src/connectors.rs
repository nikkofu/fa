use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorKind {
    Mes,
    Erp,
    Cmms,
    Quality,
    Scada,
    Warehouse,
    Safety,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorAccess {
    ReadOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorSubject {
    Task(Uuid),
    Equipment(String),
    Line(String),
    Site(String),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorRecordKind {
    TaskContext,
    EquipmentTelemetry,
    MaintenanceHistory,
    WorkOrderContext,
    QualityContext,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorReadRequest {
    pub correlation_id: Option<String>,
    pub task_id: Option<Uuid>,
    pub subject: ConnectorSubject,
    pub requested_records: Vec<ConnectorRecordKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorRecord {
    pub kind: ConnectorRecordKind,
    pub source_ref: String,
    pub payload: String,
    pub observed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorReadResult {
    pub connector: ConnectorKind,
    pub records: Vec<ConnectorRecord>,
}

pub trait Connector: Send + Sync {
    fn kind(&self) -> ConnectorKind;

    fn access(&self) -> ConnectorAccess {
        ConnectorAccess::ReadOnly
    }

    fn read(&self, request: &ConnectorReadRequest) -> Result<ConnectorReadResult>;
}
