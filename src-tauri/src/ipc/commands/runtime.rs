use calcine_core::Services;
use calcine_core::hardware::{HardwareInfo, HardwareUsage};
use calcine_core::jobs::JobId;
use calcine_core::models::Chipset;
use calcine_core::runtime::{
    CachedInstaller, InstallSource, ReleaseChannel, RuntimeInfo, RuntimeUpdateCheck,
};
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

/// The newest GenieX on `channel`, compared with the installed one.
#[tauri::command]
#[specta::specta]
pub async fn check_runtime_update(
    services: State<'_, Services>,
    channel: ReleaseChannel,
) -> ApiResult<RuntimeUpdateCheck> {
    Ok(services.installer.check(channel).await?)
}

/// Install a GenieX release or a cached installer (update, roll back,
/// repair). Progress arrives as `JobUpdated` events. Async so the job is
/// spawned on the Tokio runtime.
#[tauri::command]
#[specta::specta]
pub async fn install_runtime(
    services: State<'_, Services>,
    source: InstallSource,
) -> ApiResult<JobId> {
    Ok(services.start_runtime_install(source))
}

/// GenieX installers kept on this PC, newest first.
#[tauri::command]
#[specta::specta]
pub fn cached_runtimes(services: State<'_, Services>) -> Vec<CachedInstaller> {
    services.installer.cached()
}
