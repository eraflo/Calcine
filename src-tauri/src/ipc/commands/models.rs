use calcine_core::Services;
use calcine_core::jobs::JobId;
use calcine_core::models::{HubCatalog, LocalModel, ModelKey};
use calcine_core::models::{ModelReference, PullRequest};
use tauri::State;

use crate::ipc::error::ApiResult;

/// Models in the local GenieX cache (`geniex list`).
#[tauri::command]
#[specta::specta]
pub async fn list_models(services: State<'_, Services>) -> ApiResult<Vec<LocalModel>> {
    Ok(services.models.list().await?)
}

/// Delete whole models or single precisions (`geniex remove`).
#[tauri::command]
#[specta::specta]
pub async fn remove_models(services: State<'_, Services>, keys: Vec<ModelKey>) -> ApiResult<()> {
    Ok(services.models.remove(&keys).await?)
}

/// Understand a pasted model name or link, to preview it before downloading.
#[tauri::command]
#[specta::specta]
pub fn parse_model_reference(input: String) -> ApiResult<ModelReference> {
    Ok(ModelReference::parse(&input)?)
}

/// Start a download in the background. Progress arrives as `JobUpdated` events.
#[tauri::command]
#[specta::specta]
pub async fn pull_model(services: State<'_, Services>, request: PullRequest) -> ApiResult<JobId> {
    Ok(services.start_pull(request))
}

/// Qualcomm AI Hub models with an NPU build (`geniex model list [--all]`).
#[tauri::command]
#[specta::specta]
pub async fn aihub_catalog(
    services: State<'_, Services>,
    all_chipsets: bool,
) -> ApiResult<HubCatalog> {
    Ok(services.catalog.aihub(all_chipsets).await?)
}
