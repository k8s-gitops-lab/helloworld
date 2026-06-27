use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;

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
    let app = Router::new()
        .route("/", get(root))
        .route("/hello/:name", get(hello))
        .route("/health", get(health));

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
