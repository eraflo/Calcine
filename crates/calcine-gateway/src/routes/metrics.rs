//! `/metrics`: usage counters for Prometheus and Grafana.

use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;

use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/metrics", get(metrics))
}

/// Any caller the guard let in may read the counters (API key needed unless
/// that's turned off): they hold names of apps and models, never content.
async fn metrics(State(app): State<Arc<AppState>>) -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        app.usage.prometheus(),
    )
}
