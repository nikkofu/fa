use axum::{
    extract::{Query, State},
    http::header,
    response::{Html, IntoResponse},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use fa_core::{
    bootstrap_blueprint, AlertClusterMonitoringView, AlertClusterQueueItemView,
    AlertClusterQueueQuery, DeliveryTrack, FollowUpMonitoringView, FollowUpQueueItemView,
    FollowUpQueueQuery, HandoffReceiptMonitoringView, HandoffReceiptQueueItemView,
    HandoffReceiptQueueQuery, OrchestrationError, PatternDecision, SystemLayer,
};

use crate::{error_response, AppState};

const QUEUE_PREVIEW_LIMIT: usize = 6;

#[derive(Debug, Clone, Serialize)]
pub struct ExperienceOverview {
    pub generated_at: DateTime<Utc>,
    pub service: ExperienceServiceSummary,
    pub lens: ExperienceLensSummary,
    pub blueprint: ExperienceBlueprintSummary,
    pub monitoring: ExperienceMonitoringSummary,
    pub queues: ExperienceQueueSummary,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperienceLens {
    #[default]
    Executive,
    ProductionSupervisor,
    MaintenanceEngineer,
    IncomingShiftSupervisor,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ExperienceOverviewQueryParams {
    pub lens: ExperienceLens,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExperienceServiceSummary {
    pub status: &'static str,
    pub service_name: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExperienceLensSummary {
    pub active_lens: ExperienceLens,
    pub label: &'static str,
    pub description: &'static str,
    pub focus_areas: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExperienceBlueprintSummary {
    pub platform_name: String,
    pub vision: String,
    pub pattern_count: usize,
    pub system_layer_count: usize,
    pub delivery_track_count: usize,
    pub organization_count: usize,
    pub site_count: usize,
    pub line_count: usize,
    pub worker_count: usize,
    pub agent_count: usize,
    pub selected_patterns: Vec<PatternDecision>,
    pub system_layers: Vec<SystemLayer>,
    pub delivery_tracks: Vec<DeliveryTrack>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExperienceMonitoringSummary {
    pub follow_up: FollowUpMonitoringView,
    pub handoff: HandoffReceiptMonitoringView,
    pub alert_cluster: AlertClusterMonitoringView,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExperienceQueueSummary {
    pub follow_up_items: Vec<FollowUpQueueItemView>,
    pub handoff_receipts: Vec<HandoffReceiptQueueItemView>,
    pub alert_clusters: Vec<AlertClusterQueueItemView>,
}

pub async fn experience_shell() -> Html<&'static str> {
    Html(include_str!("ui/index.html"))
}

pub async fn experience_styles() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("ui/app.css"),
    )
}

pub async fn experience_script() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("ui/app.js"),
    )
}

pub async fn experience_overview(
    State(state): State<AppState>,
    Query(query): Query<ExperienceOverviewQueryParams>,
) -> Result<Json<ExperienceOverview>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    build_experience_overview(&state, query.lens)
        .map(Json)
        .map_err(error_response)
}

fn build_experience_overview(
    state: &AppState,
    lens: ExperienceLens,
) -> Result<ExperienceOverview, OrchestrationError> {
    let blueprint = bootstrap_blueprint();
    let (follow_up_query, handoff_query, alert_cluster_query) = experience_queries_for_lens(lens);
    let follow_up_monitoring = state
        .orchestrator
        .get_follow_up_monitoring(&follow_up_query)?;
    let handoff_monitoring = state
        .orchestrator
        .get_handoff_receipt_monitoring(&handoff_query)?;
    let alert_cluster_monitoring = state
        .orchestrator
        .get_alert_cluster_monitoring(&alert_cluster_query)?;
    let follow_up_items = state
        .orchestrator
        .list_follow_up_items(&follow_up_query)?;
    let handoff_receipts = state
        .orchestrator
        .list_handoff_receipts(&handoff_query)?;
    let alert_clusters = state
        .orchestrator
        .list_alert_clusters(&alert_cluster_query)?;

    Ok(ExperienceOverview {
        generated_at: Utc::now(),
        service: ExperienceServiceSummary {
            status: "ok",
            service_name: "fa-server",
            version: env!("CARGO_PKG_VERSION"),
        },
        lens: experience_lens_summary(lens),
        blueprint: ExperienceBlueprintSummary {
            platform_name: blueprint.platform_name,
            vision: blueprint.vision,
            pattern_count: blueprint.selected_patterns.len(),
            system_layer_count: blueprint.system_layers.len(),
            delivery_track_count: blueprint.delivery_tracks.len(),
            organization_count: blueprint.reference_enterprise.organizations.len(),
            site_count: blueprint.reference_enterprise.sites.len(),
            line_count: blueprint.reference_enterprise.lines.len(),
            worker_count: blueprint.reference_enterprise.workers.len(),
            agent_count: blueprint.reference_enterprise.agents.len(),
            selected_patterns: blueprint.selected_patterns,
            system_layers: blueprint.system_layers,
            delivery_tracks: blueprint.delivery_tracks,
        },
        monitoring: ExperienceMonitoringSummary {
            follow_up: follow_up_monitoring,
            handoff: handoff_monitoring,
            alert_cluster: alert_cluster_monitoring,
        },
        queues: ExperienceQueueSummary {
            follow_up_items: follow_up_items
                .into_iter()
                .take(QUEUE_PREVIEW_LIMIT)
                .collect(),
            handoff_receipts: handoff_receipts
                .into_iter()
                .take(QUEUE_PREVIEW_LIMIT)
                .collect(),
            alert_clusters: alert_clusters
                .into_iter()
                .take(QUEUE_PREVIEW_LIMIT)
                .collect(),
        },
    })
}

fn experience_queries_for_lens(
    lens: ExperienceLens,
) -> (
    FollowUpQueueQuery,
    HandoffReceiptQueueQuery,
    AlertClusterQueueQuery,
) {
    let mut follow_up_query = FollowUpQueueQuery::default();
    let mut handoff_query = HandoffReceiptQueueQuery::default();
    let mut alert_cluster_query = AlertClusterQueueQuery::default();

    match lens {
        ExperienceLens::Executive => {}
        ExperienceLens::ProductionSupervisor => {
            follow_up_query.owner_role = Some("production_supervisor".to_string());
            handoff_query.receiving_role = Some("production_supervisor".to_string());
            alert_cluster_query.recommended_owner_role = Some("production_supervisor".to_string());
        }
        ExperienceLens::MaintenanceEngineer => {
            follow_up_query.owner_role = Some("maintenance_engineer".to_string());
            handoff_query.receiving_role = Some("maintenance_engineer".to_string());
            alert_cluster_query.recommended_owner_role = Some("maintenance_engineer".to_string());
        }
        ExperienceLens::IncomingShiftSupervisor => {
            follow_up_query.owner_role = Some("incoming_shift_supervisor".to_string());
            handoff_query.receiving_role = Some("incoming_shift_supervisor".to_string());
            alert_cluster_query.recommended_owner_role =
                Some("incoming_shift_supervisor".to_string());
        }
    }

    (follow_up_query, handoff_query, alert_cluster_query)
}

fn experience_lens_summary(lens: ExperienceLens) -> ExperienceLensSummary {
    match lens {
        ExperienceLens::Executive => ExperienceLensSummary {
            active_lens: lens,
            label: "Control Tower",
            description:
                "Executive-wide visibility across live queues, handoffs, alerts, and AI-governed work.",
            focus_areas: vec![
                "Cross-workflow monitoring",
                "Escalation hotspots",
                "Governance visibility",
            ],
        },
        ExperienceLens::ProductionSupervisor => ExperienceLensSummary {
            active_lens: lens,
            label: "Production Desk",
            description:
                "Supervisor-focused view of first-response work, alert triage, and governed approvals.",
            focus_areas: vec![
                "Production-owned follow-ups",
                "Supervisor triage backlog",
                "Approval-critical work",
            ],
        },
        ExperienceLens::MaintenanceEngineer => ExperienceLensSummary {
            active_lens: lens,
            label: "Reliability Desk",
            description:
                "Maintenance-focused view of diagnostic ownership, sustained threshold review, and reliability response load.",
            focus_areas: vec![
                "Maintenance-owned follow-ups",
                "SCADA threshold review",
                "Diagnostic workload",
            ],
        },
        ExperienceLens::IncomingShiftSupervisor => ExperienceLensSummary {
            active_lens: lens,
            label: "Shift Lead Desk",
            description:
                "Shift-handoff view of incoming ownership, acknowledgement windows, and residual start-up risk.",
            focus_areas: vec![
                "Incoming handoff backlog",
                "Shift-owned follow-ups",
                "Acknowledgement windows",
            ],
        },
    }
}
