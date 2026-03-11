use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::connectors::{ConnectorKind, ConnectorReadResult, ConnectorRecord, ConnectorRecordKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskEvidence {
    pub connector: ConnectorKind,
    pub record_kind: ConnectorRecordKind,
    pub source_ref: String,
    pub observed_at: Option<DateTime<Utc>>,
    pub summary: String,
    pub payload: String,
}

pub fn evidence_from_context_reads(context_reads: &[ConnectorReadResult]) -> Vec<TaskEvidence> {
    context_reads
        .iter()
        .flat_map(|result| {
            result.records.iter().map(|record| TaskEvidence {
                connector: result.connector.clone(),
                record_kind: record.kind.clone(),
                source_ref: record.source_ref.clone(),
                observed_at: record.observed_at,
                summary: summarize_record(&result.connector, record),
                payload: record.payload.clone(),
            })
        })
        .collect()
}

fn summarize_record(connector: &ConnectorKind, record: &ConnectorRecord) -> String {
    let payload = serde_json::from_str::<Value>(&record.payload).ok();

    match record.kind {
        ConnectorRecordKind::TaskContext => format!(
            "{connector:?} task context for {} is {}",
            field(&payload, "subject").unwrap_or("the requested subject"),
            field(&payload, "status").unwrap_or("available")
        ),
        ConnectorRecordKind::EquipmentTelemetry => {
            let subject = field(&payload, "subject").unwrap_or("the requested equipment");
            let trend = field(&payload, "trend").unwrap_or("stable");
            if let Some(temperature_c) = payload
                .as_ref()
                .and_then(|value| value.get("temperature_c"))
            {
                format!(
                    "{connector:?} telemetry shows {subject} at {temperature_c}C with {trend} trend"
                )
            } else {
                format!("{connector:?} telemetry captured {trend} trend for {subject}")
            }
        }
        ConnectorRecordKind::MaintenanceHistory => format!(
            "{connector:?} history for {} highlights {}",
            field(&payload, "subject").unwrap_or("the requested asset"),
            field(&payload, "finding").unwrap_or("recent maintenance findings")
        ),
        ConnectorRecordKind::WorkOrderContext => format!(
            "{connector:?} recommends {} for {}",
            field(&payload, "recommended_action").unwrap_or("follow-up maintenance"),
            field(&payload, "subject").unwrap_or("the requested asset")
        ),
        ConnectorRecordKind::QualityContext => format!(
            "{connector:?} quality context is available from {}",
            record.source_ref
        ),
        ConnectorRecordKind::Custom(_) => format!(
            "{connector:?} returned contextual evidence from {}",
            record.source_ref
        ),
    }
}

fn field<'a>(payload: &'a Option<Value>, key: &str) -> Option<&'a str> {
    payload
        .as_ref()
        .and_then(|value| value.get(key))
        .and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::{ConnectorReadResult, ConnectorRecord};

    #[test]
    fn evidence_snapshots_are_built_from_connector_reads() {
        let context_reads = vec![ConnectorReadResult {
            connector: ConnectorKind::Mes,
            records: vec![ConnectorRecord {
                kind: ConnectorRecordKind::EquipmentTelemetry,
                source_ref: "mes://telemetry/spindle".to_string(),
                payload:
                    "{\"subject\":\"eq_cnc_01\",\"temperature_c\":67,\"trend\":\"drifting_up\"}"
                        .to_string(),
                observed_at: None,
            }],
        }];

        let evidence = evidence_from_context_reads(&context_reads);

        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].connector, ConnectorKind::Mes);
        assert_eq!(
            evidence[0].record_kind,
            ConnectorRecordKind::EquipmentTelemetry
        );
        assert!(evidence[0].summary.contains("67"));
        assert!(evidence[0].summary.contains("drifting_up"));
    }
}
