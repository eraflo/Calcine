use calcine_core::Services;
use calcine_core::hardware::{HardwareInfo, HardwareUsage};
use calcine_core::models::Chipset;
use calcine_core::runtime::RuntimeInfo;
use tauri::State;

use crate::ipc::error::ApiResult;

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

/// Pin the chipset models are downloaded for, or `None` to detect it again
/// (`geniex config set chipset`).
#[tauri::command]
#[specta::specta]
pub async fn set_chipset(services: State<'_, Services>, chipset: Option<String>) -> ApiResult<()> {
    Ok(services.runtime.set_chipset(chipset.as_deref()).await?)
}

/// Chipsets Qualcomm AI Hub builds models for.
#[tauri::command]
#[specta::specta]
pub async fn list_chipsets(services: State<'_, Services>) -> ApiResult<Vec<Chipset>> {
    Ok(services.directory.chipsets().await?)
}

/// CPU, memory, NPU/GPU and free space for the model cache.
#[tauri::command]
#[specta::specta]
pub async fn hardware_info(services: State<'_, Services>) -> ApiResult<HardwareInfo> {
    let models_dir = services.runtime.models_dir();
    Ok(services.hardware.snapshot(models_dir.as_deref()).await?)
}

/// Live CPU, GPU, NPU and memory load. Poll about once a second.
#[tauri::command]
#[specta::specta]
pub async fn hardware_usage(services: State<'_, Services>) -> ApiResult<HardwareUsage> {
    Ok(services.hardware.usage().await?)
}
