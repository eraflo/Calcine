use calcine_core::{BackendKind, Services};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub backend: BackendKind,
}

/// Calcine's own version and the active backend.
#[tauri::command]
#[specta::specta]
pub fn app_info(app: AppHandle, services: State<'_, Services>) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
        backend: services.backend,
    }
}
