use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use axum_prometheus::{metrics_exporter_prometheus::PrometheusBuilder, PrometheusMetricLayer};
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[derive(Serialize)]
struct MessageResponse {
    service: &'static str,
    message: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[tokio::main]
async fn main() {
    // Logs structures JSON sur stdout (collectes par le pod_logs_via_loki de
    // l'add-on grafana-k8s-monitoring, cf. platform-gitops). Niveau par
    // defaut "info", surchargable via RUST_LOG.
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    // `install_recorder` (pas `PrometheusMetricLayer::pair`) : enregistre le
    // recorder Prometheus en process sans ouvrir de listener HTTP dedie, on
    // sert nous-memes /metrics via la route Axum ci-dessous.
    let metric_layer = PrometheusMetricLayer::new();
    let metric_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");

    let app = Router::new()
        .route("/", get(root))
        .route("/hello/:name", get(hello))
        .route("/health", get(health))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(metric_layer)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind API listener");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("API server failed");
}

async fn root() -> impl IntoResponse {
    Json(MessageResponse {
        service: "helloworld-svc",
        message: "Hello, World!".to_string(),
    })
}

async fn hello(Path(name): Path<String>) -> Result<Json<MessageResponse>, StatusCode> {
    let name = name.trim();
    if name.len() > 80 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let name = if name.is_empty() { "World" } else { name };
    Ok(Json(MessageResponse {
        service: "helloworld-svc",
        message: format!("Hello, {name}!"),
    }))
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
