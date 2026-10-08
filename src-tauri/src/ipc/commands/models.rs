use calcine_core::Services;
use calcine_core::jobs::JobId;
use calcine_core::models::{
    HubCatalog, LocalModel, ModelKey, ModelReference, ModelType, PullRequest, RemoteModel,
    RemoteModelDetails,
};
use serde::Deserialize;
use specta::Type;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::ipc::error::{ApiError, ApiResult};

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
    Ok(services.remove_models(&keys).await?)
}

/// Delete every cached model (`geniex clean`).
#[tauri::command]
#[specta::specta]
pub async fn clean_models(services: State<'_, Services>) -> ApiResult<()> {
    Ok(services.clean_models().await?)
}

/// Correct whether a model takes images and audio (`geniex model set-type`).
#[tauri::command]
#[specta::specta]
pub async fn set_model_type(
    services: State<'_, Services>,
    name: String,
    model_type: ModelType,
) -> ApiResult<()> {
    Ok(services.models.set_type(&name, model_type).await?)
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
    if request.is_import() {
        return Err(ApiError::invalid_input(
            "use import_model to add a model from this PC".into(),
        ));
    }
    Ok(services.start_pull(request))
}

/// Copy a model folder or AI Hub `.zip` from this PC into the cache.
#[tauri::command]
#[specta::specta]
pub async fn import_model(
    services: State<'_, Services>,
    name: String,
    path: String,
    model_type: Option<ModelType>,
) -> ApiResult<JobId> {
    Ok(services.start_pull(PullRequest::import(&name, path, model_type)?))
}

/// What can be imported.
#[derive(Debug, Clone, Copy, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ImportSource {
    /// GGUF files, or an extracted AI Hub bundle.
    Folder,
    /// An AI Hub `.zip`.
    Archive,
}

/// Ask for a folder or `.zip` to import with the system file picker. `None`
/// when the user cancels.
#[tauri::command]
#[specta::specta]
pub async fn pick_import_source(app: AppHandle, source: ImportSource) -> Option<String> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let dialog = app.dialog().file().set_title("Import a model");
    let reply = move |path: Option<tauri_plugin_dialog::FilePath>| {
        let _ = sender.send(path.map(|path| path.to_string()));
    };
    match source {
        ImportSource::Folder => dialog.pick_folder(reply),
        ImportSource::Archive => dialog
            .add_filter("AI Hub model archive", &["zip"])
            .pick_file(reply),
    }
    receiver.await.ok().flatten()
}

/// Show the model cache in the file manager.
#[tauri::command]
#[specta::specta]
pub fn open_models_folder(app: AppHandle, services: State<'_, Services>) -> ApiResult<()> {
    let dir = services
        .runtime
        .models_dir()
        .filter(|dir| dir.is_dir())
        .ok_or_else(|| ApiError::invalid_input("the model folder doesn't exist yet".into()))?;
    app.opener()
        .open_path(dir.display().to_string(), None::<&str>)
        .map_err(ApiError::io)
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

/// Search Hugging Face for GGUF models.
#[tauri::command]
#[specta::specta]
pub async fn search_models(
    services: State<'_, Services>,
    query: String,
    limit: u32,
) -> ApiResult<Vec<RemoteModel>> {
    Ok(services.directory.search(&query, limit).await?)
}

/// Precisions of a remote model with their download sizes.
#[tauri::command]
#[specta::specta]
pub async fn model_details(
    services: State<'_, Services>,
    reference: ModelReference,
) -> ApiResult<RemoteModelDetails> {
    Ok(services.directory.details(&reference).await?)
}
