use std::{env, net::SocketAddr};

use anyhow::Context;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use fa_core::{bootstrap_blueprint, InMemoryAuditSink, WorkOrchestrator};
use fa_domain::TaskRequest;
use serde_json::json;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    orchestrator: WorkOrchestrator,
    audit_sink: Arc<InMemoryAuditSink>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let address = env::var("FA_SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8000".to_string());
    let socket_addr: SocketAddr = address
        .parse()
        .with_context(|| format!("invalid FA_SERVER_ADDR: {address}"))?;

    let audit_sink = Arc::new(InMemoryAuditSink::default());
    let state = AppState {
        orchestrator: WorkOrchestrator::with_m1_defaults(audit_sink.clone()),
        audit_sink,
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/api/v1/blueprint", get(blueprint))
        .route("/api/v1/audit/events", get(audit_events))
        .route("/api/v1/tasks/intake", post(intake_task))
        .route("/api/v1/tasks/plan", post(plan_task))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

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
        .map_err(|error| {
            let status = if error.downcast_ref::<fa_domain::LifecycleError>().is_some() {
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(json!({
                    "error": error.to_string(),
                })),
            )
        })
}

async fn audit_events(
    State(state): State<AppState>,
) -> Result<Json<Vec<fa_core::AuditEvent>>, (StatusCode, Json<serde_json::Value>)> {
    state.audit_sink.snapshot().map(Json).map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": error.to_string(),
            })),
        )
    })
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
