use std::sync::Arc;

use calcine_core::Services;
use calcine_core::runtime::ServerOptions;
use calcine_gateway::{
    ApiKeyInfo, CreatedApiKey, Gateway, GatewaySettings, GatewayStatus, KeyError, NewApiKey,
    RequestEntry,
};
use serde::Serialize;
use specta::Type;
use tauri::State;

use crate::ipc::error::{ApiError, ApiResult};
use crate::setup::ServerOptionsFile;

/// How Calcine's own UI talks to the gateway.
#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GatewayConnection {
    pub base_url: String,
    /// Per-launch token with full rights. Only given to the app's webview.
    pub token: String,
}

#[tauri::command]
#[specta::specta]
pub fn gateway_status(gateway: State<'_, Arc<Gateway>>) -> GatewayStatus {
    gateway.status()
}

#[tauri::command]
#[specta::specta]
pub fn gateway_connection(gateway: State<'_, Arc<Gateway>>) -> GatewayConnection {
    GatewayConnection {
        base_url: gateway.status().base_url,
        token: gateway.keys().internal_token().to_owned(),
    }
}

/// Start GenieX now instead of on the first request.
#[tauri::command]
#[specta::specta]
pub async fn start_server(services: State<'_, Services>) -> ApiResult<()> {
    services.server.ensure_running().await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn stop_server(services: State<'_, Services>) -> ApiResult<()> {
    Ok(services.server.stop().await?)
}

/// Recent output of `geniex serve`.
#[tauri::command]
#[specta::specta]
pub fn server_logs(services: State<'_, Services>) -> Vec<String> {
    services.server.logs()
}

/// What `geniex serve` starts with: model unload delay, context window.
#[tauri::command]
#[specta::specta]
pub fn server_options(services: State<'_, Services>) -> ServerOptions {
    services.server.options()
}

/// Save new server options. A running GenieX restarts to apply them.
#[tauri::command]
#[specta::specta]
pub async fn set_server_options(
    services: State<'_, Services>,
    file: State<'_, ServerOptionsFile>,
    options: ServerOptions,
) -> ApiResult<()> {
    options.validate().map_err(ApiError::invalid_input)?;
    file.save(options).map_err(ApiError::io)?;
    Ok(services.server.set_options(options).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn set_require_api_key(gateway: State<'_, Arc<Gateway>>, require: bool) -> ApiResult<()> {
    gateway
        .set_require_api_key(require)
        .await
        .map_err(ApiError::io)
}

/// Recent API requests, newest first.
#[tauri::command]
#[specta::specta]
pub fn list_requests(gateway: State<'_, Arc<Gateway>>) -> Vec<RequestEntry> {
    gateway.requests()
}

#[tauri::command]
#[specta::specta]
pub fn list_api_keys(gateway: State<'_, Arc<Gateway>>) -> Vec<ApiKeyInfo> {
    gateway.keys().list()
}

/// Create a key. The token is returned once and never stored in clear.
#[tauri::command]
#[specta::specta]
pub fn create_api_key(
    gateway: State<'_, Arc<Gateway>>,
    request: NewApiKey,
) -> ApiResult<CreatedApiKey> {
    gateway.keys().create(request).map_err(key_error)
}

#[tauri::command]
#[specta::specta]
pub fn revoke_api_key(gateway: State<'_, Arc<Gateway>>, id: String) -> ApiResult<bool> {
    gateway.keys().revoke(&id).map_err(key_error)
}

fn key_error(err: KeyError) -> ApiError {
    match err {
        KeyError::MissingName | KeyError::NoScope => ApiError::invalid_input(err.to_string()),
        KeyError::Io(_) | KeyError::Corrupted(_) => ApiError::io(err),
    }
}

/// The port and allowed browser origins of the local API.
#[tauri::command]
#[specta::specta]
pub fn gateway_settings(gateway: State<'_, Arc<Gateway>>) -> GatewaySettings {
    gateway.settings()
}

/// Move the local API to another port. Apps must use the new address.
#[tauri::command]
#[specta::specta]
pub async fn set_gateway_port(gateway: State<'_, Arc<Gateway>>, port: u16) -> ApiResult<()> {
    gateway
        .set_port(port)
        .await
        .map_err(ApiError::invalid_input)
}

/// Browser origins allowed to call the local API (local web apps).
#[tauri::command]
#[specta::specta]
pub fn set_allowed_origins(
    gateway: State<'_, Arc<Gateway>>,
    origins: Vec<String>,
) -> ApiResult<()> {
    gateway
        .set_allowed_origins(origins)
        .map_err(ApiError::invalid_input)
}
