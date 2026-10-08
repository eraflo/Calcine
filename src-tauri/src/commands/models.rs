use calcine_core::Services;
use calcine_core::model::LocalModel;
use tauri::State;

use crate::error::ApiResult;

/// Models in the local GenieX cache (`geniex list`).
#[tauri::command]
#[specta::specta]
pub async fn list_models(services: State<'_, Services>) -> ApiResult<Vec<LocalModel>> {
    Ok(services.models.list().await?)
}
