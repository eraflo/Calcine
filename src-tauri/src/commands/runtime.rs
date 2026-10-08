use calcine_core::Services;
use calcine_core::hardware::HardwareInfo;
use calcine_core::runtime::RuntimeInfo;
use tauri::State;

use crate::error::ApiResult;

/// The installed GenieX runtime (`geniex version`).
#[tauri::command]
#[specta::specta]
pub async fn runtime_info(services: State<'_, Services>) -> ApiResult<RuntimeInfo> {
    Ok(services.runtime.info().await?)
}

/// The Snapdragon chipset GenieX targets (`geniex config get chipset`).
#[tauri::command]
#[specta::specta]
pub async fn chipset(services: State<'_, Services>) -> ApiResult<Option<String>> {
    Ok(services.runtime.chipset().await?)
}

/// CPU, memory, NPU/GPU and free space for the model cache.
#[tauri::command]
#[specta::specta]
pub async fn hardware_info(services: State<'_, Services>) -> ApiResult<HardwareInfo> {
    let models_dir = services.runtime.models_dir();
    Ok(services.hardware.snapshot(models_dir.as_deref()).await?)
}
