pub mod client;
pub mod config;
pub mod metrics;

use std::sync::Arc;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use serde_json::json;

pub use client::QingpingClient;
pub use config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub client: Arc<QingpingClient>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let client = Arc::new(QingpingClient::new(
            &config.app_key,
            &config.app_secret,
            config.cache_ttl_secs,
        ));
        Self { config, client }
    }
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/healthz", get(healthz_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}

pub async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "name": "qinping-air-monitor",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running"
    }))
}

pub async fn healthz_handler() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

pub async fn metrics_handler(State(state): State<AppState>) -> Response {
    let result = state.client.fetch_devices().await;
    let text = metrics::render_prometheus_metrics(&result);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")
        .body(axum::body::Body::from(text))
        .unwrap()
}
