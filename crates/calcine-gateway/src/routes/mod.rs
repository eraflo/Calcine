mod manage;
mod ollama;
mod openai;

use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::middleware;
use axum::routing::get;
use axum::{Extension, Router};

use crate::error::api_error;
use crate::security::{self, Listener};
use crate::state::AppState;

/// Base64 images and audio can be large.
const BODY_LIMIT: usize = 64 * 1024 * 1024;

/// The API served on `listener`. The Ollama port also answers `/` like
/// Ollama, so clients recognize it.
pub fn router(app: Arc<AppState>, listener: Listener) -> Router {
    let routes = Router::new()
        .merge(openai::router())
        .merge(manage::router())
        .merge(ollama::router());
    let routes = match listener {
        Listener::Main | Listener::Network { .. } => routes,
        Listener::Ollama { .. } => routes.route("/", get(ollama::root)),
    };
    routes
        .fallback(|| async {
            api_error(
                StatusCode::NOT_FOUND,
                "invalid_request_error",
                "unknown endpoint",
            )
        })
        .layer(middleware::from_fn_with_state(app.clone(), security::guard))
        // Outside the guard, so it knows which socket the request came in on.
        .layer(Extension(listener))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .with_state(app)
}
