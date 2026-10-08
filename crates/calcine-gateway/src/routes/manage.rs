//! Calcine's own management API under `/calcine/v1` (keys with `manage`).

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use calcine_core::models::{ModelKey, ModelReference, PullRequest};
use serde::Deserialize;
use serde_json::json;

use crate::caller::Caller;
use crate::error::api_error;
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/calcine/v1/status", get(status))
        .route("/calcine/v1/models", get(models))
        .route("/calcine/v1/models/pull", post(pull))
        .route("/calcine/v1/models/{*key}", delete(remove))
        .route("/calcine/v1/jobs", get(jobs))
}

fn forbidden() -> Response {
    api_error(
        StatusCode::FORBIDDEN,
        "permission_error",
        "this API key can't manage models",
    )
}

async fn status(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
) -> Response {
    if !caller.can_manage() {
        return forbidden();
    }
    let server = app.services.server.state().borrow().clone();
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "backend": app.services.backend,
        "server": server,
    }))
    .into_response()
}

async fn models(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
) -> Response {
    if !caller.can_manage() {
        return forbidden();
    }
    match app.services.models.list().await {
        Ok(models) => Json(models).into_response(),
        Err(err) => api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "server_error",
            &err.to_string(),
        ),
    }
}

#[derive(Deserialize)]
struct PullBody {
    /// A name or link, as accepted by Calcine's Discover page.
    model: String,
}

async fn pull(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
    Json(body): Json<PullBody>,
) -> Response {
    if !caller.can_manage() {
        return forbidden();
    }
    match ModelReference::parse(&body.model) {
        Ok(reference) => {
            let id = app.services.start_pull(PullRequest {
                reference,
                model_type: None,
            });
            (StatusCode::ACCEPTED, Json(json!({ "jobId": id }))).into_response()
        }
        Err(err) => api_error(
            StatusCode::BAD_REQUEST,
            "invalid_request_error",
            &err.to_string(),
        ),
    }
}

async fn remove(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
    Path(key): Path<String>,
) -> Response {
    if !caller.can_manage() {
        return forbidden();
    }
    let reference = match ModelReference::parse(&key) {
        Ok(reference) => reference,
        Err(err) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                &err.to_string(),
            );
        }
    };
    let key = ModelKey {
        name: reference.name,
        precision: reference.precision,
    };
    match app.services.models.remove(&[key]).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => api_error(
            StatusCode::BAD_REQUEST,
            "invalid_request_error",
            &err.to_string(),
        ),
    }
}

async fn jobs(State(app): State<Arc<AppState>>, Extension(caller): Extension<Caller>) -> Response {
    if !caller.can_manage() {
        return forbidden();
    }
    Json(app.services.jobs.list()).into_response()
}
