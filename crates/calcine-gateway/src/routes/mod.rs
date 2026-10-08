mod manage;
mod openai;

use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::middleware;

use crate::error::api_error;
use crate::security;
use crate::state::AppState;

/// Base64 images and audio can be large.
const BODY_LIMIT: usize = 64 * 1024 * 1024;

pub fn router(app: Arc<AppState>) -> Router {
    Router::new()
        .merge(openai::router())
        .merge(manage::router())
        .fallback(|| async {
            api_error(
                StatusCode::NOT_FOUND,
                "invalid_request_error",
                "unknown endpoint",
            )
        })
        .layer(middleware::from_fn_with_state(app.clone(), security::guard))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .with_state(app)
}
