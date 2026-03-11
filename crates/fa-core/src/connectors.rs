use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use fa_domain::IntegrationTarget;

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

#[derive(Clone, Default)]
pub struct ConnectorRegistry {
    connectors: Vec<Arc<dyn Connector>>,
}

impl ConnectorRegistry {
    pub fn new(connectors: Vec<Arc<dyn Connector>>) -> Self {
        Self { connectors }
    }

    pub fn with_m1_defaults() -> Self {
        Self::new(vec![
            Arc::new(MockMesConnector),
            Arc::new(MockCmmsConnector),
        ])
    }

    pub fn connector_for_kind(&self, kind: &ConnectorKind) -> Option<Arc<dyn Connector>> {
        self.connectors
            .iter()
            .find(|connector| connector.kind() == *kind)
            .cloned()
    }

    pub fn kind_for_target(target: &IntegrationTarget) -> Option<ConnectorKind> {
        match target {
            IntegrationTarget::Mes => Some(ConnectorKind::Mes),
            IntegrationTarget::Cmms => Some(ConnectorKind::Cmms),
            IntegrationTarget::Erp => Some(ConnectorKind::Erp),
            IntegrationTarget::Quality => Some(ConnectorKind::Quality),
            IntegrationTarget::Scada => Some(ConnectorKind::Scada),
            IntegrationTarget::Warehouse => Some(ConnectorKind::Warehouse),
            IntegrationTarget::Safety => Some(ConnectorKind::Safety),
            IntegrationTarget::Custom(value) => Some(ConnectorKind::Custom(value.clone())),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MockMesConnector;

impl Connector for MockMesConnector {
    fn kind(&self) -> ConnectorKind {
        ConnectorKind::Mes
    }

    fn read(&self, request: &ConnectorReadRequest) -> Result<ConnectorReadResult> {
        let records = request
            .requested_records
            .iter()
            .filter_map(|kind| match kind {
                ConnectorRecordKind::TaskContext => Some(ConnectorRecord {
                    kind: ConnectorRecordKind::TaskContext,
                    source_ref: "mes://orders/active".to_string(),
                    payload: format!(
                        "{{\"subject\":\"{}\",\"status\":\"running\",\"active_order\":\"MO-20260311-01\"}}",
                        subject_ref(&request.subject)
                    ),
                    observed_at: Some(Utc::now()),
                }),
                ConnectorRecordKind::EquipmentTelemetry => Some(ConnectorRecord {
                    kind: ConnectorRecordKind::EquipmentTelemetry,
                    source_ref: "mes://telemetry/spindle".to_string(),
                    payload: format!(
                        "{{\"subject\":\"{}\",\"temperature_c\":67,\"trend\":\"drifting_up\"}}",
                        subject_ref(&request.subject)
                    ),
                    observed_at: Some(Utc::now()),
                }),
                _ => None,
            })
            .collect();

        Ok(ConnectorReadResult {
            connector: ConnectorKind::Mes,
            records,
        })
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MockCmmsConnector;

impl Connector for MockCmmsConnector {
    fn kind(&self) -> ConnectorKind {
        ConnectorKind::Cmms
    }

    fn read(&self, request: &ConnectorReadRequest) -> Result<ConnectorReadResult> {
        let records = request
            .requested_records
            .iter()
            .filter_map(|kind| match kind {
                ConnectorRecordKind::MaintenanceHistory => Some(ConnectorRecord {
                    kind: ConnectorRecordKind::MaintenanceHistory,
                    source_ref: "cmms://history/eq_cnc_01".to_string(),
                    payload: format!(
                        "{{\"subject\":\"{}\",\"last_work_order\":\"WO-2048\",\"finding\":\"bearing_wear_watch\"}}",
                        subject_ref(&request.subject)
                    ),
                    observed_at: Some(Utc::now()),
                }),
                ConnectorRecordKind::WorkOrderContext => Some(ConnectorRecord {
                    kind: ConnectorRecordKind::WorkOrderContext,
                    source_ref: "cmms://recommendations".to_string(),
                    payload: format!(
                        "{{\"subject\":\"{}\",\"recommended_action\":\"inspect_spindle_cooling_loop\"}}",
                        subject_ref(&request.subject)
                    ),
                    observed_at: Some(Utc::now()),
                }),
                _ => None,
            })
            .collect();

        Ok(ConnectorReadResult {
            connector: ConnectorKind::Cmms,
            records,
        })
    }
}

fn subject_ref(subject: &ConnectorSubject) -> String {
    match subject {
        ConnectorSubject::Task(task_id) => task_id.to_string(),
        ConnectorSubject::Equipment(value)
        | ConnectorSubject::Line(value)
        | ConnectorSubject::Site(value)
        | ConnectorSubject::Custom(value) => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connector_registry_resolves_mes_and_cmms() {
        let registry = ConnectorRegistry::with_m1_defaults();

        assert!(registry.connector_for_kind(&ConnectorKind::Mes).is_some());
        assert!(registry.connector_for_kind(&ConnectorKind::Cmms).is_some());
    }

    #[test]
    fn mock_mes_connector_returns_requested_context() {
        let connector = MockMesConnector;
        let result = connector
            .read(&ConnectorReadRequest {
                correlation_id: Some("corr-1".to_string()),
                task_id: Some(Uuid::new_v4()),
                subject: ConnectorSubject::Equipment("eq_cnc_01".to_string()),
                requested_records: vec![
                    ConnectorRecordKind::TaskContext,
                    ConnectorRecordKind::EquipmentTelemetry,
                ],
            })
            .expect("read should succeed");

        assert_eq!(result.connector, ConnectorKind::Mes);
        assert_eq!(result.records.len(), 2);
    }
}
