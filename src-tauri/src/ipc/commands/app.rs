use calcine_core::{BackendKind, Services};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

use crate::desktop::locale::{Language, Locale};
use crate::desktop::tray;

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

/// The UI language, for the tray menu and notifications.
#[tauri::command]
#[specta::specta]
pub fn set_language(app: AppHandle, locale: State<'_, Locale>, language: Language) {
    locale.set(language);
    tray::set_language(&app, language);
}
