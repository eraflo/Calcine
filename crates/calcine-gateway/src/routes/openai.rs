//! The OpenAI-compatible API under `/v1`.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use calcine_core::models::LocalModel;
use serde_json::{Value, json};

use crate::caller::Caller;
use crate::error::api_error;
use crate::proxy;
use crate::state::AppState;
use crate::structured::{self, tools};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1", get(health))
        .route("/v1/", get(health))
        .route("/v1/models", get(list_models))
        .route("/v1/models/{*id}", get(get_model))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/completions", post(completions))
        .route("/v1/logits", post(logits))
        .route("/v1/systemone", post(systemone))
}

async fn health() -> &'static str {
    "Calcine is running"
}

async fn chat_completions(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
    body: Bytes,
) -> Response {
    let Ok(mut json) = serde_json::from_slice::<Value>(&body) else {
        return proxy::inference(app, caller, "/v1/chat/completions", body).await;
    };
    // GenieX ignores `tool_choice` and `response_format`: Calcine enforces them.
    let refused = tools::refused(&json);
    if refused {
        json = tools::without_tools(&json);
    }
    match structured::goal(&json) {
        Ok(Some(goal)) => structured::complete(app, caller, json, goal).await,
        Ok(None) if refused => {
            let body = Bytes::from(json.to_string());
            proxy::inference(app, caller, "/v1/chat/completions", body).await
        }
        Ok(None) => proxy::inference(app, caller, "/v1/chat/completions", body).await,
        Err(message) => api_error(StatusCode::BAD_REQUEST, "invalid_request_error", &message),
    }
}

async fn completions(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
    body: Bytes,
) -> Response {
    proxy::inference(app, caller, "/v1/completions", body).await
}

async fn logits(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
    body: Bytes,
) -> Response {
    proxy::inference(app, caller, "/v1/logits", body).await
}

async fn systemone(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
    body: Bytes,
) -> Response {
    proxy::inference(app, caller, "/v1/systemone", body).await
}

/// Answered from the model cache, without starting GenieX. Ids match
/// `geniex serve`: `name:precision`.
async fn list_models(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
) -> Response {
    if !caller.can_infer() {
        return api_error(
            StatusCode::FORBIDDEN,
            "permission_error",
            "this API key can't list models",
        );
    }
    match app.services.models.list().await {
        Ok(models) => {
            let data: Vec<Value> = models.iter().flat_map(model_entries).collect();
            Json(json!({ "object": "list", "data": data })).into_response()
        }
        Err(err) => api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "server_error",
            &err.to_string(),
        ),
    }
}

async fn get_model(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
) -> Response {
    if !caller.can_infer() {
        return api_error(
            StatusCode::FORBIDDEN,
            "permission_error",
            "this API key can't list models",
        );
    }
    let models = match app.services.models.list().await {
        Ok(models) => models,
        Err(err) => {
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "server_error",
                &err.to_string(),
            );
        }
    };
    models
        .iter()
        .flat_map(model_entries)
        .find(|entry| {
            entry["id"] == id
                || entry["id"]
                    .as_str()
                    .is_some_and(|full| full.split(':').next() == Some(&id))
        })
        .map_or_else(
            || {
                api_error(
                    StatusCode::NOT_FOUND,
                    "invalid_request_error",
                    &format!("model {id} isn't downloaded"),
                )
            },
            |entry| Json(entry).into_response(),
        )
}

fn model_entries(model: &LocalModel) -> Vec<Value> {
    let owner = model.name.split('/').next().unwrap_or("local");
    let ids: Vec<String> = if model.precisions.is_empty() {
        vec![model.name.clone()]
    } else {
        model
            .precisions
            .iter()
            .map(|precision| format!("{}:{precision}", model.name))
            .collect()
    };
    ids.into_iter()
        .map(|id| json!({ "id": id, "object": "model", "created": 0, "owned_by": owner }))
        .collect()
}
