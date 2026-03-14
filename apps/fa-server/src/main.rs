mod experience;

use std::{env, net::SocketAddr, path::PathBuf};

use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use experience::{experience_overview, experience_script, experience_shell, experience_styles};
use fa_core::{
    bootstrap_blueprint, AcceptFollowUpOwnerRequest, AcknowledgeHandoffReceiptRequest,
    AlertClusterMonitoringView, AlertClusterQueueItemView, AlertClusterQueueQuery,
    ApprovalActionRequest, AuditEventKind, AuditEventQuery, AuditStore, CompleteTaskRequest,
    EscalateHandoffReceiptRequest, ExecuteTaskRequest, FailTaskRequest, FileAuditStore,
    FileTaskRepository, FollowUpMonitoringView, FollowUpQueueItemView, FollowUpQueueQuery,
    HandoffReceiptMonitoringView, HandoffReceiptQueueItemView, HandoffReceiptQueueQuery,
    InMemoryAuditSink, InMemoryTaskRepository, OrchestrationError, ResubmitTaskRequest,
    SqliteAuditStore, SqliteTaskRepository, TrackedTaskState, WorkOrchestrator,
};
use fa_domain::{LifecycleError, TaskPriority, TaskRequest, TaskRisk};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    orchestrator: WorkOrchestrator,
    audit_sink: Arc<dyn AuditStore>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct AuditEventsQueryParams {
    task_id: Option<Uuid>,
    approval_id: Option<Uuid>,
    correlation_id: Option<String>,
    kind: Option<AuditEventKind>,
}

impl From<AuditEventsQueryParams> for AuditEventQuery {
    fn from(value: AuditEventsQueryParams) -> Self {
        Self {
            task_id: value.task_id,
            approval_id: value.approval_id,
            correlation_id: value.correlation_id,
            kind: value.kind,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct FollowUpItemsQueryParams {
    task_id: Option<Uuid>,
    source_kind: Option<String>,
    status: Option<String>,
    owner_id: Option<String>,
    owner_role: Option<String>,
    overdue_only: bool,
    blocked_only: bool,
    escalation_required: bool,
    due_before: Option<DateTime<Utc>>,
    risk: Option<TaskRisk>,
    priority: Option<TaskPriority>,
}

impl From<FollowUpItemsQueryParams> for FollowUpQueueQuery {
    fn from(value: FollowUpItemsQueryParams) -> Self {
        Self {
            task_id: value.task_id,
            source_kind: value.source_kind,
            status: value.status,
            owner_id: value.owner_id,
            owner_role: value.owner_role,
            overdue_only: value.overdue_only,
            blocked_only: value.blocked_only,
            escalation_required: value.escalation_required,
            due_before: value.due_before,
            task_risk: value.risk,
            task_priority: value.priority,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct HandoffReceiptsQueryParams {
    task_id: Option<Uuid>,
    shift_id: Option<String>,
    receipt_status: Option<String>,
    receiving_role: Option<String>,
    receiving_actor_id: Option<String>,
    overdue_only: bool,
    has_exceptions: bool,
    escalated_only: bool,
}

impl From<HandoffReceiptsQueryParams> for HandoffReceiptQueueQuery {
    fn from(value: HandoffReceiptsQueryParams) -> Self {
        Self {
            task_id: value.task_id,
            shift_id: value.shift_id,
            receipt_status: value.receipt_status,
            receiving_role: value.receiving_role,
            receiving_actor_id: value.receiving_actor_id,
            overdue_only: value.overdue_only,
            has_exceptions: value.has_exceptions,
            escalated_only: value.escalated_only,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct AlertClustersQueryParams {
    task_id: Option<Uuid>,
    cluster_status: Option<String>,
    source_system: Option<String>,
    equipment_id: Option<String>,
    line_id: Option<String>,
    severity_band: Option<String>,
    triage_label: Option<String>,
    recommended_owner_role: Option<String>,
    follow_up_owner_id: Option<String>,
    unaccepted_follow_up_only: bool,
    follow_up_escalation_required: bool,
    escalation_candidate: bool,
    window_from: Option<DateTime<Utc>>,
    window_to: Option<DateTime<Utc>>,
    open_only: bool,
}

impl From<AlertClustersQueryParams> for AlertClusterQueueQuery {
    fn from(value: AlertClustersQueryParams) -> Self {
        Self {
            task_id: value.task_id,
            cluster_status: value.cluster_status,
            source_system: value.source_system,
            equipment_id: value.equipment_id,
            line_id: value.line_id,
            severity_band: value.severity_band,
            triage_label: value.triage_label,
            recommended_owner_role: value.recommended_owner_role,
            follow_up_owner_id: value.follow_up_owner_id,
            unaccepted_follow_up_only: value.unaccepted_follow_up_only,
            follow_up_escalation_required: value.follow_up_escalation_required,
            escalation_candidate: value.escalation_candidate,
            window_from: value.window_from,
            window_to: value.window_to,
            open_only: value.open_only,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let address = env::var("FA_SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8000".to_string());
    let socket_addr: SocketAddr = address
        .parse()
        .with_context(|| format!("invalid FA_SERVER_ADDR: {address}"))?;

    let app = app(build_state()?);

    let listener = tokio::net::TcpListener::bind(socket_addr).await?;
    tracing::info!(%socket_addr, "fa-server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn healthz() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "fa-server",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn blueprint() -> impl IntoResponse {
    Json(bootstrap_blueprint())
}

async fn plan_task(
    State(state): State<AppState>,
    Json(request): Json<TaskRequest>,
) -> impl IntoResponse {
    Json(state.orchestrator.plan_task(request))
}

async fn intake_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<TaskRequest>,
) -> Result<Json<fa_core::TaskIntakeResult>, (StatusCode, Json<serde_json::Value>)> {
    let correlation_id = headers
        .get("x-correlation-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    state
        .orchestrator
        .intake_task_with_correlation(request, correlation_id)
        .map(Json)
        .map_err(error_response)
}

async fn audit_events(
    State(state): State<AppState>,
    Query(query): Query<AuditEventsQueryParams>,
) -> Result<Json<Vec<fa_core::AuditEvent>>, (StatusCode, Json<serde_json::Value>)> {
    state
        .audit_sink
        .query(&query.into())
        .map(Json)
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": error.to_string(),
                })),
            )
        })
}

async fn list_follow_up_items(
    State(state): State<AppState>,
    Query(query): Query<FollowUpItemsQueryParams>,
) -> Result<Json<Vec<FollowUpQueueItemView>>, (StatusCode, Json<serde_json::Value>)> {
    state
        .orchestrator
        .list_follow_up_items(&query.into())
        .map(Json)
        .map_err(error_response)
}

async fn get_follow_up_monitoring(
    State(state): State<AppState>,
    Query(query): Query<FollowUpItemsQueryParams>,
) -> Result<Json<FollowUpMonitoringView>, (StatusCode, Json<serde_json::Value>)> {
    state
        .orchestrator
        .get_follow_up_monitoring(&query.into())
        .map(Json)
        .map_err(error_response)
}

async fn list_handoff_receipts(
    State(state): State<AppState>,
    Query(query): Query<HandoffReceiptsQueryParams>,
) -> Result<Json<Vec<HandoffReceiptQueueItemView>>, (StatusCode, Json<serde_json::Value>)> {
    state
        .orchestrator
        .list_handoff_receipts(&query.into())
        .map(Json)
        .map_err(error_response)
}

async fn list_alert_clusters(
    State(state): State<AppState>,
    Query(query): Query<AlertClustersQueryParams>,
) -> Result<Json<Vec<AlertClusterQueueItemView>>, (StatusCode, Json<serde_json::Value>)> {
    state
        .orchestrator
        .list_alert_clusters(&query.into())
        .map(Json)
        .map_err(error_response)
}

async fn get_alert_cluster_monitoring(
    State(state): State<AppState>,
    Query(query): Query<AlertClustersQueryParams>,
) -> Result<Json<AlertClusterMonitoringView>, (StatusCode, Json<serde_json::Value>)> {
    state
        .orchestrator
        .get_alert_cluster_monitoring(&query.into())
        .map(Json)
        .map_err(error_response)
}

async fn get_handoff_receipt_monitoring(
    State(state): State<AppState>,
    Query(query): Query<HandoffReceiptsQueryParams>,
) -> Result<Json<HandoffReceiptMonitoringView>, (StatusCode, Json<serde_json::Value>)> {
    state
        .orchestrator
        .get_handoff_receipt_monitoring(&query.into())
        .map(Json)
        .map_err(error_response)
}

async fn task_audit_events(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Query(query): Query<AuditEventsQueryParams>,
) -> Result<Json<Vec<fa_core::AuditEvent>>, (StatusCode, Json<serde_json::Value>)> {
    let query = AuditEventQuery {
        task_id: Some(task_id),
        approval_id: query.approval_id,
        correlation_id: query.correlation_id,
        kind: query.kind,
    };

    state.audit_sink.query(&query).map(Json).map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": error.to_string(),
            })),
        )
    })
}

async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TrackedTaskState>, (StatusCode, Json<serde_json::Value>)> {
    state
        .orchestrator
        .get_task(task_id)
        .map(Json)
        .map_err(error_response)
}

