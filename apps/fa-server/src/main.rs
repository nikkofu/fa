use std::{env, net::SocketAddr};

use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use fa_core::{
    bootstrap_blueprint, ApprovalActionRequest, AuditEventKind, AuditEventQuery, AuditStore,
    CompleteTaskRequest, ExecuteTaskRequest, FailTaskRequest, FileAuditStore, FileTaskRepository,
    InMemoryAuditSink, InMemoryTaskRepository, OrchestrationError, ResubmitTaskRequest,
    TrackedTaskState, WorkOrchestrator,
};
use fa_domain::TaskRequest;
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
        OrchestrationError::Lifecycle(_) => StatusCode::UNPROCESSABLE_ENTITY,
        OrchestrationError::TaskAlreadyExists(_) => StatusCode::CONFLICT,
        OrchestrationError::TaskNotFound(_) | OrchestrationError::ApprovalNotFound(_) => {
            StatusCode::NOT_FOUND
        }
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
    if let Some(data_dir) = env::var_os("FA_DATA_DIR") {
        let data_dir = std::path::PathBuf::from(data_dir);
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
        .route("/healthz", get(healthz))
        .route("/api/v1/blueprint", get(blueprint))
        .route("/api/v1/audit/events", get(audit_events))
        .route("/api/v1/tasks/intake", post(intake_task))
        .route("/api/v1/tasks/plan", post(plan_task))
        .route("/api/v1/tasks/{task_id}", get(get_task))
        .route(
            "/api/v1/tasks/{task_id}/audit-events",
            get(task_audit_events),
        )
        .route("/api/v1/tasks/{task_id}/approve", post(approve_task))
        .route("/api/v1/tasks/{task_id}/resubmit", post(resubmit_task))
        .route("/api/v1/tasks/{task_id}/execute", post(execute_task))
        .route("/api/v1/tasks/{task_id}/complete", post(complete_task))
        .route("/api/v1/tasks/{task_id}/fail", post(fail_task))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use super::*;

    const TASK_ID: &str = "72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0";

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

    async fn json_body(response: axum::response::Response) -> serde_json::Value {
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should read");
        serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            panic!("failed to decode JSON body with status {status}: {error}")
        })
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
                            "decided_by":{"id":"worker_2001","display_name":"Chen QE","role":"Quality Engineer"},
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
                            "decided_by":{"id":"worker_2001","display_name":"Chen QE","role":"Quality Engineer"},
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
                            "decided_by":{"id":"worker_2001","display_name":"Chen QE","role":"Quality Engineer"},
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
                            "decided_by":{"id":"worker_2001","display_name":"Chen QE","role":"Quality Engineer"},
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
                            "decided_by":{"id":"worker_2001","display_name":"Chen QE","role":"Quality Engineer"},
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
}
