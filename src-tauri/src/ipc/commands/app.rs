use calcine_core::{BackendKind, Services};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;
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

/// Whether Calcine starts with Windows (in the tray).
#[tauri::command]
#[specta::specta]
pub fn autostart_enabled(app: AppHandle) -> ApiResult<bool> {
    app.autolaunch().is_enabled().map_err(ApiError::io)
}

#[tauri::command]
#[specta::specta]
pub fn set_autostart(app: AppHandle, enabled: bool) -> ApiResult<()> {
    let launcher = app.autolaunch();
    if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    }
    .map_err(ApiError::io)
}

/// Exports larger than this are refused (a conversation is a few hundred KB).
const MAX_EXPORT_BYTES: usize = 16 << 20;

/// Save a Markdown export where the user picks in the system dialog.
/// Returns the saved path, or `None` when the user cancels.
#[tauri::command]
#[specta::specta]
pub async fn save_markdown(
    app: AppHandle,
    file_name: String,
    contents: String,
) -> ApiResult<Option<String>> {
    if contents.len() > MAX_EXPORT_BYTES {
        return Err(ApiError::invalid_input("this export is too large".into()));
    }
    let (sender, receiver) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_file_name(markdown_file_name(&file_name))
        .add_filter("Markdown", &["md"])
        .save_file(move |path| {
            let _ = sender.send(path);
        });
    let Some(path) = receiver.await.ok().flatten() else {
        return Ok(None);
    };
    let path = path.into_path().map_err(ApiError::io)?;
    tokio::fs::write(&path, contents)
        .await
        .map_err(ApiError::io)?;
    Ok(Some(path.display().to_string()))
}

/// A safe Windows file name ending in `.md`.
fn markdown_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_control() || r#"<>:"/\|?*"#.contains(c) {
                ' '
            } else {
                c
            }
        })
        .collect();
    let words = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let stem: String = words.trim_end_matches('.').chars().take(80).collect();
    let stem = stem.trim_end();
    if stem.is_empty() {
        "Conversation.md".to_owned()
    } else {
        format!("{stem}.md")
    }
}

#[cfg(test)]
mod tests {
    use super::markdown_file_name;

    #[test]
    fn makes_safe_markdown_file_names() {
        assert_eq!(markdown_file_name("What's 2/3?"), "What's 2 3.md");
        assert_eq!(markdown_file_name("  \u{7}  "), "Conversation.md");
        assert_eq!(markdown_file_name("notes..."), "notes.md");
        assert_eq!(markdown_file_name(&"a".repeat(200)).len(), 83);
    }
}
