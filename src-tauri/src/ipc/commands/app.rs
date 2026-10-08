use calcine_core::{BackendKind, Services};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

use tauri_plugin_opener::OpenerExt;

use crate::desktop::locale::{Language, Locale};
use crate::desktop::tray;
use crate::ipc::error::{ApiError, ApiResult};

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

/// Sites Calcine links to (release notes, model pages).
const LINK_HOSTS: &[&str] = &["github.com", "huggingface.co", "aihub.qualcomm.com"];

/// Open a release notes or model page in the default browser. Only HTTPS
/// links to the sites Calcine itself links to are opened.
#[tauri::command]
#[specta::specta]
pub fn open_url(app: AppHandle, url: String) -> ApiResult<()> {
    let host = url
        .strip_prefix("https://")
        .and_then(|rest| rest.split(['/', '?', '#']).next())
        .unwrap_or_default();
    if !LINK_HOSTS.contains(&host) {
        return Err(ApiError::invalid_input(format!(
            "Calcine doesn't open {url}"
        )));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(ApiError::io)
}
