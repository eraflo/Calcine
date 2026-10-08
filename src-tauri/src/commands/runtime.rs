use calcine_core::Services;
use calcine_core::runtime::RuntimeInfo;
use tauri::State;

use crate::error::ApiResult;

/// The installed GenieX runtime (`geniex version`).
#[tauri::command]
#[specta::specta]
pub async fn runtime_info(services: State<'_, Services>) -> ApiResult<RuntimeInfo> {
    Ok(services.runtime.info().await?)
}