async fn task_evidence(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Vec<fa_core::TaskEvidence>>, (StatusCode, Json<serde_json::Value>)> {
    state
        .orchestrator
        .get_task_evidence(task_id)
        .map(Json)
        .map_err(error_response)
}

async fn task_governance(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<fa_domain::WorkflowGovernance>, (StatusCode, Json<serde_json::Value>)> {
    state
        .orchestrator
        .get_task_governance(task_id)
        .map(Json)
        .map_err(error_response)
}

async fn approve_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<ApprovalActionRequest>,
) -> Result<Json<TrackedTaskState>, (StatusCode, Json<serde_json::Value>)> {
    let correlation_id = headers
        .get("x-correlation-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    state
        .orchestrator
        .approve_task(task_id, request, correlation_id)
        .map(Json)
        .map_err(error_response)
}

async fn execute_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<ExecuteTaskRequest>,
) -> Result<Json<TrackedTaskState>, (StatusCode, Json<serde_json::Value>)> {
    let correlation_id = headers
        .get("x-correlation-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    state
        .orchestrator
        .start_execution(task_id, request, correlation_id)
        .map(Json)
        .map_err(error_response)
}

async fn resubmit_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<ResubmitTaskRequest>,
) -> Result<Json<TrackedTaskState>, (StatusCode, Json<serde_json::Value>)> {
    let correlation_id = headers
        .get("x-correlation-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    state
        .orchestrator
        .resubmit_task(task_id, request, correlation_id)
        .map(Json)
        .map_err(error_response)
}

async fn complete_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<CompleteTaskRequest>,
) -> Result<Json<TrackedTaskState>, (StatusCode, Json<serde_json::Value>)> {
    let correlation_id = headers
        .get("x-correlation-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    state
        .orchestrator
        .complete_task(task_id, request, correlation_id)
        .map(Json)
        .map_err(error_response)
}

async fn fail_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<FailTaskRequest>,
) -> Result<Json<TrackedTaskState>, (StatusCode, Json<serde_json::Value>)> {
    let correlation_id = headers
        .get("x-correlation-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    state
        .orchestrator
        .fail_task(task_id, request, correlation_id)
        .map(Json)
        .map_err(error_response)
}

async fn accept_follow_up_owner(
    State(state): State<AppState>,
    Path((task_id, follow_up_id)): Path<(Uuid, String)>,
    headers: HeaderMap,
    Json(request): Json<AcceptFollowUpOwnerRequest>,
) -> Result<Json<TrackedTaskState>, (StatusCode, Json<serde_json::Value>)> {
    let correlation_id = headers
        .get("x-correlation-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    state
        .orchestrator
        .accept_follow_up_owner(task_id, follow_up_id, request, correlation_id)
        .map(Json)
        .map_err(error_response)
}

async fn acknowledge_handoff_receipt(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<AcknowledgeHandoffReceiptRequest>,
) -> Result<Json<TrackedTaskState>, (StatusCode, Json<serde_json::Value>)> {
    let correlation_id = headers
        .get("x-correlation-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    state
        .orchestrator
        .acknowledge_handoff_receipt(task_id, request, correlation_id)
        .map(Json)
        .map_err(error_response)
}

async fn escalate_handoff_receipt(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<EscalateHandoffReceiptRequest>,
) -> Result<Json<TrackedTaskState>, (StatusCode, Json<serde_json::Value>)> {
    let correlation_id = headers
        .get("x-correlation-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    state
        .orchestrator
        .escalate_handoff_receipt(task_id, request, correlation_id)
        .map(Json)
        .map_err(error_response)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};

        let mut signal =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        signal.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fa_server=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn error_response(error: OrchestrationError) -> (StatusCode, Json<serde_json::Value>) {
    let status = match &error {
        OrchestrationError::Lifecycle(LifecycleError::ApprovalRoleMismatch { .. }) => {
            StatusCode::FORBIDDEN
        }
        OrchestrationError::FollowUpRoleMismatch { .. }
        | OrchestrationError::HandoffReceiptRoleMismatch { .. } => StatusCode::FORBIDDEN,
        OrchestrationError::Lifecycle(_) => StatusCode::UNPROCESSABLE_ENTITY,
        OrchestrationError::TaskAlreadyExists(_) => StatusCode::CONFLICT,
        OrchestrationError::TaskNotFound(_)
        | OrchestrationError::ApprovalNotFound(_)
        | OrchestrationError::FollowUpItemNotFound { .. }
        | OrchestrationError::HandoffReceiptNotFound(_) => StatusCode::NOT_FOUND,
        OrchestrationError::InvalidFollowUpItemState { .. }
        | OrchestrationError::InvalidHandoffReceiptState { .. } => StatusCode::UNPROCESSABLE_ENTITY,
        OrchestrationError::TaskRepository(_)
        | OrchestrationError::Connector(_)
        | OrchestrationError::Audit(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (
        status,
        Json(json!({
            "error": error.to_string(),
        })),
    )
}

fn build_state() -> anyhow::Result<AppState> {
    build_state_with_storage(
        env::var_os("FA_SQLITE_DB_PATH").map(PathBuf::from),
        env::var_os("FA_DATA_DIR").map(PathBuf::from),
    )
}

fn build_state_with_storage(
    sqlite_db_path: Option<PathBuf>,
    data_dir: Option<PathBuf>,
) -> anyhow::Result<AppState> {
    if let Some(db_path) = sqlite_db_path {
        let audit_sink = Arc::new(SqliteAuditStore::new(&db_path)?);
        let task_repository = Arc::new(SqliteTaskRepository::new(&db_path)?);

        Ok(AppState {
            orchestrator: WorkOrchestrator::with_m1_defaults_and_repository(
                audit_sink.clone(),
                task_repository,
            ),
            audit_sink,
        })
    } else if let Some(data_dir) = data_dir {
        let audit_sink = Arc::new(FileAuditStore::new(&data_dir)?);
        let task_repository = Arc::new(FileTaskRepository::new(&data_dir)?);

        Ok(AppState {
            orchestrator: WorkOrchestrator::with_m1_defaults_and_repository(
                audit_sink.clone(),
                task_repository,
            ),
            audit_sink,
        })
    } else {
        let audit_sink = Arc::new(InMemoryAuditSink::default());

        Ok(AppState {
            orchestrator: WorkOrchestrator::with_m1_defaults_and_repository(
                audit_sink.clone(),
                Arc::new(InMemoryTaskRepository::default()),
            ),
            audit_sink,
        })
    }
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(experience_shell))
        .route("/assets/fa-experience.css", get(experience_styles))
        .route("/assets/fa-experience.js", get(experience_script))
        .route("/healthz", get(healthz))
        .route("/api/v1/experience/overview", get(experience_overview))
        .route("/api/v1/blueprint", get(blueprint))
        .route("/api/v1/audit/events", get(audit_events))
        .route(
            "/api/v1/follow-up-monitoring",
            get(get_follow_up_monitoring),
        )
        .route("/api/v1/follow-up-items", get(list_follow_up_items))
        .route(
            "/api/v1/handoff-receipt-monitoring",
            get(get_handoff_receipt_monitoring),
        )
        .route("/api/v1/handoff-receipts", get(list_handoff_receipts))
        .route(
            "/api/v1/alert-cluster-monitoring",
            get(get_alert_cluster_monitoring),
        )
        .route("/api/v1/alert-clusters", get(list_alert_clusters))
        .route("/api/v1/tasks/intake", post(intake_task))
        .route("/api/v1/tasks/plan", post(plan_task))
        .route("/api/v1/tasks/{task_id}", get(get_task))
        .route("/api/v1/tasks/{task_id}/evidence", get(task_evidence))
        .route("/api/v1/tasks/{task_id}/governance", get(task_governance))
        .route(
            "/api/v1/tasks/{task_id}/audit-events",
            get(task_audit_events),
        )
        .route("/api/v1/tasks/{task_id}/approve", post(approve_task))
        .route("/api/v1/tasks/{task_id}/resubmit", post(resubmit_task))
        .route("/api/v1/tasks/{task_id}/execute", post(execute_task))
        .route("/api/v1/tasks/{task_id}/complete", post(complete_task))
        .route("/api/v1/tasks/{task_id}/fail", post(fail_task))
        .route(
            "/api/v1/tasks/{task_id}/follow-up-items/{follow_up_id}/accept-owner",
            post(accept_follow_up_owner),
        )
        .route(
            "/api/v1/tasks/{task_id}/handoff-receipt/acknowledge",
            post(acknowledge_handoff_receipt),
        )
        .route(
            "/api/v1/tasks/{task_id}/handoff-receipt/escalate",
            post(escalate_handoff_receipt),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use fa_core::TaskRepository;
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;

    const TASK_ID: &str = "72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0";
    const SHIFT_TASK_ID: &str = "72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f1";
    const ALERT_TASK_ID: &str = "72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f2";
    const SHIFT_TASK_ID_2: &str = "72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f3";
    const SHIFT_TASK_ID_3: &str = "72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f4";
    const SHIFT_TASK_ID_4: &str = "72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f5";
    const ALERT_TASK_ID_2: &str = "72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f6";
    const ALERT_TASK_ID_3: &str = "72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f7";

    fn high_risk_request() -> String {
        format!(
            r#"{{
                "id":"{TASK_ID}",
                "title":"Investigate spindle temperature drift",
                "description":"Diagnose repeated spindle temperature drift before the next shift.",
                "priority":"critical",
                "risk":"high",
                "initiator":{{"id":"worker_1001","display_name":"Liu Supervisor","role":"Production Supervisor"}},
                "stakeholders":[],
                "equipment_ids":["eq_cnc_01"],
                "integrations":["mes","cmms"],
                "desired_outcome":"Recover stable spindle temperature within tolerance",
                "requires_human_approval":true,
                "requires_diagnostic_loop":true
            }}"#
        )
    }

    fn shift_handoff_request() -> String {
        shift_handoff_request_with_id(
            SHIFT_TASK_ID,
            "Summarize shift notes",
            "Summarize shift notes for morning handoff.",
        )
    }

    fn shift_handoff_request_with_id(task_id: &str, title: &str, description: &str) -> String {
        format!(
            r#"{{
                "id":"{task_id}",
                "title":"{title}",
                "description":"{description}",
                "priority":"routine",
                "risk":"low",
                "initiator":{{"id":"worker_1001","display_name":"Liu Supervisor","role":"Production Supervisor"}},
                "stakeholders":[],
                "equipment_ids":[],
                "integrations":["mes"],
                "desired_outcome":"Publish a clean handoff summary with pending items",
                "requires_human_approval":false,
                "requires_diagnostic_loop":false
            }}"#
        )
    }

    fn alert_triage_request() -> String {
        format!(
            r#"{{
                "id":"{ALERT_TASK_ID}",
                "title":"Triage repeated andon alerts on pack line 4",
                "description":"Review repeated alert burst and cluster similar signals before escalation.",
                "priority":"expedited",
                "risk":"high",
                "initiator":{{"id":"worker_1001","display_name":"Liu Supervisor","role":"Production Supervisor"}},
                "stakeholders":[],
                "equipment_ids":["eq_pack_04"],
                "integrations":["mes"],
                "desired_outcome":"Create a triage-ready alert cluster and route it to the production supervisor.",
                "requires_human_approval":false,
                "requires_diagnostic_loop":false
            }}"#
        )
    }

    fn scada_threshold_alert_request() -> String {
        scada_threshold_alert_request_with_id(
            ALERT_TASK_ID_2,
            "Triage sustained temperature alert on mix line 2",
            "Review sustained SCADA threshold breach and sensor drift on mix line 2 before escalation.",
            "eq_mix_02",
        )
    }

    fn scada_threshold_alert_request_with_id(
        task_id: &str,
        title: &str,
        description: &str,
        equipment_id: &str,
    ) -> String {
        format!(
            r#"{{
                "id":"{task_id}",
                "title":"{title}",
                "description":"{description}",
                "priority":"expedited",
                "risk":"medium",
                "initiator":{{"id":"worker_1001","display_name":"Liu Supervisor","role":"Production Supervisor"}},
                "stakeholders":[],
                "equipment_ids":["{equipment_id}"],
                "integrations":["scada"],
                "desired_outcome":"Cluster sustained threshold signals and route first diagnostic review to maintenance.",
                "requires_human_approval":false,
                "requires_diagnostic_loop":false
            }}"#
        )
    }

    fn handoff_ack_request() -> String {
        r#"{
                "actor":{"id":"worker_1101","display_name":"Zhang Incoming","role":"Incoming Shift Supervisor"},
                "exception_note":null
            }"#
        .to_string()
    }

    fn shift_follow_up_accept_request() -> String {
        r#"{
                "actor":{"id":"worker_1101","display_name":"Zhang Incoming","role":"Incoming Shift Supervisor"},
                "note":"Incoming shift supervisor accepts remaining work ownership."
            }"#
        .to_string()
    }

    fn alert_follow_up_accept_request() -> String {
        r#"{
                "actor":{"id":"worker_1001","display_name":"Liu Supervisor","role":"Production Supervisor"},
                "note":"Production supervisor accepts first response ownership."
            }"#
        .to_string()
    }

    fn handoff_ack_with_exception_request() -> String {
        r#"{
                "actor":{"id":"worker_1101","display_name":"Zhang Incoming","role":"Incoming Shift Supervisor"},
                "exception_note":"Need clarification for one unresolved packaging stop before accepting all items."
            }"#
        .to_string()
    }

    fn handoff_escalation_request() -> String {
        r#"{
                "actor":{"id":"worker_1001","display_name":"Liu Supervisor","role":"Production Supervisor"},
                "note":"Escalate to day-shift review before startup release."
            }"#
        .to_string()
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .components()
            .collect()
    }

    fn sandbox_root() -> PathBuf {
        std::env::var_os("FA_SANDBOX_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| repo_root().join("sandbox"))
    }

    struct SandboxDirGuard {
        path: PathBuf,
        keep: bool,
    }

    impl SandboxDirGuard {
        fn new(prefix: &str) -> Self {
            let root = sandbox_root();
            fs::create_dir_all(&root).expect("sandbox root should be creatable");
            let path = root.join(format!("{prefix}-{}", Uuid::new_v4()));
            fs::create_dir_all(&path).expect("sandbox data dir should be creatable");

            Self {
                path,
                keep: std::env::var_os("FA_KEEP_SANDBOX").is_some(),
            }
        }

        fn path(&self) -> &PathBuf {
            &self.path
        }
    }

    impl Drop for SandboxDirGuard {
        fn drop(&mut self) {
            if !self.keep {
                let _ = fs::remove_dir_all(&self.path);
            }
        }
    }

    fn build_file_backed_app(data_dir: PathBuf) -> Router {
        app(build_state_with_storage(None, Some(data_dir)).expect("state should build"))
    }

    fn build_in_memory_app_with_repository() -> (Router, Arc<InMemoryTaskRepository>) {
        let audit_sink: Arc<dyn AuditStore> = Arc::new(InMemoryAuditSink::default());
        let task_repository = Arc::new(InMemoryTaskRepository::default());
        let orchestrator = WorkOrchestrator::with_m1_defaults_and_repository(
            audit_sink.clone(),
            task_repository.clone(),
        );

        (
            app(AppState {
                orchestrator,
                audit_sink,
            }),
            task_repository,
        )
    }

    fn build_file_backed_app_with_repository(
        data_dir: PathBuf,
    ) -> (Router, Arc<FileTaskRepository>) {
        let audit_sink: Arc<dyn AuditStore> =
            Arc::new(FileAuditStore::new(&data_dir).expect("audit store should build"));
        let task_repository =
            Arc::new(FileTaskRepository::new(&data_dir).expect("task repository should build"));
        let orchestrator = WorkOrchestrator::with_m1_defaults_and_repository(
            audit_sink.clone(),
            task_repository.clone(),
        );

        (
            app(AppState {
                orchestrator,
                audit_sink,
            }),
            task_repository,
        )
    }

    async fn json_body(response: axum::response::Response) -> serde_json::Value {
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should read");
        serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            panic!("failed to decode JSON body with status {status}: {error}")
        })
    }

    async fn text_body(response: axum::response::Response) -> String {
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should read");
        String::from_utf8(bytes.to_vec()).unwrap_or_else(|error| {
            panic!("failed to decode text body with status {status}: {error}")
        })
    }

    fn json_bucket_count(value: &serde_json::Value, field: &str, key: &str) -> usize {
        value[field]
            .as_array()
            .and_then(|buckets| {
                buckets.iter().find_map(|bucket| {
                    (bucket["key"].as_str() == Some(key))
                        .then(|| bucket["count"].as_u64())
                        .flatten()
                })
            })
            .and_then(|count| usize::try_from(count).ok())
            .unwrap_or_default()
    }

    #[tokio::test]
    async fn experience_shell_route_returns_html() {
        let response = app(build_state_with_storage(None, None).expect("state should build"))
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should build");

        assert_eq!(response.status(), StatusCode::OK);
        let body = text_body(response).await;
        assert!(body.contains("FA 体验指挥中心"));
        assert!(body.contains("启动演示批次"));
    }

    #[tokio::test]
    async fn experience_overview_returns_aggregated_monitoring_and_queue_preview() {
        let app = build_in_memory_app_with_repository().0;

        let shift_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks/intake")
                    .header("content-type", "application/json")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("response should build");
        assert_eq!(shift_response.status(), StatusCode::OK);

        let alert_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks/intake")
                    .header("content-type", "application/json")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("response should build");
        assert_eq!(alert_response.status(), StatusCode::OK);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/experience/overview")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should build");

        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["service"]["status"], "ok");
        assert!(
            body["blueprint"]["pattern_count"]
                .as_u64()
                .expect("pattern_count should be numeric")
                >= 1
        );
        assert!(
            body["monitoring"]["follow_up"]["open_items"]
                .as_u64()
                .expect("open_items should be numeric")
                >= 1
        );
        assert!(
            body["monitoring"]["handoff"]["total_receipts"]
                .as_u64()
                .expect("total_receipts should be numeric")
                >= 1
        );
        assert!(
            body["monitoring"]["alert_cluster"]["total_clusters"]
                .as_u64()
                .expect("total_clusters should be numeric")
                >= 1
        );
        assert_eq!(
            body["queues"]["follow_up_items"]
                .as_array()
                .expect("follow_up_items should be an array")
                .len(),
            2
        );
        assert_eq!(
            body["queues"]["handoff_receipts"]
                .as_array()
                .expect("handoff_receipts should be an array")
                .len(),
            1
        );
        assert_eq!(
            body["queues"]["alert_clusters"]
                .as_array()
                .expect("alert_clusters should be an array")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn lifecycle_happy_path_works_end_to_end() {
        let app = app(build_state().expect("state should build"));

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-intake-001")
                    .body(Body::from(high_risk_request()))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);
        let intake_json = json_body(intake_response).await;
        assert_eq!(
            intake_json["planned_task"]["task"]["status"],
            "awaiting_approval"
        );
        assert_eq!(
            intake_json["evidence"]
                .as_array()
                .expect("evidence list")
                .len(),
            4
        );
        assert!(intake_json["follow_up_items"]
            .as_array()
            .expect("follow-up list")
            .is_empty());
        assert_eq!(intake_json["follow_up_summary"]["total_items"], 0);
        assert!(intake_json["handoff_receipt"].is_null());
        assert_eq!(
            intake_json["handoff_receipt_summary"]["covered_follow_up_count"],
            0
        );
        assert!(intake_json["alert_cluster_drafts"]
            .as_array()
            .expect("alert cluster list")
            .is_empty());
        assert_eq!(intake_json["alert_triage_summary"]["total_clusters"], 0);

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_json = json_body(get_response).await;
        assert_eq!(get_json["planned_task"]["task"]["id"], TASK_ID);
        assert_eq!(
            get_json["evidence"]
                .as_array()
                .expect("evidence list")
                .len(),
            4
        );
        assert!(get_json["follow_up_items"]
            .as_array()
            .expect("follow-up list")
            .is_empty());
        assert_eq!(get_json["follow_up_summary"]["open_items"], 0);
        assert!(get_json["handoff_receipt"].is_null());
        assert_eq!(
            get_json["handoff_receipt_summary"]["unaccepted_follow_up_count"],
            0
        );
        assert!(get_json["alert_cluster_drafts"]
            .as_array()
            .expect("alert cluster list")
            .is_empty());
        assert_eq!(get_json["alert_triage_summary"]["open_clusters"], 0);

        let evidence_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/evidence"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(evidence_response.status(), StatusCode::OK);
        let evidence_json = json_body(evidence_response).await;
        let evidence_items = evidence_json.as_array().expect("evidence list");
        assert_eq!(evidence_items.len(), 4);
        assert!(evidence_items
            .iter()
            .any(|item| item["summary"].as_str().unwrap_or("").contains("telemetry")));

        let governance_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/governance"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(governance_response.status(), StatusCode::OK);
        let governance_json = json_body(governance_response).await;
        assert_eq!(
            governance_json["approval_strategy"]["required_role"],
            "safety_officer"
        );
        assert!(governance_json["responsibility_matrix"]
            .as_array()
            .expect("responsibility list")
            .iter()
            .any(|item| item["role"] == "maintenance_engineer"));

        let approve_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/approve"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-approve-001")
                    .body(Body::from(
                        r#"{
                            "decided_by":{"id":"worker_2001","display_name":"Wang Safety","role":"Safety Officer"},
                            "approved":true,
                            "comment":"Proceed to execution"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(approve_response.status(), StatusCode::OK);
        let approve_json = json_body(approve_response).await;
        assert_eq!(approve_json["planned_task"]["task"]["status"], "approved");

        let execute_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/execute"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-execute-001")
                    .body(Body::from(
                        r#"{
                            "actor":{"id":"worker_3001","display_name":"Wu Maint","role":"Maintenance Technician"},
                            "note":"Execution stub started"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(execute_response.status(), StatusCode::OK);
        let execute_json = json_body(execute_response).await;
        assert_eq!(execute_json["planned_task"]["task"]["status"], "executing");

        let complete_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/complete"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-complete-001")
                    .body(Body::from(
                        r#"{
                            "actor":{"id":"worker_3001","display_name":"Wu Maint","role":"Maintenance Technician"},
                            "note":"Execution finished"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(complete_response.status(), StatusCode::OK);
        let complete_json = json_body(complete_response).await;
        assert_eq!(complete_json["planned_task"]["task"]["status"], "completed");

        let audit_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/audit/events")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(audit_response.status(), StatusCode::OK);
        let audit_json = json_body(audit_response).await;
        assert!(audit_json.as_array().expect("audit list").len() >= 10);
    }

    #[tokio::test]
    async fn fail_endpoint_marks_task_failed() {
        let app = app(build_state().expect("state should build"));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-intake-002")
                    .body(Body::from(high_risk_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/approve"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-approve-002")
                    .body(Body::from(
                        r#"{
                            "decided_by":{"id":"worker_2001","display_name":"Wang Safety","role":"Safety Officer"},
                            "approved":true,
                            "comment":"Proceed to execution"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("approval should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/execute"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-execute-002")
                    .body(Body::from(
                        r#"{
                            "actor":{"id":"worker_3001","display_name":"Wu Maint","role":"Maintenance Technician"},
                            "note":"Execution stub started"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("execute should succeed");

        let fail_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/fail"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-fail-001")
                    .body(Body::from(
                        r#"{
                            "actor":{"id":"worker_3001","display_name":"Wu Maint","role":"Maintenance Technician"},
                            "reason":"Cooling loop inspection failed"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("fail should succeed");

        assert_eq!(fail_response.status(), StatusCode::OK);
        let fail_json = json_body(fail_response).await;
        assert_eq!(fail_json["planned_task"]["task"]["status"], "failed");
        assert_eq!(
            fail_json["planned_task"]["task"]["latest_error"],
            "Cooling loop inspection failed"
        );
    }

    #[tokio::test]
    async fn shift_handoff_intake_returns_seeded_follow_up_item() {
        let app = app(build_state().expect("state should build"));

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-shift-handoff-001")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);
        let intake_json = json_body(intake_response).await;
        assert_eq!(intake_json["planned_task"]["task"]["status"], "approved");
        assert_eq!(
            intake_json["follow_up_items"]
                .as_array()
                .expect("follow-up list")
                .len(),
            1
        );
        assert_eq!(
            intake_json["follow_up_items"][0]["source_kind"],
            "shift_handoff"
        );
        assert_eq!(
            intake_json["follow_up_items"][0]["recommended_owner_role"],
            "incoming_shift_supervisor"
        );
        assert_eq!(intake_json["follow_up_summary"]["total_items"], 1);
        assert_eq!(intake_json["follow_up_summary"]["open_items"], 1);
        assert_eq!(intake_json["handoff_receipt"]["status"], "published");
        assert_eq!(
            intake_json["handoff_receipt"]["receiving_role"],
            "incoming_shift_supervisor"
        );
        assert_eq!(
            intake_json["handoff_receipt_summary"]["covered_follow_up_count"],
            1
        );
        assert_eq!(
            intake_json["handoff_receipt_summary"]["unaccepted_follow_up_count"],
            1
        );

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{SHIFT_TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_json = json_body(get_response).await;
        assert_eq!(
            get_json["follow_up_items"]
                .as_array()
                .expect("follow-up list")
                .len(),
            1
        );
        assert_eq!(get_json["follow_up_summary"]["total_items"], 1);
        assert_eq!(get_json["handoff_receipt"]["status"], "published");
        assert_eq!(
            get_json["handoff_receipt_summary"]["covered_follow_up_count"],
            1
        );
    }

    #[tokio::test]
    async fn shift_handoff_follow_up_accept_owner_endpoint_updates_task_state() {
        let app = app(build_state().expect("state should build"));

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-accept-intake-001")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);
        let intake_json = json_body(intake_response).await;
        let follow_up_id = intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let accept_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/follow-up-items/{follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-accept-001")
                    .body(Body::from(shift_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up accept should succeed");
        assert_eq!(accept_response.status(), StatusCode::OK);
        let accept_json = json_body(accept_response).await;
        assert_eq!(accept_json["follow_up_items"][0]["status"], "accepted");
        assert_eq!(
            accept_json["follow_up_items"][0]["accepted_owner_id"],
            "worker_1101"
        );
        assert_eq!(
            accept_json["handoff_receipt_summary"]["unaccepted_follow_up_count"],
            0
        );

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{SHIFT_TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_json = json_body(get_response).await;
        assert_eq!(get_json["follow_up_items"][0]["status"], "accepted");
        assert_eq!(
            get_json["handoff_receipt_summary"]["unaccepted_follow_up_count"],
            0
        );

        let audit_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/audit-events?kind=follow_up_owner_accepted"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("audit query should succeed");
        assert_eq!(audit_response.status(), StatusCode::OK);
        let audit_json = json_body(audit_response).await;
        assert_eq!(audit_json.as_array().expect("audit list").len(), 1);
    }

    #[tokio::test]
    async fn shift_handoff_follow_up_accept_owner_endpoint_rejects_wrong_role() {
        let app = app(build_state().expect("state should build"));

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-accept-intake-002")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");
        let intake_json = json_body(intake_response).await;
        let follow_up_id = intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let accept_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/follow-up-items/{follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-accept-002")
                    .body(Body::from(alert_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up accept should return response");
        assert_eq!(accept_response.status(), StatusCode::FORBIDDEN);
        let accept_json = json_body(accept_response).await;
        assert_eq!(
            accept_json["error"],
            format!(
                "follow-up item '{follow_up_id}' requires role 'incoming_shift_supervisor', got 'production_supervisor'"
            )
        );
    }

    #[tokio::test]
    async fn shift_handoff_follow_up_accept_owner_endpoint_rejects_already_accepted_item() {
        let app = app(build_state().expect("state should build"));

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-accept-intake-003")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");
        let intake_json = json_body(intake_response).await;
        let follow_up_id = intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/follow-up-items/{follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-accept-003")
                    .body(Body::from(shift_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("first follow-up accept should succeed");

        let accept_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/follow-up-items/{follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-accept-004")
                    .body(Body::from(shift_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("second follow-up accept should return response");
        assert_eq!(accept_response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let accept_json = json_body(accept_response).await;
        assert_eq!(
            accept_json["error"],
            format!(
                "follow-up item '{follow_up_id}' for task {SHIFT_TASK_ID} cannot transition from status 'accepted'"
            )
        );
    }

    #[tokio::test]
    async fn shift_handoff_receipt_acknowledge_endpoint_updates_receipt_state() {
        let app = app(build_state().expect("state should build"));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-shift-handoff-ack-intake-001")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");

        let acknowledge_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-shift-handoff-ack-001")
                    .body(Body::from(handoff_ack_with_exception_request()))
                    .expect("request should build"),
            )
            .await
            .expect("acknowledge should succeed");
        assert_eq!(acknowledge_response.status(), StatusCode::OK);
        let acknowledge_json = json_body(acknowledge_response).await;
        assert_eq!(
            acknowledge_json["handoff_receipt"]["status"],
            "acknowledged_with_exceptions"
        );
        assert_eq!(
            acknowledge_json["handoff_receipt"]["receiving_actor"]["id"],
            "worker_1101"
        );
        assert_eq!(
            acknowledge_json["handoff_receipt_summary"]["status"],
            "acknowledged_with_exceptions"
        );
        assert_eq!(
            acknowledge_json["handoff_receipt_summary"]["exception_flag"],
            true
        );
        assert!(acknowledge_json["handoff_receipt_summary"]["acknowledged_at"].is_string());

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{SHIFT_TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_json = json_body(get_response).await;
        assert_eq!(
            get_json["handoff_receipt"]["status"],
            "acknowledged_with_exceptions"
        );

        let audit_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/audit-events?kind=handoff_acknowledged_with_exceptions"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("audit query should succeed");
        assert_eq!(audit_response.status(), StatusCode::OK);
        let audit_json = json_body(audit_response).await;
        assert_eq!(audit_json.as_array().expect("audit list").len(), 1);
    }

    #[tokio::test]
    async fn shift_handoff_receipt_acknowledge_endpoint_rejects_wrong_role() {
        let app = app(build_state().expect("state should build"));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-shift-handoff-ack-intake-002")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");

        let acknowledge_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-shift-handoff-ack-002")
                    .body(Body::from(
                        r#"{
                            "actor":{"id":"worker_2002","display_name":"Chen QE","role":"Quality Engineer"},
                            "exception_note":null
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("acknowledge should return response");
        assert_eq!(acknowledge_response.status(), StatusCode::FORBIDDEN);
        let acknowledge_json = json_body(acknowledge_response).await;
        assert_eq!(
            acknowledge_json["error"],
            "handoff receipt requires role 'incoming_shift_supervisor', got 'quality_engineer'"
        );

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{SHIFT_TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        let get_json = json_body(get_response).await;
        assert_eq!(get_json["handoff_receipt"]["status"], "published");
    }

    #[tokio::test]
    async fn shift_handoff_receipt_escalate_endpoint_updates_receipt_state() {
        let app = app(build_state().expect("state should build"));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-shift-handoff-escalate-intake-001",
                    )
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-shift-handoff-escalate-ack-001")
                    .body(Body::from(handoff_ack_with_exception_request()))
                    .expect("request should build"),
            )
            .await
            .expect("acknowledge should succeed");

        let escalate_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/handoff-receipt/escalate"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-shift-handoff-escalate-001")
                    .body(Body::from(handoff_escalation_request()))
                    .expect("request should build"),
            )
            .await
            .expect("escalate should succeed");
        assert_eq!(escalate_response.status(), StatusCode::OK);
        let escalate_json = json_body(escalate_response).await;
        assert_eq!(escalate_json["handoff_receipt"]["status"], "escalated");
        assert_eq!(
            escalate_json["handoff_receipt"]["escalation_state"],
            "escalated"
        );
        assert_eq!(
            escalate_json["handoff_receipt_summary"]["status"],
            "escalated"
        );
        assert_eq!(
            escalate_json["handoff_receipt_summary"]["exception_flag"],
            true
        );

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{SHIFT_TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_json = json_body(get_response).await;
        assert_eq!(get_json["handoff_receipt"]["status"], "escalated");

        let audit_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/audit-events?kind=handoff_receipt_escalated"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("audit query should succeed");
        assert_eq!(audit_response.status(), StatusCode::OK);
        let audit_json = json_body(audit_response).await;
        assert_eq!(audit_json.as_array().expect("audit list").len(), 1);
    }

    #[tokio::test]
    async fn shift_handoff_receipt_escalate_endpoint_rejects_wrong_state() {
        let app = app(build_state().expect("state should build"));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-shift-handoff-escalate-intake-002",
                    )
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-shift-handoff-escalate-ack-002")
                    .body(Body::from(handoff_ack_request()))
                    .expect("request should build"),
            )
            .await
            .expect("acknowledge should succeed");

        let escalate_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/handoff-receipt/escalate"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-shift-handoff-escalate-002")
                    .body(Body::from(handoff_escalation_request()))
                    .expect("request should build"),
            )
            .await
            .expect("escalate should return response");
        assert_eq!(escalate_response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let escalate_json = json_body(escalate_response).await;
        assert!(escalate_json["error"]
            .as_str()
            .expect("error string")
            .contains("cannot transition from status 'acknowledged'"));
    }

    #[tokio::test]
    async fn shift_handoff_receipt_escalate_endpoint_rejects_wrong_role() {
        let app = app(build_state().expect("state should build"));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-shift-handoff-escalate-intake-003",
                    )
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-shift-handoff-escalate-ack-003")
                    .body(Body::from(handoff_ack_with_exception_request()))
                    .expect("request should build"),
            )
            .await
            .expect("acknowledge should succeed");

        let escalate_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/handoff-receipt/escalate"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-shift-handoff-escalate-003")
                    .body(Body::from(
                        r#"{
                            "actor":{"id":"worker_1101","display_name":"Zhang Incoming","role":"Incoming Shift Supervisor"},
                            "note":"Trying to escalate from receiving role"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("escalate should return response");
        assert_eq!(escalate_response.status(), StatusCode::FORBIDDEN);
        let escalate_json = json_body(escalate_response).await;
        assert_eq!(
            escalate_json["error"],
            "handoff receipt requires role 'production_supervisor', got 'incoming_shift_supervisor'"
        );
    }

    #[tokio::test]
    async fn alert_triage_intake_returns_seeded_alert_cluster_and_follow_up() {
        let app = app(build_state().expect("state should build"));

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-alert-triage-001")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);
        let intake_json = json_body(intake_response).await;
        assert_eq!(
            intake_json["planned_task"]["task"]["status"],
            "awaiting_approval"
        );
        assert_eq!(
            intake_json["follow_up_items"]
                .as_array()
                .expect("follow-up list")
                .len(),
            1
        );
        assert_eq!(
            intake_json["follow_up_items"][0]["source_kind"],
            "alert_triage"
        );
        assert_eq!(
            intake_json["follow_up_items"][0]["recommended_owner_role"],
            "production_supervisor"
        );
        assert_eq!(intake_json["follow_up_summary"]["total_items"], 1);
        assert_eq!(intake_json["follow_up_summary"]["open_items"], 1);
        assert!(intake_json["handoff_receipt"].is_null());
        assert_eq!(
            intake_json["alert_cluster_drafts"]
                .as_array()
                .expect("alert cluster list")
                .len(),
            1
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["cluster_status"],
            "open"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["source_system"],
            "andon"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["line_id"],
            "line_pack_04"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["triage_label"],
            "repeated_alert_review"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["recommended_owner_role"],
            "production_supervisor"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["severity_band"],
            "high"
        );
        assert_eq!(intake_json["alert_triage_summary"]["total_clusters"], 1);
        assert_eq!(intake_json["alert_triage_summary"]["open_clusters"], 1);
        assert_eq!(
            intake_json["alert_triage_summary"]["high_priority_clusters"],
            1
        );
        assert_eq!(
            intake_json["alert_triage_summary"]["escalation_candidate_count"],
            1
        );

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{ALERT_TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_json = json_body(get_response).await;
        assert_eq!(
            get_json["alert_cluster_drafts"]
                .as_array()
                .expect("alert cluster list")
                .len(),
            1
        );
        assert_eq!(
            get_json["alert_cluster_drafts"][0]["cluster_status"],
            "open"
        );
        assert_eq!(
            get_json["alert_cluster_drafts"][0]["line_id"],
            "line_pack_04"
        );
        assert_eq!(
            get_json["follow_up_items"]
                .as_array()
                .expect("follow-up list")
                .len(),
            1
        );
        assert_eq!(get_json["follow_up_summary"]["total_items"], 1);
        assert_eq!(get_json["alert_triage_summary"]["total_clusters"], 1);
        assert_eq!(get_json["alert_triage_summary"]["open_clusters"], 1);
    }

    #[tokio::test]
    async fn alert_triage_intake_infers_scada_threshold_cluster_shape() {
        let app = app(build_state().expect("state should build"));

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-alert-triage-002")
                    .body(Body::from(scada_threshold_alert_request()))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);
        let intake_json = json_body(intake_response).await;
        assert_eq!(intake_json["planned_task"]["task"]["status"], "approved");
        assert_eq!(
            intake_json["follow_up_items"][0]["recommended_owner_role"],
            "maintenance_engineer"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["source_system"],
            "scada"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["line_id"],
            "line_mix_02"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["triage_label"],
            "sustained_threshold_review"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["recommended_owner_role"],
            "maintenance_engineer"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["source_event_refs"][0],
            format!("scada://cluster/{}", ALERT_TASK_ID_2.replace('-', ""))
        );
        let window_start = DateTime::parse_from_rfc3339(
            intake_json["alert_cluster_drafts"][0]["window_start"]
                .as_str()
                .expect("window_start should exist"),
        )
        .expect("window_start should parse")
        .with_timezone(&Utc);
        let window_end = DateTime::parse_from_rfc3339(
            intake_json["alert_cluster_drafts"][0]["window_end"]
                .as_str()
                .expect("window_end should exist"),
        )
        .expect("window_end should parse")
        .with_timezone(&Utc);
        assert_eq!(window_end - window_start, chrono::Duration::minutes(15));
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["severity_band"],
            "medium"
        );
        assert_eq!(
            intake_json["alert_triage_summary"]["high_priority_clusters"],
            0
        );
        assert_eq!(
            intake_json["alert_triage_summary"]["escalation_candidate_count"],
            0
        );

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{ALERT_TASK_ID_2}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_json = json_body(get_response).await;
        assert_eq!(
            get_json["alert_cluster_drafts"][0]["source_system"],
            "scada"
        );
        assert_eq!(
            get_json["alert_cluster_drafts"][0]["line_id"],
            "line_mix_02"
        );
        assert_eq!(
            get_json["alert_cluster_drafts"][0]["triage_label"],
            "sustained_threshold_review"
        );
    }

    #[tokio::test]
    async fn alert_triage_follow_up_accept_owner_endpoint_updates_task_state() {
        let app = app(build_state().expect("state should build"));

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-alert-follow-up-accept-intake-001",
                    )
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");
        let intake_json = json_body(intake_response).await;
        let follow_up_id = intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let accept_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{ALERT_TASK_ID}/follow-up-items/{follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-alert-follow-up-accept-001")
                    .body(Body::from(alert_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up accept should succeed");
        assert_eq!(accept_response.status(), StatusCode::OK);
        let accept_json = json_body(accept_response).await;
        assert_eq!(accept_json["follow_up_items"][0]["status"], "accepted");
        assert_eq!(
            accept_json["follow_up_items"][0]["accepted_owner_id"],
            "worker_1001"
        );
        assert_eq!(
            accept_json["alert_cluster_drafts"][0]["cluster_status"],
            "open"
        );

        let audit_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{ALERT_TASK_ID}/audit-events?kind=follow_up_owner_accepted"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("audit query should succeed");
        assert_eq!(audit_response.status(), StatusCode::OK);
        let audit_json = json_body(audit_response).await;
        assert_eq!(audit_json.as_array().expect("audit list").len(), 1);
    }

    #[tokio::test]
    async fn follow_up_items_queue_endpoint_returns_cross_task_items_sorted_by_due_at() {
        let app = app(build_state().expect("state should build"));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-queue-intake-001")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("shift handoff intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-queue-intake-002")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("alert triage intake should succeed");

        let queue_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-items")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("queue request should succeed");
        assert_eq!(queue_response.status(), StatusCode::OK);
        let queue_json = json_body(queue_response).await;
        let items = queue_json.as_array().expect("queue list");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["task_id"], ALERT_TASK_ID);
        assert_eq!(items[0]["source_kind"], "alert_triage");
        assert_eq!(items[0]["task_status"], "awaiting_approval");
        assert_eq!(items[0]["effective_sla_status"], "due_soon");
        assert_eq!(items[1]["task_id"], SHIFT_TASK_ID);
        assert_eq!(items[1]["source_kind"], "shift_handoff");
        assert_eq!(items[1]["task_status"], "approved");
        assert_eq!(items[1]["effective_sla_status"], "due_soon");
    }

    #[tokio::test]
    async fn follow_up_items_queue_endpoint_filters_by_owner_and_source_kind() {
        let app = app(build_state().expect("state should build"));

        let shift_intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-queue-intake-003")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("shift handoff intake should succeed");
        let shift_intake_json = json_body(shift_intake_response).await;
        let follow_up_id = shift_intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-queue-intake-004")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("alert triage intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/follow-up-items/{follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-queue-accept-001")
                    .body(Body::from(shift_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up acceptance should succeed");

        let owner_queue_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-items?owner_id=worker_1101")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("owner queue request should succeed");
        assert_eq!(owner_queue_response.status(), StatusCode::OK);
        let owner_queue_json = json_body(owner_queue_response).await;
        let owner_items = owner_queue_json.as_array().expect("owner queue list");

        assert_eq!(owner_items.len(), 1);
        assert_eq!(owner_items[0]["task_id"], SHIFT_TASK_ID);
        assert_eq!(owner_items[0]["status"], "accepted");
        assert_eq!(owner_items[0]["accepted_owner_id"], "worker_1101");

        let source_queue_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-items?source_kind=alert_triage")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("source queue request should succeed");
        assert_eq!(source_queue_response.status(), StatusCode::OK);
        let source_queue_json = json_body(source_queue_response).await;
        let source_items = source_queue_json.as_array().expect("source queue list");

        assert_eq!(source_items.len(), 1);
        assert_eq!(source_items[0]["task_id"], ALERT_TASK_ID);
        assert_eq!(source_items[0]["source_kind"], "alert_triage");
        assert!(source_items[0]["accepted_owner_id"].is_null());
    }

    #[tokio::test]
    async fn follow_up_items_queue_endpoint_filters_by_risk_priority_and_due_before() {
        let app = app(build_state().expect("state should build"));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-queue-intake-005")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("shift handoff intake should succeed");

        let alert_intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-queue-intake-006")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("alert triage intake should succeed");
        let alert_intake_json = json_body(alert_intake_response).await;
        let due_before = alert_intake_json["follow_up_items"][0]["due_at"]
            .as_str()
            .expect("alert triage due_at should exist");

        let queue_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/follow-up-items?risk=high&priority=expedited&due_before={due_before}"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("risk and priority queue request should succeed");
        assert_eq!(queue_response.status(), StatusCode::OK);
        let queue_json = json_body(queue_response).await;
        let items = queue_json.as_array().expect("queue list");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["task_id"], ALERT_TASK_ID);
        assert_eq!(items[0]["task_risk"], "high");
        assert_eq!(items[0]["task_priority"], "expedited");
        assert_eq!(items[0]["due_at"], due_before);
    }

    #[tokio::test]
    async fn follow_up_items_queue_endpoint_filters_by_blocked_and_escalation_flags() {
        let (app, repository) = build_in_memory_app_with_repository();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-queue-intake-007")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("shift handoff intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-queue-intake-008")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("alert triage intake should succeed");

        let now = Utc::now();

        let mut shift_state = repository
            .get(Uuid::parse_str(SHIFT_TASK_ID).expect("shift task id should parse"))
            .expect("shift task lookup should succeed")
            .expect("shift task should exist");
        shift_state.follow_up_items[0].status = "blocked".to_string();
        shift_state.follow_up_items[0].blocked_reason =
            Some("Waiting for outgoing shift clarification.".to_string());
        shift_state.follow_up_items[0].due_at = Some(now + chrono::Duration::minutes(20));
        repository
            .save(shift_state)
            .expect("shift task save should succeed");

        let mut alert_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID).expect("alert task id should parse"))
            .expect("alert task lookup should succeed")
            .expect("alert task should exist");
        alert_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        alert_state.follow_up_items[0].due_at = Some(now + chrono::Duration::minutes(10));
        repository
            .save(alert_state)
            .expect("alert task save should succeed");

        let blocked_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-items?blocked_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("blocked queue request should succeed");
        assert_eq!(blocked_response.status(), StatusCode::OK);
        let blocked_json = json_body(blocked_response).await;
        let blocked_items = blocked_json.as_array().expect("blocked queue list");

        assert_eq!(blocked_items.len(), 1);
        assert_eq!(blocked_items[0]["task_id"], SHIFT_TASK_ID);
        assert_eq!(blocked_items[0]["status"], "blocked");
        assert_eq!(
            blocked_items[0]["blocked_reason"],
            "Waiting for outgoing shift clarification."
        );

        let escalation_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-items?escalation_required=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("escalation queue request should succeed");
        assert_eq!(escalation_response.status(), StatusCode::OK);
        let escalation_json = json_body(escalation_response).await;
        let escalation_items = escalation_json.as_array().expect("escalation queue list");

        assert_eq!(escalation_items.len(), 1);
        assert_eq!(escalation_items[0]["task_id"], ALERT_TASK_ID);
        assert_eq!(
            escalation_items[0]["effective_sla_status"],
            "escalation_required"
        );
        assert_eq!(escalation_items[0]["overdue"], true);
    }

    #[tokio::test]
    async fn follow_up_monitoring_endpoint_returns_aggregated_and_filtered_views() {
        let (app, repository) = build_in_memory_app_with_repository();

        let shift_intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-monitoring-intake-001")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("shift handoff intake should succeed");
        let shift_intake_json = json_body(shift_intake_response).await;
        let follow_up_id = shift_intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-monitoring-intake-002")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("alert triage intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/follow-up-items/{follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-follow-up-monitoring-accept-001")
                    .body(Body::from(shift_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up acceptance should succeed");

        let now = Utc::now();
        let expected_next_due_at = now + chrono::Duration::minutes(10);

        let mut shift_state = repository
            .get(Uuid::parse_str(SHIFT_TASK_ID).expect("shift task id should parse"))
            .expect("shift task lookup should succeed")
            .expect("shift task should exist");
        shift_state.follow_up_items[0].status = "blocked".to_string();
        shift_state.follow_up_items[0].blocked_reason =
            Some("Waiting for outgoing shift clarification.".to_string());
        shift_state.follow_up_items[0].due_at = Some(now + chrono::Duration::minutes(20));
        repository
            .save(shift_state)
            .expect("shift task save should succeed");

        let mut alert_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID).expect("alert task id should parse"))
            .expect("alert task lookup should succeed")
            .expect("alert task should exist");
        alert_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        alert_state.follow_up_items[0].due_at = Some(expected_next_due_at);
        repository
            .save(alert_state)
            .expect("alert task save should succeed");

        let monitoring_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-monitoring")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("monitoring request should succeed");
        assert_eq!(monitoring_response.status(), StatusCode::OK);
        let monitoring_json = json_body(monitoring_response).await;

        assert_eq!(monitoring_json["total_items"], 2);
        assert_eq!(monitoring_json["open_items"], 2);
        assert_eq!(monitoring_json["accepted_items"], 1);
        assert_eq!(monitoring_json["unaccepted_items"], 1);
        assert_eq!(monitoring_json["blocked_items"], 1);
        assert_eq!(monitoring_json["overdue_items"], 1);
        assert_eq!(monitoring_json["escalation_required_items"], 1);
        assert_eq!(
            DateTime::parse_from_rfc3339(
                monitoring_json["next_due_at"]
                    .as_str()
                    .expect("next_due_at should exist"),
            )
            .expect("next_due_at should parse")
            .with_timezone(&Utc),
            expected_next_due_at
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "source_kind_counts", "shift_handoff"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "source_kind_counts", "alert_triage"),
            1
        );
        assert_eq!(
            json_bucket_count(
                &monitoring_json,
                "owner_role_counts",
                "incoming_shift_supervisor"
            ),
            1
        );
        assert_eq!(
            json_bucket_count(
                &monitoring_json,
                "owner_role_counts",
                "production_supervisor"
            ),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "sla_status_counts", "due_soon"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "sla_status_counts", "escalation_required"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "task_risk_counts", "low"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "task_risk_counts", "high"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "task_priority_counts", "routine"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "task_priority_counts", "expedited"),
            1
        );
        assert!(monitoring_json["last_evaluated_at"].is_string());

        let filtered_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-monitoring?source_kind=alert_triage")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("filtered monitoring request should succeed");
        assert_eq!(filtered_response.status(), StatusCode::OK);
        let filtered_json = json_body(filtered_response).await;

        assert_eq!(filtered_json["total_items"], 1);
        assert_eq!(filtered_json["open_items"], 1);
        assert_eq!(filtered_json["accepted_items"], 0);
        assert_eq!(filtered_json["unaccepted_items"], 1);
        assert_eq!(filtered_json["blocked_items"], 0);
        assert_eq!(filtered_json["overdue_items"], 1);
        assert_eq!(filtered_json["escalation_required_items"], 1);
        assert_eq!(
            json_bucket_count(&filtered_json, "source_kind_counts", "alert_triage"),
            1
        );
        assert_eq!(
            json_bucket_count(&filtered_json, "owner_role_counts", "production_supervisor"),
            1
        );
    }

    #[tokio::test]
    async fn handoff_receipts_queue_endpoint_returns_cross_shift_items_sorted_by_urgency() {
        let (app, repository) = build_in_memory_app_with_repository();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-receipt-queue-intake-001")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("first handoff intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-receipt-queue-intake-002")
                    .body(Body::from(shift_handoff_request_with_id(
                        SHIFT_TASK_ID_2,
                        "Summarize packaging handoff notes",
                        "Summarize packaging line handoff notes for the next shift.",
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("second handoff intake should succeed");

        let now = Utc::now();

        let mut first_state = repository
            .get(Uuid::parse_str(SHIFT_TASK_ID).expect("shift task id should parse"))
            .expect("first task lookup should succeed")
            .expect("first task should exist");
        first_state
            .handoff_receipt
            .as_mut()
            .expect("handoff receipt should exist")
            .required_ack_by = Some(now - chrono::Duration::minutes(5));
        repository
            .save(first_state)
            .expect("first task save should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID_2}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-receipt-queue-ack-001")
                    .body(Body::from(handoff_ack_with_exception_request()))
                    .expect("request should build"),
            )
            .await
            .expect("receipt acknowledgement should succeed");

        let queue_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipts")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("handoff receipt queue request should succeed");
        assert_eq!(queue_response.status(), StatusCode::OK);
        let queue_json = json_body(queue_response).await;
        let items = queue_json.as_array().expect("handoff receipt queue list");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["task_id"], SHIFT_TASK_ID);
        assert_eq!(items[0]["effective_status"], "expired");
        assert_eq!(items[0]["overdue"], true);
        assert_eq!(items[1]["task_id"], SHIFT_TASK_ID_2);
        assert_eq!(items[1]["effective_status"], "acknowledged_with_exceptions");
        assert_eq!(items[1]["has_exceptions"], true);
    }

    #[tokio::test]
    async fn handoff_receipts_queue_endpoint_filters_by_queue_dimensions() {
        let (app, repository) = build_in_memory_app_with_repository();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-receipt-queue-intake-003")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("first handoff intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-receipt-queue-intake-004")
                    .body(Body::from(shift_handoff_request_with_id(
                        SHIFT_TASK_ID_2,
                        "Summarize packaging handoff notes",
                        "Summarize packaging line handoff notes for the next shift.",
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("second handoff intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-receipt-queue-intake-005")
                    .body(Body::from(shift_handoff_request_with_id(
                        SHIFT_TASK_ID_3,
                        "Summarize assembly handoff notes",
                        "Summarize assembly line handoff notes for the next shift.",
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("third handoff intake should succeed");

        let now = Utc::now();

        let first_task_id = Uuid::parse_str(SHIFT_TASK_ID).expect("shift task id should parse");
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
            .required_ack_by = Some(now - chrono::Duration::minutes(5));
        repository
            .save(first_state)
            .expect("first task save should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID_2}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-receipt-queue-ack-002")
                    .body(Body::from(handoff_ack_with_exception_request()))
                    .expect("request should build"),
            )
            .await
            .expect("second receipt acknowledgement should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID_3}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-receipt-queue-ack-003")
                    .body(Body::from(handoff_ack_with_exception_request()))
                    .expect("request should build"),
            )
            .await
            .expect("third receipt acknowledgement should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID_3}/handoff-receipt/escalate"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-handoff-receipt-queue-escalate-001",
                    )
                    .body(Body::from(handoff_escalation_request()))
                    .expect("request should build"),
            )
            .await
            .expect("third receipt escalation should succeed");

        let overdue_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipts?overdue_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("overdue receipt queue request should succeed");
        assert_eq!(overdue_response.status(), StatusCode::OK);
        let overdue_json = json_body(overdue_response).await;
        let overdue_items = overdue_json.as_array().expect("overdue receipt queue list");
        assert_eq!(overdue_items.len(), 1);
        assert_eq!(overdue_items[0]["task_id"], SHIFT_TASK_ID);
        assert_eq!(overdue_items[0]["effective_status"], "expired");

        let exceptions_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipts?has_exceptions=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("exceptions queue request should succeed");
        assert_eq!(exceptions_response.status(), StatusCode::OK);
        let exceptions_json = json_body(exceptions_response).await;
        let exceptions_items = exceptions_json
            .as_array()
            .expect("exception receipt queue list");
        assert_eq!(exceptions_items.len(), 2);

        let escalated_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipts?escalated_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("escalated queue request should succeed");
        assert_eq!(escalated_response.status(), StatusCode::OK);
        let escalated_json = json_body(escalated_response).await;
        let escalated_items = escalated_json
            .as_array()
            .expect("escalated receipt queue list");
        assert_eq!(escalated_items.len(), 1);
        assert_eq!(escalated_items[0]["task_id"], SHIFT_TASK_ID_3);
        assert_eq!(escalated_items[0]["effective_status"], "escalated");

        let actor_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipts?receiving_actor_id=worker_1101")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("actor queue request should succeed");
        assert_eq!(actor_response.status(), StatusCode::OK);
        let actor_json = json_body(actor_response).await;
        let actor_items = actor_json.as_array().expect("actor receipt queue list");
        assert_eq!(actor_items.len(), 2);

        let shift_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/handoff-receipts?shift_id={first_shift_id}"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("shift-specific queue request should succeed");
        assert_eq!(shift_response.status(), StatusCode::OK);
        let shift_json = json_body(shift_response).await;
        let shift_items = shift_json.as_array().expect("shift receipt queue list");
        assert_eq!(shift_items.len(), 1);
        assert_eq!(shift_items[0]["task_id"], SHIFT_TASK_ID);

        let expired_status_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipts?receipt_status=expired")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("expired status queue request should succeed");
        assert_eq!(expired_status_response.status(), StatusCode::OK);
        let expired_status_json = json_body(expired_status_response).await;
        let expired_status_items = expired_status_json
            .as_array()
            .expect("expired status receipt queue list");
        assert_eq!(expired_status_items.len(), 1);
        assert_eq!(expired_status_items[0]["task_id"], SHIFT_TASK_ID);
    }

    #[tokio::test]
    async fn alert_clusters_queue_endpoint_returns_cross_task_items_sorted_by_escalation() {
        let app = app(build_state().expect("state should build"));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-alert-cluster-queue-intake-001")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("andon alert intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-alert-cluster-queue-intake-002")
                    .body(Body::from(scada_threshold_alert_request()))
                    .expect("request should build"),
            )
            .await
            .expect("scada alert intake should succeed");

        let queue_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("alert cluster queue request should succeed");
        assert_eq!(queue_response.status(), StatusCode::OK);
        let queue_json = json_body(queue_response).await;
        let items = queue_json.as_array().expect("alert cluster queue list");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["task_id"], ALERT_TASK_ID);
        assert_eq!(items[0]["source_system"], "andon");
        assert_eq!(items[0]["severity_band"], "high");
        assert_eq!(items[0]["escalation_candidate"], true);
        assert_eq!(items[1]["task_id"], ALERT_TASK_ID_2);
        assert_eq!(items[1]["source_system"], "scada");
        assert_eq!(items[1]["severity_band"], "medium");
        assert_eq!(items[1]["recommended_owner_role"], "maintenance_engineer");
    }

    #[tokio::test]
    async fn alert_clusters_queue_endpoint_filters_by_queue_dimensions() {
        let (app, repository) = build_in_memory_app_with_repository();

        let andon_intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-alert-cluster-queue-intake-003")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("andon alert intake should succeed");
        assert_eq!(andon_intake_response.status(), StatusCode::OK);

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-alert-cluster-queue-intake-004")
                    .body(Body::from(scada_threshold_alert_request()))
                    .expect("request should build"),
            )
            .await
            .expect("scada alert intake should succeed");

        let now = Utc::now();
        let window_from = (now - chrono::Duration::minutes(12))
            .to_rfc3339()
            .replace("+00:00", "Z");
        let window_to = (now - chrono::Duration::minutes(2))
            .to_rfc3339()
            .replace("+00:00", "Z");

        let mut andon_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID).expect("alert task id should parse"))
            .expect("andon task lookup should succeed")
            .expect("andon task should exist");
        andon_state.alert_cluster_drafts[0].window_start = now - chrono::Duration::minutes(30);
        andon_state.alert_cluster_drafts[0].window_end = now - chrono::Duration::minutes(25);
        repository
            .save(andon_state)
            .expect("andon task save should succeed");

        let mut scada_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID_2).expect("scada task id should parse"))
            .expect("scada task lookup should succeed")
            .expect("scada task should exist");
        scada_state.alert_cluster_drafts[0].cluster_status = "closed".to_string();
        scada_state.alert_cluster_drafts[0].window_start = now - chrono::Duration::minutes(10);
        scada_state.alert_cluster_drafts[0].window_end = now - chrono::Duration::minutes(5);
        repository
            .save(scada_state)
            .expect("scada task save should succeed");

        let source_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?source_system=scada")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("source-filtered alert cluster queue request should succeed");
        assert_eq!(source_response.status(), StatusCode::OK);
        let source_json = json_body(source_response).await;
        let source_items = source_json.as_array().expect("source queue list");
        assert_eq!(source_items.len(), 1);
        assert_eq!(source_items[0]["task_id"], ALERT_TASK_ID_2);

        let line_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?line_id=line_mix_02")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("line-filtered alert cluster queue request should succeed");
        assert_eq!(line_response.status(), StatusCode::OK);
        let line_json = json_body(line_response).await;
        let line_items = line_json.as_array().expect("line queue list");
        assert_eq!(line_items.len(), 1);
        assert_eq!(line_items[0]["task_id"], ALERT_TASK_ID_2);

        let label_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?triage_label=sustained_threshold_review")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("label-filtered alert cluster queue request should succeed");
        assert_eq!(label_response.status(), StatusCode::OK);
        let label_json = json_body(label_response).await;
        let label_items = label_json.as_array().expect("label queue list");
        assert_eq!(label_items.len(), 1);
        assert_eq!(label_items[0]["task_id"], ALERT_TASK_ID_2);

        let owner_role_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?recommended_owner_role=maintenance_engineer")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("owner-role-filtered alert cluster queue request should succeed");
        assert_eq!(owner_role_response.status(), StatusCode::OK);
        let owner_role_json = json_body(owner_role_response).await;
        let owner_role_items = owner_role_json.as_array().expect("owner-role queue list");
        assert_eq!(owner_role_items.len(), 1);
        assert_eq!(owner_role_items[0]["task_id"], ALERT_TASK_ID_2);

        let escalation_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?escalation_candidate=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("escalation-filtered alert cluster queue request should succeed");
        assert_eq!(escalation_response.status(), StatusCode::OK);
        let escalation_json = json_body(escalation_response).await;
        let escalation_items = escalation_json.as_array().expect("escalation queue list");
        assert_eq!(escalation_items.len(), 1);
        assert_eq!(escalation_items[0]["task_id"], ALERT_TASK_ID);

        let window_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/alert-clusters?window_from={window_from}&window_to={window_to}"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("window-filtered alert cluster queue request should succeed");
        assert_eq!(window_response.status(), StatusCode::OK);
        let window_json = json_body(window_response).await;
        let window_items = window_json.as_array().expect("window queue list");
        assert_eq!(window_items.len(), 1);
        assert_eq!(window_items[0]["task_id"], ALERT_TASK_ID_2);

        let open_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?open_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("open-only alert cluster queue request should succeed");
        assert_eq!(open_response.status(), StatusCode::OK);
        let open_json = json_body(open_response).await;
        let open_items = open_json.as_array().expect("open queue list");
        assert_eq!(open_items.len(), 1);
        assert_eq!(open_items[0]["task_id"], ALERT_TASK_ID);
    }

    #[tokio::test]
    async fn alert_clusters_queue_endpoint_includes_linked_follow_up_state() {
        let (app, repository) = build_in_memory_app_with_repository();

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-alert-cluster-link-intake-001")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("alert intake should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);
        let intake_json = json_body(intake_response).await;
        let follow_up_id = intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let accept_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{ALERT_TASK_ID}/follow-up-items/{follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-alert-cluster-link-accept-001")
                    .body(Body::from(alert_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up accept should succeed");
        assert_eq!(accept_response.status(), StatusCode::OK);

        let mut stored = repository
            .get(Uuid::parse_str(ALERT_TASK_ID).expect("alert task id should parse"))
            .expect("alert task lookup should succeed")
            .expect("alert task should exist");
        let cluster_id = stored.alert_cluster_drafts[0].cluster_id.clone();
        stored.follow_up_items[0].source_kind = "alert_cluster".to_string();
        stored.follow_up_items[0].source_refs = vec![cluster_id];
        stored.follow_up_items[0].sla_status = "escalation_required".to_string();
        stored.follow_up_items[0].updated_at = Utc::now();
        repository
            .save(stored)
            .expect("alert task save should succeed");

        let queue_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("alert cluster queue request should succeed");
        assert_eq!(queue_response.status(), StatusCode::OK);
        let queue_json = json_body(queue_response).await;
        let items = queue_json.as_array().expect("alert cluster queue list");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["task_id"], ALERT_TASK_ID);
        assert_eq!(items[0]["linked_follow_up"]["total_items"], 1);
        assert_eq!(items[0]["linked_follow_up"]["open_items"], 1);
        assert_eq!(items[0]["linked_follow_up"]["accepted_items"], 1);
        assert_eq!(items[0]["linked_follow_up"]["unaccepted_items"], 0);
        assert_eq!(
            items[0]["linked_follow_up"]["follow_up_ids"][0],
            follow_up_id
        );
        assert_eq!(
            items[0]["linked_follow_up"]["accepted_owner_ids"][0],
            "worker_1001"
        );
        assert_eq!(
            items[0]["linked_follow_up"]["worst_effective_sla_status"],
            "escalation_required"
        );
    }

    #[tokio::test]
    async fn alert_clusters_queue_endpoint_filters_by_linked_follow_up_dimensions() {
        let (app, repository) = build_in_memory_app_with_repository();

        let andon_intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-alert-cluster-link-filters-intake-001",
                    )
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("andon alert intake should succeed");
        assert_eq!(andon_intake_response.status(), StatusCode::OK);
        let andon_intake_json = json_body(andon_intake_response).await;
        let andon_follow_up_id = andon_intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-alert-cluster-link-filters-intake-002",
                    )
                    .body(Body::from(scada_threshold_alert_request()))
                    .expect("request should build"),
            )
            .await
            .expect("scada alert intake should succeed");

        let accept_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{ALERT_TASK_ID}/follow-up-items/{andon_follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-alert-cluster-link-filters-accept-001")
                    .body(Body::from(alert_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up accept should succeed");
        assert_eq!(accept_response.status(), StatusCode::OK);

        let mut andon_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID).expect("alert task id should parse"))
            .expect("andon task lookup should succeed")
            .expect("andon task should exist");
        let cluster_id = andon_state.alert_cluster_drafts[0].cluster_id.clone();
        andon_state.follow_up_items[0].source_kind = "alert_cluster".to_string();
        andon_state.follow_up_items[0].source_refs = vec![cluster_id];
        andon_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        repository
            .save(andon_state)
            .expect("andon task save should succeed");

        let owner_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?follow_up_owner_id=worker_1001")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("owner-filtered alert cluster queue request should succeed");
        assert_eq!(owner_response.status(), StatusCode::OK);
        let owner_json = json_body(owner_response).await;
        let owner_items = owner_json.as_array().expect("owner queue list");
        assert_eq!(owner_items.len(), 1);
        assert_eq!(owner_items[0]["task_id"], ALERT_TASK_ID);

        let unaccepted_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?unaccepted_follow_up_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("unaccepted alert cluster queue request should succeed");
        assert_eq!(unaccepted_response.status(), StatusCode::OK);
        let unaccepted_json = json_body(unaccepted_response).await;
        let unaccepted_items = unaccepted_json.as_array().expect("unaccepted queue list");
        assert_eq!(unaccepted_items.len(), 1);
        assert_eq!(unaccepted_items[0]["task_id"], ALERT_TASK_ID_2);

        let escalation_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?follow_up_escalation_required=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("follow-up escalation alert cluster queue request should succeed");
        assert_eq!(escalation_response.status(), StatusCode::OK);
        let escalation_json = json_body(escalation_response).await;
        let escalation_items = escalation_json.as_array().expect("escalation queue list");
        assert_eq!(escalation_items.len(), 1);
        assert_eq!(escalation_items[0]["task_id"], ALERT_TASK_ID);
    }

    #[tokio::test]
    async fn alert_cluster_monitoring_endpoint_returns_aggregated_and_filtered_views() {
        let (app, repository) = build_in_memory_app_with_repository();

        let andon_intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-alert-cluster-monitoring-intake-001",
                    )
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("andon alert intake should succeed");
        assert_eq!(andon_intake_response.status(), StatusCode::OK);
        let andon_intake_json = json_body(andon_intake_response).await;
        let andon_follow_up_id = andon_intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-alert-cluster-monitoring-intake-002",
                    )
                    .body(Body::from(scada_threshold_alert_request()))
                    .expect("request should build"),
            )
            .await
            .expect("scada alert intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-alert-cluster-monitoring-intake-003")
                    .body(Body::from(scada_threshold_alert_request_with_id(
                        ALERT_TASK_ID_3,
                        "Triage reserve threshold alert on mix line 3",
                        "Review reserve SCADA threshold breach on mix line 3 for the next planned batch.",
                        "eq_mix_03",
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("future scada alert intake should succeed");

        let now = Utc::now();
        let expected_next_window_end = now - chrono::Duration::minutes(25);

        let accept_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{ALERT_TASK_ID}/follow-up-items/{andon_follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-alert-cluster-monitoring-linked-follow-up-001",
                    )
                    .body(Body::from(alert_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up accept should succeed");
        assert_eq!(accept_response.status(), StatusCode::OK);

        let mut andon_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID).expect("alert task id should parse"))
            .expect("andon task lookup should succeed")
            .expect("andon task should exist");
        let cluster_id = andon_state.alert_cluster_drafts[0].cluster_id.clone();
        andon_state.alert_cluster_drafts[0].window_start = now - chrono::Duration::minutes(30);
        andon_state.alert_cluster_drafts[0].window_end = expected_next_window_end;
        andon_state.follow_up_items[0].source_kind = "alert_cluster".to_string();
        andon_state.follow_up_items[0].source_refs = vec![cluster_id];
        andon_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        repository
            .save(andon_state)
            .expect("andon task save should succeed");

        let mut scada_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID_2).expect("scada task id should parse"))
            .expect("scada task lookup should succeed")
            .expect("scada task should exist");
        scada_state.alert_cluster_drafts[0].window_start = now - chrono::Duration::minutes(5);
        scada_state.alert_cluster_drafts[0].window_end = now + chrono::Duration::minutes(10);
        repository
            .save(scada_state)
            .expect("scada task save should succeed");

        let mut future_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID_3).expect("future task id should parse"))
            .expect("future task lookup should succeed")
            .expect("future task should exist");
        future_state.alert_cluster_drafts[0].cluster_status = "closed".to_string();
        future_state.alert_cluster_drafts[0].window_start = now + chrono::Duration::minutes(20);
        future_state.alert_cluster_drafts[0].window_end = now + chrono::Duration::minutes(35);
        repository
            .save(future_state)
            .expect("future task save should succeed");

        let monitoring_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-cluster-monitoring")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("monitoring request should succeed");
        assert_eq!(monitoring_response.status(), StatusCode::OK);
        let monitoring_json = json_body(monitoring_response).await;

        assert_eq!(monitoring_json["total_clusters"], 3);
        assert_eq!(monitoring_json["open_clusters"], 2);
        assert_eq!(monitoring_json["escalation_candidate_clusters"], 1);
        assert_eq!(monitoring_json["high_severity_clusters"], 1);
        assert_eq!(monitoring_json["active_window_clusters"], 1);
        assert_eq!(monitoring_json["stale_window_clusters"], 1);
        assert_eq!(monitoring_json["linked_follow_up_clusters"], 3);
        assert_eq!(monitoring_json["unlinked_follow_up_clusters"], 0);
        assert_eq!(monitoring_json["accepted_follow_up_clusters"], 1);
        assert_eq!(monitoring_json["unaccepted_follow_up_clusters"], 2);
        assert_eq!(monitoring_json["follow_up_escalation_clusters"], 1);
        assert_eq!(
            DateTime::parse_from_rfc3339(
                monitoring_json["next_window_end_at"]
                    .as_str()
                    .expect("next_window_end_at should exist"),
            )
            .expect("next_window_end_at should parse")
            .with_timezone(&Utc),
            expected_next_window_end
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "cluster_status_counts", "open"),
            2
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "cluster_status_counts", "closed"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "source_system_counts", "andon"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "source_system_counts", "scada"),
            2
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "severity_band_counts", "high"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "severity_band_counts", "medium"),
            2
        );
        assert_eq!(
            json_bucket_count(
                &monitoring_json,
                "triage_label_counts",
                "repeated_alert_review"
            ),
            1
        );
        assert_eq!(
            json_bucket_count(
                &monitoring_json,
                "triage_label_counts",
                "sustained_threshold_review"
            ),
            2
        );
        assert_eq!(
            json_bucket_count(
                &monitoring_json,
                "owner_role_counts",
                "production_supervisor"
            ),
            1
        );
        assert_eq!(
            json_bucket_count(
                &monitoring_json,
                "owner_role_counts",
                "maintenance_engineer"
            ),
            2
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "window_state_counts", "stale"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "window_state_counts", "active"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "window_state_counts", "future"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "follow_up_coverage_counts", "linked"),
            3
        );
        assert_eq!(
            json_bucket_count(
                &monitoring_json,
                "follow_up_sla_status_counts",
                "escalation_required"
            ),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "follow_up_sla_status_counts", "due_soon"),
            2
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "task_risk_counts", "high"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "task_risk_counts", "medium"),
            2
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "task_priority_counts", "expedited"),
            3
        );
        assert!(monitoring_json["last_evaluated_at"].is_string());

        let filtered_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-cluster-monitoring?source_system=scada")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("filtered monitoring request should succeed");
        assert_eq!(filtered_response.status(), StatusCode::OK);
        let filtered_json = json_body(filtered_response).await;

        assert_eq!(filtered_json["total_clusters"], 2);
        assert_eq!(filtered_json["open_clusters"], 1);
        assert_eq!(filtered_json["escalation_candidate_clusters"], 0);
        assert_eq!(filtered_json["high_severity_clusters"], 0);
        assert_eq!(filtered_json["active_window_clusters"], 1);
        assert_eq!(filtered_json["stale_window_clusters"], 0);
        assert_eq!(filtered_json["linked_follow_up_clusters"], 2);
        assert_eq!(filtered_json["unlinked_follow_up_clusters"], 0);
        assert_eq!(filtered_json["accepted_follow_up_clusters"], 0);
        assert_eq!(filtered_json["unaccepted_follow_up_clusters"], 2);
        assert_eq!(filtered_json["follow_up_escalation_clusters"], 0);
        assert_eq!(
            json_bucket_count(&filtered_json, "cluster_status_counts", "open"),
            1
        );
        assert_eq!(
            json_bucket_count(&filtered_json, "cluster_status_counts", "closed"),
            1
        );
        assert_eq!(
            json_bucket_count(&filtered_json, "source_system_counts", "scada"),
            2
        );
        assert_eq!(
            json_bucket_count(&filtered_json, "window_state_counts", "active"),
            1
        );
        assert_eq!(
            json_bucket_count(&filtered_json, "follow_up_coverage_counts", "linked"),
            2
        );
        assert_eq!(
            json_bucket_count(&filtered_json, "follow_up_sla_status_counts", "due_soon"),
            2
        );
    }

    #[tokio::test]
    async fn alert_cluster_monitoring_endpoint_filters_by_linked_follow_up_dimensions() {
        let (app, repository) = build_in_memory_app_with_repository();

        let andon_intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-alert-cluster-monitoring-link-intake-001",
                    )
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("andon alert intake should succeed");
        assert_eq!(andon_intake_response.status(), StatusCode::OK);
        let andon_intake_json = json_body(andon_intake_response).await;
        let andon_follow_up_id = andon_intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-alert-cluster-monitoring-link-intake-002",
                    )
                    .body(Body::from(scada_threshold_alert_request()))
                    .expect("request should build"),
            )
            .await
            .expect("scada alert intake should succeed");

        let accept_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{ALERT_TASK_ID}/follow-up-items/{andon_follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "itest-alert-cluster-monitoring-link-accept-001",
                    )
                    .body(Body::from(alert_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up accept should succeed");
        assert_eq!(accept_response.status(), StatusCode::OK);

        let mut andon_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID).expect("alert task id should parse"))
            .expect("andon task lookup should succeed")
            .expect("andon task should exist");
        let cluster_id = andon_state.alert_cluster_drafts[0].cluster_id.clone();
        andon_state.follow_up_items[0].source_kind = "alert_cluster".to_string();
        andon_state.follow_up_items[0].source_refs = vec![cluster_id];
        andon_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        repository
            .save(andon_state)
            .expect("andon task save should succeed");

        let escalation_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-cluster-monitoring?follow_up_escalation_required=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("follow-up escalation monitoring request should succeed");
        assert_eq!(escalation_response.status(), StatusCode::OK);
        let escalation_json = json_body(escalation_response).await;
        assert_eq!(escalation_json["total_clusters"], 1);
        assert_eq!(
            json_bucket_count(&escalation_json, "source_system_counts", "andon"),
            1
        );

        let unaccepted_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-cluster-monitoring?unaccepted_follow_up_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("unaccepted monitoring request should succeed");
        assert_eq!(unaccepted_response.status(), StatusCode::OK);
        let unaccepted_json = json_body(unaccepted_response).await;
        assert_eq!(unaccepted_json["total_clusters"], 1);
        assert_eq!(
            json_bucket_count(&unaccepted_json, "source_system_counts", "scada"),
            1
        );
    }

    #[tokio::test]
    async fn handoff_receipt_monitoring_endpoint_returns_aggregated_and_filtered_views() {
        let (app, repository) = build_in_memory_app_with_repository();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-monitoring-intake-001")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("first handoff intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-monitoring-intake-002")
                    .body(Body::from(shift_handoff_request_with_id(
                        SHIFT_TASK_ID_2,
                        "Summarize packaging handoff notes",
                        "Summarize packaging line handoff notes for the next shift.",
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("second handoff intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-monitoring-intake-003")
                    .body(Body::from(shift_handoff_request_with_id(
                        SHIFT_TASK_ID_3,
                        "Summarize assembly handoff notes",
                        "Summarize assembly line handoff notes for the next shift.",
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("third handoff intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-monitoring-intake-004")
                    .body(Body::from(shift_handoff_request_with_id(
                        SHIFT_TASK_ID_4,
                        "Summarize paint handoff notes",
                        "Summarize paint line handoff notes for the next shift.",
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("fourth handoff intake should succeed");

        let now = Utc::now();
        let expected_next_ack_due_at = now - chrono::Duration::minutes(5);

        let mut first_state = repository
            .get(Uuid::parse_str(SHIFT_TASK_ID).expect("shift task id should parse"))
            .expect("first task lookup should succeed")
            .expect("first task should exist");
        first_state
            .handoff_receipt
            .as_mut()
            .expect("handoff receipt should exist")
            .required_ack_by = Some(expected_next_ack_due_at);
        repository
            .save(first_state)
            .expect("first task save should succeed");

        let mut second_state = repository
            .get(Uuid::parse_str(SHIFT_TASK_ID_2).expect("shift task id should parse"))
            .expect("second task lookup should succeed")
            .expect("second task should exist");
        second_state
            .handoff_receipt
            .as_mut()
            .expect("handoff receipt should exist")
            .required_ack_by = Some(now + chrono::Duration::minutes(20));
        repository
            .save(second_state)
            .expect("second task save should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID_3}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-monitoring-ack-001")
                    .body(Body::from(handoff_ack_with_exception_request()))
                    .expect("request should build"),
            )
            .await
            .expect("third receipt acknowledgement should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID_4}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-monitoring-ack-002")
                    .body(Body::from(handoff_ack_with_exception_request()))
                    .expect("request should build"),
            )
            .await
            .expect("fourth receipt acknowledgement should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID_4}/handoff-receipt/escalate"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-handoff-monitoring-escalate-001")
                    .body(Body::from(handoff_escalation_request()))
                    .expect("request should build"),
            )
            .await
            .expect("fourth receipt escalation should succeed");

        let monitoring_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipt-monitoring")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("monitoring request should succeed");
        assert_eq!(monitoring_response.status(), StatusCode::OK);
        let monitoring_json = json_body(monitoring_response).await;

        assert_eq!(monitoring_json["total_receipts"], 4);
        assert_eq!(monitoring_json["open_receipts"], 4);
        assert_eq!(monitoring_json["acknowledged_receipts"], 2);
        assert_eq!(monitoring_json["unacknowledged_receipts"], 2);
        assert_eq!(monitoring_json["overdue_receipts"], 1);
        assert_eq!(monitoring_json["exception_receipts"], 2);
        assert_eq!(monitoring_json["escalated_receipts"], 1);
        assert_eq!(
            DateTime::parse_from_rfc3339(
                monitoring_json["next_ack_due_at"]
                    .as_str()
                    .expect("next_ack_due_at should exist"),
            )
            .expect("next_ack_due_at should parse")
            .with_timezone(&Utc),
            expected_next_ack_due_at
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "effective_status_counts", "expired"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "effective_status_counts", "published"),
            1
        );
        assert_eq!(
            json_bucket_count(
                &monitoring_json,
                "effective_status_counts",
                "acknowledged_with_exceptions"
            ),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "effective_status_counts", "escalated"),
            1
        );
        assert_eq!(
            json_bucket_count(
                &monitoring_json,
                "receiving_role_counts",
                "incoming_shift_supervisor"
            ),
            4
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "ack_window_counts", "overdue"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "ack_window_counts", "due_within_30m"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "task_risk_counts", "low"),
            4
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "task_priority_counts", "routine"),
            4
        );
        assert!(monitoring_json["last_evaluated_at"].is_string());

        let filtered_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipt-monitoring?escalated_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("filtered monitoring request should succeed");
        assert_eq!(filtered_response.status(), StatusCode::OK);
        let filtered_json = json_body(filtered_response).await;

        assert_eq!(filtered_json["total_receipts"], 1);
        assert_eq!(filtered_json["open_receipts"], 1);
        assert_eq!(filtered_json["acknowledged_receipts"], 1);
        assert_eq!(filtered_json["unacknowledged_receipts"], 0);
        assert_eq!(filtered_json["overdue_receipts"], 0);
        assert_eq!(filtered_json["exception_receipts"], 1);
        assert_eq!(filtered_json["escalated_receipts"], 1);
        assert!(filtered_json["next_ack_due_at"].is_null());
        assert_eq!(
            json_bucket_count(&filtered_json, "effective_status_counts", "escalated"),
            1
        );
    }

    #[tokio::test]
    async fn rejected_task_can_be_resubmitted_for_approval() {
        let app = app(build_state().expect("state should build"));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-intake-003")
                    .body(Body::from(high_risk_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");

        let reject_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/approve"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-reject-001")
                    .body(Body::from(
                        r#"{
                            "decided_by":{"id":"worker_2001","display_name":"Wang Safety","role":"Safety Officer"},
                            "approved":false,
                            "comment":"Need additional evidence"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("rejection should succeed");
        assert_eq!(reject_response.status(), StatusCode::OK);
        let reject_json = json_body(reject_response).await;
        assert_eq!(reject_json["planned_task"]["task"]["status"], "planned");
        assert_eq!(
            reject_json["planned_task"]["approval"]["status"],
            "rejected"
        );

        let execute_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/execute"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-execute-003")
                    .body(Body::from(
                        r#"{
                            "actor":{"id":"worker_3001","display_name":"Wu Maint","role":"Maintenance Technician"},
                            "note":"Execution should not start"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("execute response should succeed");
        assert_eq!(execute_response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let execute_json = json_body(execute_response).await;
        assert_eq!(
            execute_json["error"],
            "invalid task transition from Planned to Executing"
        );

        let resubmit_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/resubmit"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-resubmit-001")
                    .body(Body::from(
                        r#"{
                            "requested_by":{"id":"worker_1001","display_name":"Liu Supervisor","role":"Production Supervisor"},
                            "comment":"Added vibration report and revised action plan"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("resubmit should succeed");
        assert_eq!(resubmit_response.status(), StatusCode::OK);
        let resubmit_json = json_body(resubmit_response).await;
        assert_eq!(
            resubmit_json["planned_task"]["task"]["status"],
            "awaiting_approval"
        );
        assert_eq!(
            resubmit_json["planned_task"]["approval"]["status"],
            "pending"
        );

        let approve_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/approve"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-approve-003")
                    .body(Body::from(
                        r#"{
                            "decided_by":{"id":"worker_2001","display_name":"Wang Safety","role":"Safety Officer"},
                            "approved":true,
                            "comment":"Revision accepted"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("approval after resubmission should succeed");
        assert_eq!(approve_response.status(), StatusCode::OK);
        let approve_json = json_body(approve_response).await;
        assert_eq!(approve_json["planned_task"]["task"]["status"], "approved");
    }

    #[tokio::test]
    async fn audit_events_support_filtering_and_task_replay() {
        let app = app(build_state().expect("state should build"));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-intake-004")
                    .body(Body::from(high_risk_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/approve"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-approve-004")
                    .body(Body::from(
                        r#"{
                            "decided_by":{"id":"worker_2001","display_name":"Wang Safety","role":"Safety Officer"},
                            "approved":true,
                            "comment":"Proceed to execution"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("approval should succeed");

        let correlation_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/audit/events?correlation_id=itest-approve-004")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("correlation query should succeed");
        assert_eq!(correlation_response.status(), StatusCode::OK);
        let correlation_json = json_body(correlation_response).await;
        let correlation_events = correlation_json.as_array().expect("audit list");
        assert_eq!(correlation_events.len(), 2);
        assert!(correlation_events
            .iter()
            .all(|event| event["correlation_id"] == "itest-approve-004"));

        let task_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/audit-events"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("task replay should succeed");
        assert_eq!(task_response.status(), StatusCode::OK);
        let task_json = json_body(task_response).await;
        let task_events = task_json.as_array().expect("audit list");
        assert!(task_events.len() >= 8);
        assert!(task_events.iter().all(|event| event["task_id"] == TASK_ID));

        let kind_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/audit/events?task_id={TASK_ID}&kind=approval_requested"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("kind query should succeed");
        assert_eq!(kind_response.status(), StatusCode::OK);
        let kind_json = json_body(kind_response).await;
        let kind_events = kind_json.as_array().expect("audit list");
        assert_eq!(kind_events.len(), 1);
        assert_eq!(kind_events[0]["kind"], "approval_requested");
    }

    #[tokio::test]
    async fn approve_endpoint_rejects_wrong_role() {
        let app = app(build_state().expect("state should build"));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-intake-005")
                    .body(Body::from(high_risk_request()))
                    .expect("request should build"),
            )
            .await
            .expect("intake should succeed");

        let approve_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/approve"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "itest-approve-mismatch-001")
                    .body(Body::from(
                        r#"{
                            "decided_by":{"id":"worker_2002","display_name":"Chen QE","role":"Quality Engineer"},
                            "approved":true,
                            "comment":"Proceed to execution"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");

        assert_eq!(approve_response.status(), StatusCode::FORBIDDEN);
        let approve_json = json_body(approve_response).await;
        assert_eq!(
            approve_json["error"],
            "approval requires role 'safety_officer', got 'quality_engineer'"
        );

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_json = json_body(get_response).await;
        assert_eq!(
            get_json["planned_task"]["task"]["status"],
            "awaiting_approval"
        );
        assert_eq!(get_json["planned_task"]["approval"]["status"], "pending");
    }

    #[tokio::test]
    async fn sandbox_safe_file_mode_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-smoke");
        let data_dir = sandbox_dir.path().to_path_buf();
        let app = build_file_backed_app(data_dir.clone());

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-intake-001")
                    .body(Body::from(high_risk_request()))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);
        let intake_json = json_body(intake_response).await;
        assert_eq!(
            intake_json["planned_task"]["task"]["status"],
            "awaiting_approval"
        );
        assert!(intake_json["follow_up_items"]
            .as_array()
            .expect("follow-up list")
            .is_empty());
        assert_eq!(intake_json["follow_up_summary"]["total_items"], 0);
        assert!(intake_json["handoff_receipt"].is_null());
        assert_eq!(
            intake_json["handoff_receipt_summary"]["covered_follow_up_count"],
            0
        );
        assert!(intake_json["alert_cluster_drafts"]
            .as_array()
            .expect("alert cluster list")
            .is_empty());
        assert_eq!(intake_json["alert_triage_summary"]["total_clusters"], 0);

        let evidence_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/evidence"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(evidence_response.status(), StatusCode::OK);
        let evidence_json = json_body(evidence_response).await;
        assert_eq!(evidence_json.as_array().expect("evidence list").len(), 4);

        let governance_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/governance"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(governance_response.status(), StatusCode::OK);
        let governance_json = json_body(governance_response).await;
        assert_eq!(
            governance_json["approval_strategy"]["required_role"],
            "safety_officer"
        );

        let restarted_app = build_file_backed_app(data_dir.clone());

        let persisted_task_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(persisted_task_response.status(), StatusCode::OK);
        let persisted_task_json = json_body(persisted_task_response).await;
        assert_eq!(
            persisted_task_json["planned_task"]["task"]["status"],
            "awaiting_approval"
        );
        assert!(persisted_task_json["follow_up_items"]
            .as_array()
            .expect("follow-up list")
            .is_empty());
        assert_eq!(persisted_task_json["follow_up_summary"]["total_items"], 0);
        assert!(persisted_task_json["handoff_receipt"].is_null());
        assert_eq!(
            persisted_task_json["handoff_receipt_summary"]["unaccepted_follow_up_count"],
            0
        );
        assert!(persisted_task_json["alert_cluster_drafts"]
            .as_array()
            .expect("alert cluster list")
            .is_empty());
        assert_eq!(
            persisted_task_json["alert_triage_summary"]["total_clusters"],
            0
        );

        let mismatched_approve_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/approve"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-approve-mismatch-001")
                    .body(Body::from(
                        r#"{
                            "decided_by":{"id":"worker_2002","display_name":"Chen QE","role":"Quality Engineer"},
                            "approved":true,
                            "comment":"Proceed to execution"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(mismatched_approve_response.status(), StatusCode::FORBIDDEN);

        let approve_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/approve"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-approve-001")
                    .body(Body::from(
                        r#"{
                            "decided_by":{"id":"worker_2001","display_name":"Wang Safety","role":"Safety Officer"},
                            "approved":true,
                            "comment":"Proceed to execution"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(approve_response.status(), StatusCode::OK);

        let execute_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/execute"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-execute-001")
                    .body(Body::from(
                        r#"{
                            "actor":{"id":"worker_3001","display_name":"Wu Maint","role":"Maintenance Technician"},
                            "note":"Execution stub started"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(execute_response.status(), StatusCode::OK);

        let complete_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/complete"))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-complete-001")
                    .body(Body::from(
                        r#"{
                            "actor":{"id":"worker_3001","display_name":"Wu Maint","role":"Maintenance Technician"},
                            "note":"Execution finished"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(complete_response.status(), StatusCode::OK);
        let complete_json = json_body(complete_response).await;
        assert_eq!(complete_json["planned_task"]["task"]["status"], "completed");

        let audit_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{TASK_ID}/audit-events"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(audit_response.status(), StatusCode::OK);
        let audit_json = json_body(audit_response).await;
        assert!(audit_json.as_array().expect("audit list").len() >= 8);

        let task_file = data_dir.join("tasks").join(format!("{TASK_ID}.json"));
        let audit_file = data_dir.join("audit-events.jsonl");
        assert!(
            task_file.exists(),
            "task file should exist in sandbox data dir"
        );
        assert!(
            audit_file.exists(),
            "audit file should exist in sandbox data dir"
        );
    }

    #[tokio::test]
    async fn sandbox_safe_shift_handoff_acknowledgement_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-handoff-ack-smoke");
        let data_dir = sandbox_dir.path().to_path_buf();
        let app = build_file_backed_app(data_dir.clone());

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-shift-handoff-intake-001")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);

        let acknowledge_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-shift-handoff-ack-001")
                    .body(Body::from(handoff_ack_request()))
                    .expect("request should build"),
            )
            .await
            .expect("acknowledge should succeed");
        assert_eq!(acknowledge_response.status(), StatusCode::OK);
        let acknowledge_json = json_body(acknowledge_response).await;
        assert_eq!(
            acknowledge_json["handoff_receipt"]["status"],
            "acknowledged"
        );
        assert_eq!(
            acknowledge_json["handoff_receipt_summary"]["status"],
            "acknowledged"
        );

        let restarted_app = build_file_backed_app(data_dir.clone());

        let persisted_task_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{SHIFT_TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(persisted_task_response.status(), StatusCode::OK);
        let persisted_task_json = json_body(persisted_task_response).await;
        assert_eq!(
            persisted_task_json["handoff_receipt"]["status"],
            "acknowledged"
        );
        assert_eq!(
            persisted_task_json["handoff_receipt"]["receiving_actor"]["id"],
            "worker_1101"
        );
        assert!(persisted_task_json["handoff_receipt_summary"]["acknowledged_at"].is_string());

        let audit_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/audit-events?kind=handoff_acknowledged"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("audit query should succeed");
        assert_eq!(audit_response.status(), StatusCode::OK);
        let audit_json = json_body(audit_response).await;
        assert_eq!(audit_json.as_array().expect("audit list").len(), 1);
    }

    #[tokio::test]
    async fn sandbox_safe_shift_handoff_follow_up_acceptance_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-follow-up-accept-smoke");
        let data_dir = sandbox_dir.path().to_path_buf();
        let app = build_file_backed_app(data_dir.clone());

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-follow-up-accept-intake-001")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);
        let intake_json = json_body(intake_response).await;
        let follow_up_id = intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let accept_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/follow-up-items/{follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-follow-up-accept-001")
                    .body(Body::from(shift_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up accept should succeed");
        assert_eq!(accept_response.status(), StatusCode::OK);
        let accept_json = json_body(accept_response).await;
        assert_eq!(accept_json["follow_up_items"][0]["status"], "accepted");
        assert_eq!(
            accept_json["handoff_receipt_summary"]["unaccepted_follow_up_count"],
            0
        );

        let restarted_app = build_file_backed_app(data_dir.clone());

        let persisted_task_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{SHIFT_TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(persisted_task_response.status(), StatusCode::OK);
        let persisted_task_json = json_body(persisted_task_response).await;
        assert_eq!(
            persisted_task_json["follow_up_items"][0]["status"],
            "accepted"
        );
        assert_eq!(
            persisted_task_json["follow_up_items"][0]["accepted_owner_id"],
            "worker_1101"
        );
        assert_eq!(
            persisted_task_json["handoff_receipt_summary"]["unaccepted_follow_up_count"],
            0
        );

        let audit_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/audit-events?kind=follow_up_owner_accepted"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("audit query should succeed");
        assert_eq!(audit_response.status(), StatusCode::OK);
        let audit_json = json_body(audit_response).await;
        assert_eq!(audit_json.as_array().expect("audit list").len(), 1);
    }

    #[tokio::test]
    async fn sandbox_safe_shift_handoff_escalation_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-handoff-escalate-smoke");
        let data_dir = sandbox_dir.path().to_path_buf();
        let app = build_file_backed_app(data_dir.clone());

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-shift-handoff-intake-002")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);

        let acknowledge_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-shift-handoff-ack-002")
                    .body(Body::from(handoff_ack_with_exception_request()))
                    .expect("request should build"),
            )
            .await
            .expect("acknowledge should succeed");
        assert_eq!(acknowledge_response.status(), StatusCode::OK);

        let escalate_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/handoff-receipt/escalate"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-shift-handoff-escalate-001")
                    .body(Body::from(handoff_escalation_request()))
                    .expect("request should build"),
            )
            .await
            .expect("escalate should succeed");
        assert_eq!(escalate_response.status(), StatusCode::OK);
        let escalate_json = json_body(escalate_response).await;
        assert_eq!(escalate_json["handoff_receipt"]["status"], "escalated");
        assert_eq!(
            escalate_json["handoff_receipt_summary"]["status"],
            "escalated"
        );

        let restarted_app = build_file_backed_app(data_dir.clone());

        let persisted_task_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{SHIFT_TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(persisted_task_response.status(), StatusCode::OK);
        let persisted_task_json = json_body(persisted_task_response).await;
        assert_eq!(
            persisted_task_json["handoff_receipt"]["status"],
            "escalated"
        );

        let persisted_audit_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/audit-events?kind=handoff_receipt_escalated"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("audit query should succeed");
        assert_eq!(persisted_audit_response.status(), StatusCode::OK);
        let persisted_audit_json = json_body(persisted_audit_response).await;
        assert_eq!(
            persisted_audit_json.as_array().expect("audit list").len(),
            1
        );
    }

    #[tokio::test]
    async fn sandbox_safe_alert_triage_cluster_and_follow_up_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-alert-smoke");
        let data_dir = sandbox_dir.path().to_path_buf();
        let app = build_file_backed_app(data_dir.clone());

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-alert-intake-001")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);
        let intake_json = json_body(intake_response).await;
        assert_eq!(
            intake_json["planned_task"]["task"]["status"],
            "awaiting_approval"
        );
        assert_eq!(
            intake_json["follow_up_items"]
                .as_array()
                .expect("follow-up list")
                .len(),
            1
        );
        assert_eq!(intake_json["follow_up_summary"]["total_items"], 1);
        assert_eq!(
            intake_json["alert_cluster_drafts"]
                .as_array()
                .expect("alert cluster list")
                .len(),
            1
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["cluster_status"],
            "open"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["line_id"],
            "line_pack_04"
        );
        assert_eq!(intake_json["alert_triage_summary"]["total_clusters"], 1);

        let restarted_app = build_file_backed_app(data_dir.clone());

        let persisted_task_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{ALERT_TASK_ID}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(persisted_task_response.status(), StatusCode::OK);
        let persisted_task_json = json_body(persisted_task_response).await;
        assert_eq!(
            persisted_task_json["alert_cluster_drafts"]
                .as_array()
                .expect("alert cluster list")
                .len(),
            1
        );
        assert_eq!(
            persisted_task_json["alert_cluster_drafts"][0]["source_system"],
            "andon"
        );
        assert_eq!(
            persisted_task_json["alert_cluster_drafts"][0]["line_id"],
            "line_pack_04"
        );
        assert_eq!(
            persisted_task_json["alert_cluster_drafts"][0]["triage_label"],
            "repeated_alert_review"
        );
        assert_eq!(
            persisted_task_json["follow_up_items"]
                .as_array()
                .expect("follow-up list")
                .len(),
            1
        );
        assert_eq!(persisted_task_json["follow_up_summary"]["total_items"], 1);
        assert_eq!(
            persisted_task_json["follow_up_items"][0]["source_kind"],
            "alert_triage"
        );
        assert_eq!(
            persisted_task_json["alert_triage_summary"]["escalation_candidate_count"],
            1
        );

        let task_file = data_dir.join("tasks").join(format!("{ALERT_TASK_ID}.json"));
        assert!(
            task_file.exists(),
            "alert triage task file should exist in sandbox data dir"
        );
    }

    #[tokio::test]
    async fn sandbox_safe_alert_triage_scada_threshold_cluster_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-alert-scada-threshold-smoke");
        let data_dir = sandbox_dir.path().to_path_buf();
        let app = build_file_backed_app(data_dir.clone());

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-alert-intake-002")
                    .body(Body::from(scada_threshold_alert_request()))
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);
        let intake_json = json_body(intake_response).await;
        assert_eq!(intake_json["planned_task"]["task"]["status"], "approved");
        assert_eq!(
            intake_json["follow_up_items"][0]["recommended_owner_role"],
            "maintenance_engineer"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["source_system"],
            "scada"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["line_id"],
            "line_mix_02"
        );
        assert_eq!(
            intake_json["alert_cluster_drafts"][0]["triage_label"],
            "sustained_threshold_review"
        );

        let restarted_app = build_file_backed_app(data_dir.clone());

        let persisted_task_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/tasks/{ALERT_TASK_ID_2}"))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response should succeed");
        assert_eq!(persisted_task_response.status(), StatusCode::OK);
        let persisted_task_json = json_body(persisted_task_response).await;
        assert_eq!(
            persisted_task_json["alert_cluster_drafts"][0]["source_system"],
            "scada"
        );
        assert_eq!(
            persisted_task_json["alert_cluster_drafts"][0]["line_id"],
            "line_mix_02"
        );
        assert_eq!(
            persisted_task_json["alert_cluster_drafts"][0]["triage_label"],
            "sustained_threshold_review"
        );
        assert_eq!(
            persisted_task_json["follow_up_items"][0]["recommended_owner_role"],
            "maintenance_engineer"
        );

        let task_file = data_dir
            .join("tasks")
            .join(format!("{ALERT_TASK_ID_2}.json"));
        assert!(
            task_file.exists(),
            "scada alert triage task file should exist in sandbox data dir"
        );
    }

    #[tokio::test]
    async fn sandbox_safe_alert_cluster_queue_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-alert-cluster-queue-smoke");
        let data_dir = sandbox_dir.path().to_path_buf();
        let (app, repository) = build_file_backed_app_with_repository(data_dir.clone());

        let intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-alert-cluster-queue-intake-001")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("andon alert intake should succeed");
        assert_eq!(intake_response.status(), StatusCode::OK);
        let intake_json = json_body(intake_response).await;
        let follow_up_id = intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-alert-cluster-queue-intake-002")
                    .body(Body::from(scada_threshold_alert_request()))
                    .expect("request should build"),
            )
            .await
            .expect("scada alert intake should succeed");

        let accept_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{ALERT_TASK_ID}/follow-up-items/{follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-alert-cluster-queue-accept-001")
                    .body(Body::from(alert_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up accept should succeed");
        assert_eq!(accept_response.status(), StatusCode::OK);

        let now = Utc::now();
        let window_from = (now - chrono::Duration::minutes(12))
            .to_rfc3339()
            .replace("+00:00", "Z");
        let window_to = (now - chrono::Duration::minutes(2))
            .to_rfc3339()
            .replace("+00:00", "Z");

        let mut andon_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID).expect("alert task id should parse"))
            .expect("andon task lookup should succeed")
            .expect("andon task should exist");
        let cluster_id = andon_state.alert_cluster_drafts[0].cluster_id.clone();
        andon_state.alert_cluster_drafts[0].window_start = now - chrono::Duration::minutes(30);
        andon_state.alert_cluster_drafts[0].window_end = now - chrono::Duration::minutes(25);
        andon_state.follow_up_items[0].source_kind = "alert_cluster".to_string();
        andon_state.follow_up_items[0].source_refs = vec![cluster_id];
        andon_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        repository
            .save(andon_state)
            .expect("andon task save should succeed");

        let mut scada_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID_2).expect("scada task id should parse"))
            .expect("scada task lookup should succeed")
            .expect("scada task should exist");
        scada_state.alert_cluster_drafts[0].cluster_status = "closed".to_string();
        scada_state.alert_cluster_drafts[0].window_start = now - chrono::Duration::minutes(10);
        scada_state.alert_cluster_drafts[0].window_end = now - chrono::Duration::minutes(5);
        repository
            .save(scada_state)
            .expect("scada task save should succeed");

        let restarted_app = build_file_backed_app(data_dir.clone());

        let queue_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("alert cluster queue request should succeed");
        assert_eq!(queue_response.status(), StatusCode::OK);
        let queue_json = json_body(queue_response).await;
        let items = queue_json.as_array().expect("alert cluster queue list");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["task_id"], ALERT_TASK_ID);
        assert_eq!(items[0]["source_system"], "andon");
        assert_eq!(items[0]["linked_follow_up"]["total_items"], 1);
        assert_eq!(items[0]["linked_follow_up"]["accepted_items"], 1);
        assert_eq!(
            items[0]["linked_follow_up"]["accepted_owner_ids"][0],
            "worker_1001"
        );
        assert_eq!(
            items[0]["linked_follow_up"]["worst_effective_sla_status"],
            "escalation_required"
        );
        assert_eq!(items[1]["task_id"], ALERT_TASK_ID_2);
        assert_eq!(items[1]["cluster_status"], "closed");

        let source_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?source_system=scada")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("source-filtered alert cluster queue request should succeed");
        assert_eq!(source_response.status(), StatusCode::OK);
        let source_json = json_body(source_response).await;
        let source_items = source_json.as_array().expect("source queue list");
        assert_eq!(source_items.len(), 1);
        assert_eq!(source_items[0]["task_id"], ALERT_TASK_ID_2);

        let open_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?open_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("open-only alert cluster queue request should succeed");
        assert_eq!(open_response.status(), StatusCode::OK);
        let open_json = json_body(open_response).await;
        let open_items = open_json.as_array().expect("open queue list");
        assert_eq!(open_items.len(), 1);
        assert_eq!(open_items[0]["task_id"], ALERT_TASK_ID);

        let owner_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?follow_up_owner_id=worker_1001")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("owner-filtered alert cluster queue request should succeed");
        assert_eq!(owner_response.status(), StatusCode::OK);
        let owner_json = json_body(owner_response).await;
        let owner_items = owner_json.as_array().expect("owner queue list");
        assert_eq!(owner_items.len(), 1);
        assert_eq!(owner_items[0]["task_id"], ALERT_TASK_ID);

        let unaccepted_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?unaccepted_follow_up_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("unaccepted alert cluster queue request should succeed");
        assert_eq!(unaccepted_response.status(), StatusCode::OK);
        let unaccepted_json = json_body(unaccepted_response).await;
        let unaccepted_items = unaccepted_json.as_array().expect("unaccepted queue list");
        assert_eq!(unaccepted_items.len(), 1);
        assert_eq!(unaccepted_items[0]["task_id"], ALERT_TASK_ID_2);

        let escalation_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-clusters?follow_up_escalation_required=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("follow-up escalation alert cluster queue request should succeed");
        assert_eq!(escalation_response.status(), StatusCode::OK);
        let escalation_json = json_body(escalation_response).await;
        let escalation_items = escalation_json.as_array().expect("escalation queue list");
        assert_eq!(escalation_items.len(), 1);
        assert_eq!(escalation_items[0]["task_id"], ALERT_TASK_ID);

        let window_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/alert-clusters?window_from={window_from}&window_to={window_to}"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("window-filtered alert cluster queue request should succeed");
        assert_eq!(window_response.status(), StatusCode::OK);
        let window_json = json_body(window_response).await;
        let window_items = window_json.as_array().expect("window queue list");
        assert_eq!(window_items.len(), 1);
        assert_eq!(window_items[0]["task_id"], ALERT_TASK_ID_2);
    }

    #[tokio::test]
    async fn sandbox_safe_alert_cluster_monitoring_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-alert-cluster-monitoring-smoke");
        let data_dir = sandbox_dir.path().to_path_buf();
        let (app, repository) = build_file_backed_app_with_repository(data_dir.clone());

        let andon_intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "sandbox-alert-cluster-monitoring-intake-001",
                    )
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("andon alert intake should succeed");
        assert_eq!(andon_intake_response.status(), StatusCode::OK);
        let andon_intake_json = json_body(andon_intake_response).await;
        let andon_follow_up_id = andon_intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "sandbox-alert-cluster-monitoring-intake-002",
                    )
                    .body(Body::from(scada_threshold_alert_request()))
                    .expect("request should build"),
            )
            .await
            .expect("scada alert intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-alert-cluster-monitoring-intake-003")
                    .body(Body::from(scada_threshold_alert_request_with_id(
                        ALERT_TASK_ID_3,
                        "Triage reserve threshold alert on mix line 3",
                        "Review reserve SCADA threshold breach on mix line 3 for the next planned batch.",
                        "eq_mix_03",
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("future scada alert intake should succeed");

        let now = Utc::now();
        let expected_next_window_end = now - chrono::Duration::minutes(25);

        let accept_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{ALERT_TASK_ID}/follow-up-items/{andon_follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "sandbox-alert-cluster-monitoring-accept-001",
                    )
                    .body(Body::from(alert_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up accept should succeed");
        assert_eq!(accept_response.status(), StatusCode::OK);

        let mut andon_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID).expect("alert task id should parse"))
            .expect("andon task lookup should succeed")
            .expect("andon task should exist");
        let cluster_id = andon_state.alert_cluster_drafts[0].cluster_id.clone();
        andon_state.alert_cluster_drafts[0].window_start = now - chrono::Duration::minutes(30);
        andon_state.alert_cluster_drafts[0].window_end = expected_next_window_end;
        andon_state.follow_up_items[0].source_kind = "alert_cluster".to_string();
        andon_state.follow_up_items[0].source_refs = vec![cluster_id];
        andon_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        repository
            .save(andon_state)
            .expect("andon task save should succeed");

        let mut scada_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID_2).expect("scada task id should parse"))
            .expect("scada task lookup should succeed")
            .expect("scada task should exist");
        scada_state.alert_cluster_drafts[0].window_start = now - chrono::Duration::minutes(5);
        scada_state.alert_cluster_drafts[0].window_end = now + chrono::Duration::minutes(10);
        repository
            .save(scada_state)
            .expect("scada task save should succeed");

        let mut future_state = repository
            .get(Uuid::parse_str(ALERT_TASK_ID_3).expect("future task id should parse"))
            .expect("future task lookup should succeed")
            .expect("future task should exist");
        future_state.alert_cluster_drafts[0].cluster_status = "closed".to_string();
        future_state.alert_cluster_drafts[0].window_start = now + chrono::Duration::minutes(20);
        future_state.alert_cluster_drafts[0].window_end = now + chrono::Duration::minutes(35);
        repository
            .save(future_state)
            .expect("future task save should succeed");

        let restarted_app = build_file_backed_app(data_dir.clone());

        let monitoring_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-cluster-monitoring")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("monitoring request should succeed");
        assert_eq!(monitoring_response.status(), StatusCode::OK);
        let monitoring_json = json_body(monitoring_response).await;

        assert_eq!(monitoring_json["total_clusters"], 3);
        assert_eq!(monitoring_json["open_clusters"], 2);
        assert_eq!(monitoring_json["escalation_candidate_clusters"], 1);
        assert_eq!(monitoring_json["active_window_clusters"], 1);
        assert_eq!(monitoring_json["stale_window_clusters"], 1);
        assert_eq!(monitoring_json["linked_follow_up_clusters"], 3);
        assert_eq!(monitoring_json["unlinked_follow_up_clusters"], 0);
        assert_eq!(monitoring_json["accepted_follow_up_clusters"], 1);
        assert_eq!(monitoring_json["unaccepted_follow_up_clusters"], 2);
        assert_eq!(monitoring_json["follow_up_escalation_clusters"], 1);
        assert_eq!(
            DateTime::parse_from_rfc3339(
                monitoring_json["next_window_end_at"]
                    .as_str()
                    .expect("next_window_end_at should exist"),
            )
            .expect("next_window_end_at should parse")
            .with_timezone(&Utc),
            expected_next_window_end
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "source_system_counts", "scada"),
            2
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "follow_up_coverage_counts", "linked"),
            3
        );
        assert_eq!(
            json_bucket_count(
                &monitoring_json,
                "follow_up_sla_status_counts",
                "escalation_required"
            ),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "follow_up_sla_status_counts", "due_soon"),
            2
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "window_state_counts", "future"),
            1
        );

        let filtered_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-cluster-monitoring?open_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("filtered monitoring request should succeed");
        assert_eq!(filtered_response.status(), StatusCode::OK);
        let filtered_json = json_body(filtered_response).await;

        assert_eq!(filtered_json["total_clusters"], 2);
        assert_eq!(filtered_json["open_clusters"], 2);
        assert_eq!(filtered_json["escalation_candidate_clusters"], 1);
        assert_eq!(filtered_json["linked_follow_up_clusters"], 2);
        assert_eq!(filtered_json["unlinked_follow_up_clusters"], 0);
        assert_eq!(filtered_json["accepted_follow_up_clusters"], 1);
        assert_eq!(filtered_json["unaccepted_follow_up_clusters"], 1);
        assert_eq!(filtered_json["follow_up_escalation_clusters"], 1);
        assert_eq!(
            json_bucket_count(&filtered_json, "cluster_status_counts", "open"),
            2
        );
        assert_eq!(
            json_bucket_count(&filtered_json, "window_state_counts", "stale"),
            1
        );
        assert_eq!(
            json_bucket_count(&filtered_json, "follow_up_coverage_counts", "linked"),
            2
        );
        assert_eq!(
            json_bucket_count(
                &filtered_json,
                "follow_up_sla_status_counts",
                "escalation_required"
            ),
            1
        );

        let escalation_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-cluster-monitoring?follow_up_escalation_required=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("follow-up escalation monitoring request should succeed");
        assert_eq!(escalation_response.status(), StatusCode::OK);
        let escalation_json = json_body(escalation_response).await;
        assert_eq!(escalation_json["total_clusters"], 1);
        assert_eq!(
            json_bucket_count(&escalation_json, "source_system_counts", "andon"),
            1
        );

        let unaccepted_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/alert-cluster-monitoring?unaccepted_follow_up_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("unaccepted monitoring request should succeed");
        assert_eq!(unaccepted_response.status(), StatusCode::OK);
        let unaccepted_json = json_body(unaccepted_response).await;
        assert_eq!(unaccepted_json["total_clusters"], 2);
        assert_eq!(
            json_bucket_count(&unaccepted_json, "source_system_counts", "scada"),
            2
        );
    }

    #[tokio::test]
    async fn sandbox_safe_follow_up_queue_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-follow-up-queue-smoke");
        let data_dir = sandbox_dir.path().to_path_buf();
        let app = build_file_backed_app(data_dir.clone());

        let shift_intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-follow-up-queue-intake-001")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("shift handoff intake should succeed");
        let shift_intake_json = json_body(shift_intake_response).await;
        let follow_up_id = shift_intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/follow-up-items/{follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-follow-up-queue-accept-001")
                    .body(Body::from(shift_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up acceptance should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-follow-up-queue-intake-002")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("alert triage intake should succeed");

        let restarted_app = build_file_backed_app(data_dir.clone());

        let queue_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-items")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("queue request should succeed");
        assert_eq!(queue_response.status(), StatusCode::OK);
        let queue_json = json_body(queue_response).await;
        let items = queue_json.as_array().expect("queue list");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["task_id"], ALERT_TASK_ID);
        assert_eq!(items[0]["source_kind"], "alert_triage");
        assert_eq!(items[1]["task_id"], SHIFT_TASK_ID);
        assert_eq!(items[1]["accepted_owner_id"], "worker_1101");

        let owner_queue_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-items?owner_id=worker_1101")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("owner queue request should succeed");
        assert_eq!(owner_queue_response.status(), StatusCode::OK);
        let owner_queue_json = json_body(owner_queue_response).await;
        let owner_items = owner_queue_json.as_array().expect("owner queue list");

        assert_eq!(owner_items.len(), 1);
        assert_eq!(owner_items[0]["task_id"], SHIFT_TASK_ID);
        assert_eq!(owner_items[0]["status"], "accepted");
        assert_eq!(owner_items[0]["accepted_owner_id"], "worker_1101");

        let source_queue_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-items?source_kind=alert_triage")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("source queue request should succeed");
        assert_eq!(source_queue_response.status(), StatusCode::OK);
        let source_queue_json = json_body(source_queue_response).await;
        let source_items = source_queue_json.as_array().expect("source queue list");

        assert_eq!(source_items.len(), 1);
        assert_eq!(source_items[0]["task_id"], ALERT_TASK_ID);
        assert_eq!(source_items[0]["source_kind"], "alert_triage");
    }

    #[tokio::test]
    async fn sandbox_safe_follow_up_queue_triage_filters_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-follow-up-queue-triage-smoke");
        let data_dir = sandbox_dir.path().to_path_buf();
        let (app, repository) = build_file_backed_app_with_repository(data_dir.clone());

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-follow-up-queue-intake-003")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("shift handoff intake should succeed");

        let alert_intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-follow-up-queue-intake-004")
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("alert triage intake should succeed");
        let alert_intake_json = json_body(alert_intake_response).await;
        let due_before = alert_intake_json["follow_up_items"][0]["due_at"]
            .as_str()
            .expect("alert triage due_at should exist")
            .to_string();

        let now = Utc::now();

        let shift_task_id = Uuid::parse_str(SHIFT_TASK_ID).expect("shift task id should parse");
        let mut shift_state = repository
            .get(shift_task_id)
            .expect("shift task lookup should succeed")
            .expect("shift task should exist");
        shift_state.follow_up_items[0].status = "blocked".to_string();
        shift_state.follow_up_items[0].blocked_reason =
            Some("Waiting for outgoing shift clarification.".to_string());
        shift_state.follow_up_items[0].due_at = Some(now + chrono::Duration::minutes(20));
        repository
            .save(shift_state)
            .expect("shift task save should succeed");

        let alert_task_id = Uuid::parse_str(ALERT_TASK_ID).expect("alert task id should parse");
        let mut alert_state = repository
            .get(alert_task_id)
            .expect("alert task lookup should succeed")
            .expect("alert task should exist");
        alert_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        alert_state.follow_up_items[0].due_at = Some(now + chrono::Duration::minutes(10));
        repository
            .save(alert_state)
            .expect("alert task save should succeed");

        let restarted_app = build_file_backed_app(data_dir.clone());

        let triage_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/follow-up-items?risk=high&priority=expedited&due_before={due_before}"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("triage queue request should succeed");
        assert_eq!(triage_response.status(), StatusCode::OK);
        let triage_json = json_body(triage_response).await;
        let triage_items = triage_json.as_array().expect("triage queue list");

        assert_eq!(triage_items.len(), 1);
        assert_eq!(triage_items[0]["task_id"], ALERT_TASK_ID);
        assert_eq!(triage_items[0]["task_risk"], "high");
        assert_eq!(triage_items[0]["task_priority"], "expedited");

        let blocked_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-items?blocked_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("blocked queue request should succeed");
        assert_eq!(blocked_response.status(), StatusCode::OK);
        let blocked_json = json_body(blocked_response).await;
        let blocked_items = blocked_json.as_array().expect("blocked queue list");

        assert_eq!(blocked_items.len(), 1);
        assert_eq!(blocked_items[0]["task_id"], SHIFT_TASK_ID);
        assert_eq!(blocked_items[0]["status"], "blocked");

        let escalation_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-items?escalation_required=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("escalation queue request should succeed");
        assert_eq!(escalation_response.status(), StatusCode::OK);
        let escalation_json = json_body(escalation_response).await;
        let escalation_items = escalation_json.as_array().expect("escalation queue list");

        assert_eq!(escalation_items.len(), 1);
        assert_eq!(escalation_items[0]["task_id"], ALERT_TASK_ID);
        assert_eq!(
            escalation_items[0]["effective_sla_status"],
            "escalation_required"
        );
    }

    #[tokio::test]
    async fn sandbox_safe_handoff_receipt_queue_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-handoff-receipt-queue-smoke");
        let data_dir = sandbox_dir.path().to_path_buf();
        let (app, repository) = build_file_backed_app_with_repository(data_dir.clone());

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "sandbox-handoff-receipt-queue-intake-001",
                    )
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("first handoff intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "sandbox-handoff-receipt-queue-intake-002",
                    )
                    .body(Body::from(shift_handoff_request_with_id(
                        SHIFT_TASK_ID_2,
                        "Summarize packaging handoff notes",
                        "Summarize packaging line handoff notes for the next shift.",
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("second handoff intake should succeed");

        let now = Utc::now();

        let first_task_id = Uuid::parse_str(SHIFT_TASK_ID).expect("shift task id should parse");
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
            .required_ack_by = Some(now - chrono::Duration::minutes(5));
        repository
            .save(first_state)
            .expect("first task save should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID_2}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-handoff-receipt-queue-ack-001")
                    .body(Body::from(handoff_ack_with_exception_request()))
                    .expect("request should build"),
            )
            .await
            .expect("second receipt acknowledgement should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID_2}/handoff-receipt/escalate"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "sandbox-handoff-receipt-queue-escalate-001",
                    )
                    .body(Body::from(handoff_escalation_request()))
                    .expect("request should build"),
            )
            .await
            .expect("second receipt escalation should succeed");

        let restarted_app = build_file_backed_app(data_dir.clone());

        let queue_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipts")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("receipt queue request should succeed");
        assert_eq!(queue_response.status(), StatusCode::OK);
        let queue_json = json_body(queue_response).await;
        let items = queue_json.as_array().expect("receipt queue list");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["task_id"], SHIFT_TASK_ID);
        assert_eq!(items[0]["effective_status"], "expired");
        assert_eq!(items[1]["task_id"], SHIFT_TASK_ID_2);
        assert_eq!(items[1]["effective_status"], "escalated");

        let overdue_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipts?overdue_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("overdue queue request should succeed");
        assert_eq!(overdue_response.status(), StatusCode::OK);
        let overdue_json = json_body(overdue_response).await;
        let overdue_items = overdue_json.as_array().expect("overdue receipt queue list");
        assert_eq!(overdue_items.len(), 1);
        assert_eq!(overdue_items[0]["task_id"], SHIFT_TASK_ID);

        let exceptions_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipts?has_exceptions=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("exceptions queue request should succeed");
        assert_eq!(exceptions_response.status(), StatusCode::OK);
        let exceptions_json = json_body(exceptions_response).await;
        let exceptions_items = exceptions_json
            .as_array()
            .expect("exceptions receipt queue list");
        assert_eq!(exceptions_items.len(), 1);
        assert_eq!(exceptions_items[0]["task_id"], SHIFT_TASK_ID_2);

        let shift_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/handoff-receipts?shift_id={first_shift_id}"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("shift-specific queue request should succeed");
        assert_eq!(shift_response.status(), StatusCode::OK);
        let shift_json = json_body(shift_response).await;
        let shift_items = shift_json.as_array().expect("shift receipt queue list");
        assert_eq!(shift_items.len(), 1);
        assert_eq!(shift_items[0]["task_id"], SHIFT_TASK_ID);

        let escalated_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipts?escalated_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("escalated queue request should succeed");
        assert_eq!(escalated_response.status(), StatusCode::OK);
        let escalated_json = json_body(escalated_response).await;
        let escalated_items = escalated_json
            .as_array()
            .expect("escalated receipt queue list");
        assert_eq!(escalated_items.len(), 1);
        assert_eq!(escalated_items[0]["task_id"], SHIFT_TASK_ID_2);
    }

    #[tokio::test]
    async fn sandbox_safe_follow_up_monitoring_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-follow-up-monitoring-smoke");
        let data_dir = sandbox_dir.path().to_path_buf();
        let (app, repository) = build_file_backed_app_with_repository(data_dir.clone());

        let shift_intake_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "sandbox-follow-up-monitoring-intake-001",
                    )
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("shift handoff intake should succeed");
        let shift_intake_json = json_body(shift_intake_response).await;
        let follow_up_id = shift_intake_json["follow_up_items"][0]["id"]
            .as_str()
            .expect("follow-up id should exist")
            .to_string();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID}/follow-up-items/{follow_up_id}/accept-owner"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "sandbox-follow-up-monitoring-accept-001",
                    )
                    .body(Body::from(shift_follow_up_accept_request()))
                    .expect("request should build"),
            )
            .await
            .expect("follow-up acceptance should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "sandbox-follow-up-monitoring-intake-002",
                    )
                    .body(Body::from(alert_triage_request()))
                    .expect("request should build"),
            )
            .await
            .expect("alert triage intake should succeed");

        let now = Utc::now();
        let expected_next_due_at = now + chrono::Duration::minutes(10);

        let shift_task_id = Uuid::parse_str(SHIFT_TASK_ID).expect("shift task id should parse");
        let mut shift_state = repository
            .get(shift_task_id)
            .expect("shift task lookup should succeed")
            .expect("shift task should exist");
        shift_state.follow_up_items[0].status = "blocked".to_string();
        shift_state.follow_up_items[0].blocked_reason =
            Some("Waiting for outgoing shift clarification.".to_string());
        shift_state.follow_up_items[0].due_at = Some(now + chrono::Duration::minutes(20));
        repository
            .save(shift_state)
            .expect("shift task save should succeed");

        let alert_task_id = Uuid::parse_str(ALERT_TASK_ID).expect("alert task id should parse");
        let mut alert_state = repository
            .get(alert_task_id)
            .expect("alert task lookup should succeed")
            .expect("alert task should exist");
        alert_state.follow_up_items[0].sla_status = "escalation_required".to_string();
        alert_state.follow_up_items[0].due_at = Some(expected_next_due_at);
        repository
            .save(alert_state)
            .expect("alert task save should succeed");

        let restarted_app = build_file_backed_app(data_dir.clone());

        let monitoring_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-monitoring")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("monitoring request should succeed");
        assert_eq!(monitoring_response.status(), StatusCode::OK);
        let monitoring_json = json_body(monitoring_response).await;

        assert_eq!(monitoring_json["total_items"], 2);
        assert_eq!(monitoring_json["accepted_items"], 1);
        assert_eq!(monitoring_json["blocked_items"], 1);
        assert_eq!(monitoring_json["escalation_required_items"], 1);
        assert_eq!(
            DateTime::parse_from_rfc3339(
                monitoring_json["next_due_at"]
                    .as_str()
                    .expect("next_due_at should exist"),
            )
            .expect("next_due_at should parse")
            .with_timezone(&Utc),
            expected_next_due_at
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "source_kind_counts", "alert_triage"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "source_kind_counts", "shift_handoff"),
            1
        );

        let filtered_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/follow-up-monitoring?source_kind=alert_triage")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("filtered monitoring request should succeed");
        assert_eq!(filtered_response.status(), StatusCode::OK);
        let filtered_json = json_body(filtered_response).await;

        assert_eq!(filtered_json["total_items"], 1);
        assert_eq!(filtered_json["open_items"], 1);
        assert_eq!(filtered_json["accepted_items"], 0);
        assert_eq!(filtered_json["escalation_required_items"], 1);
        assert_eq!(
            json_bucket_count(&filtered_json, "owner_role_counts", "production_supervisor"),
            1
        );
    }

    #[tokio::test]
    async fn sandbox_safe_handoff_receipt_monitoring_smoke_works_end_to_end() {
        let sandbox_dir = SandboxDirGuard::new("fa-v0.2.0-sandbox-handoff-receipt-monitoring");
        let data_dir = sandbox_dir.path().to_path_buf();
        let (app, repository) = build_file_backed_app_with_repository(data_dir.clone());

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-handoff-monitoring-intake-001")
                    .body(Body::from(shift_handoff_request()))
                    .expect("request should build"),
            )
            .await
            .expect("first handoff intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-handoff-monitoring-intake-002")
                    .body(Body::from(shift_handoff_request_with_id(
                        SHIFT_TASK_ID_2,
                        "Summarize packaging handoff notes",
                        "Summarize packaging line handoff notes for the next shift.",
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("second handoff intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-handoff-monitoring-intake-003")
                    .body(Body::from(shift_handoff_request_with_id(
                        SHIFT_TASK_ID_3,
                        "Summarize assembly handoff notes",
                        "Summarize assembly line handoff notes for the next shift.",
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("third handoff intake should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/intake")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-handoff-monitoring-intake-004")
                    .body(Body::from(shift_handoff_request_with_id(
                        SHIFT_TASK_ID_4,
                        "Summarize paint handoff notes",
                        "Summarize paint line handoff notes for the next shift.",
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("fourth handoff intake should succeed");

        let now = Utc::now();
        let expected_next_ack_due_at = now - chrono::Duration::minutes(5);

        let first_task_id = Uuid::parse_str(SHIFT_TASK_ID).expect("shift task id should parse");
        let mut first_state = repository
            .get(first_task_id)
            .expect("first task lookup should succeed")
            .expect("first task should exist");
        first_state
            .handoff_receipt
            .as_mut()
            .expect("handoff receipt should exist")
            .required_ack_by = Some(expected_next_ack_due_at);
        repository
            .save(first_state)
            .expect("first task save should succeed");

        let second_task_id =
            Uuid::parse_str(SHIFT_TASK_ID_2).expect("second shift task id should parse");
        let mut second_state = repository
            .get(second_task_id)
            .expect("second task lookup should succeed")
            .expect("second task should exist");
        second_state
            .handoff_receipt
            .as_mut()
            .expect("handoff receipt should exist")
            .required_ack_by = Some(now + chrono::Duration::minutes(20));
        repository
            .save(second_state)
            .expect("second task save should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID_3}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-handoff-monitoring-ack-001")
                    .body(Body::from(handoff_ack_with_exception_request()))
                    .expect("request should build"),
            )
            .await
            .expect("third receipt acknowledgement should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID_4}/handoff-receipt/acknowledge"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("x-correlation-id", "sandbox-handoff-monitoring-ack-002")
                    .body(Body::from(handoff_ack_with_exception_request()))
                    .expect("request should build"),
            )
            .await
            .expect("fourth receipt acknowledgement should succeed");

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/tasks/{SHIFT_TASK_ID_4}/handoff-receipt/escalate"
                    ))
                    .method("POST")
                    .header("content-type", "application/json")
                    .header(
                        "x-correlation-id",
                        "sandbox-handoff-monitoring-escalate-001",
                    )
                    .body(Body::from(handoff_escalation_request()))
                    .expect("request should build"),
            )
            .await
            .expect("fourth receipt escalation should succeed");

        let restarted_app = build_file_backed_app(data_dir.clone());

        let monitoring_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipt-monitoring")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("monitoring request should succeed");
        assert_eq!(monitoring_response.status(), StatusCode::OK);
        let monitoring_json = json_body(monitoring_response).await;

        assert_eq!(monitoring_json["total_receipts"], 4);
        assert_eq!(monitoring_json["open_receipts"], 4);
        assert_eq!(monitoring_json["acknowledged_receipts"], 2);
        assert_eq!(monitoring_json["unacknowledged_receipts"], 2);
        assert_eq!(monitoring_json["overdue_receipts"], 1);
        assert_eq!(monitoring_json["exception_receipts"], 2);
        assert_eq!(monitoring_json["escalated_receipts"], 1);
        assert_eq!(
            DateTime::parse_from_rfc3339(
                monitoring_json["next_ack_due_at"]
                    .as_str()
                    .expect("next_ack_due_at should exist"),
            )
            .expect("next_ack_due_at should parse")
            .with_timezone(&Utc),
            expected_next_ack_due_at
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "effective_status_counts", "expired"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "effective_status_counts", "escalated"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "ack_window_counts", "overdue"),
            1
        );
        assert_eq!(
            json_bucket_count(&monitoring_json, "ack_window_counts", "due_within_30m"),
            1
        );

        let filtered_response = restarted_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/handoff-receipt-monitoring?escalated_only=true")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("filtered monitoring request should succeed");
        assert_eq!(filtered_response.status(), StatusCode::OK);
        let filtered_json = json_body(filtered_response).await;

        assert_eq!(filtered_json["total_receipts"], 1);
        assert_eq!(filtered_json["open_receipts"], 1);
        assert_eq!(filtered_json["escalated_receipts"], 1);
        assert!(filtered_json["next_ack_due_at"].is_null());
    }
}
