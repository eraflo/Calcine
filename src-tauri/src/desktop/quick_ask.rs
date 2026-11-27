//! Quick questions: a global shortcut opens a small window over whatever is
//! in front, to ask a model something without opening Calcine. It works
//! while Calcine sits in the tray.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{App, AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// The window's label; the webview shows the quick question UI under it.
pub const LABEL: &str = "quick";
/// Sent to the window each time it shows, to start afresh.
const SHOWN_EVENT: &str = "quick-ask:shown";
pub const DEFAULT_SHORTCUT: &str = "Ctrl+Shift+Space";
const SETTINGS_FILE: &str = "quick-ask.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct QuickAskSettings {
    /// The keys that open the window (`None`: off), e.g. `Ctrl+Shift+Space`.
    pub shortcut: Option<String>,
}

impl Default for QuickAskSettings {
    fn default() -> Self {
        Self {
            shortcut: Some(DEFAULT_SHORTCUT.into()),
        }
    }
}

/// The shortcut chosen, and why it doesn't work, if it doesn't (taken by
/// another app).
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct QuickAskStatus {
    pub shortcut: Option<String>,
    pub error: Option<String>,
}

/// Managed state: the settings, where they're saved, and the shortcut's state.
pub struct QuickAsk {
    path: PathBuf,
    status: Mutex<QuickAskStatus>,
}

impl QuickAsk {
    pub fn status(&self) -> QuickAskStatus {
        self.status
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// The plugin, opening or closing the window on its shortcut.
pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                toggle(app);
            }
        })
        .build()
}

/// Register the saved shortcut. A taken one is reported, not fatal.
pub fn install(app: &App, data_dir: &Path) {
    let path = data_dir.join(SETTINGS_FILE);
    let settings: QuickAskSettings = std::fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    let error = settings
        .shortcut
        .as_deref()
        .and_then(|shortcut| register(app.handle(), shortcut).err());
    if let Some(err) = &error {
        tracing::warn!(%err, "couldn't register the quick question shortcut");
    }
    app.manage(QuickAsk {
        path,
        status: Mutex::new(QuickAskStatus {
            shortcut: settings.shortcut,
            error,
        }),
    });
}

/// Use `shortcut` (`None`: none). A shortcut another app holds is refused
/// and the previous one kept.
pub fn set_shortcut(app: &AppHandle, shortcut: Option<String>) -> Result<(), String> {
    let quick = app.state::<QuickAsk>();
    let shortcut = shortcut
        .map(|keys| keys.trim().to_owned())
        .filter(|keys| !keys.is_empty());
    if let Some(keys) = &shortcut {
        parse(keys)?;
    }
    let previous = quick.status().shortcut;
    let _ = app.global_shortcut().unregister_all();
    if let Some(keys) = &shortcut
        && let Err(err) = register(app, keys)
    {
        if let Some(previous) = previous.as_deref() {
            let _ = register(app, previous);
        }
        return Err(err);
    }
    let settings = QuickAskSettings {
        shortcut: shortcut.clone(),
    };
    if let Some(dir) = quick.path.parent() {
        std::fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    }
    let json = serde_json::to_vec_pretty(&settings).map_err(|err| err.to_string())?;
    std::fs::write(&quick.path, json).map_err(|err| err.to_string())?;
    *quick.status.lock().unwrap_or_else(PoisonError::into_inner) = QuickAskStatus {
        shortcut,
        error: None,
    };
    Ok(())
}

/// Show the window (made the first time), or hide it if it's in front.
pub fn toggle(app: &AppHandle) {
    match app.get_webview_window(LABEL) {
        Some(window)
            if window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false) =>
        {
            let _ = window.hide();
        }
        Some(window) => {
            let _ = window.center();
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.emit_to(LABEL, SHOWN_EVENT, ());
        }
        None => {
            let built = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
                .title("Calcine")
                .inner_size(680.0, 460.0)
                .resizable(false)
                .decorations(false)
                .shadow(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .center()
                .focused(true)
                .build();
            if let Err(err) = built {
                tracing::error!(%err, "couldn't open the quick question window");
            }
        }
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.hide();
    }
}

fn parse(keys: &str) -> Result<Shortcut, String> {
    keys.parse::<Shortcut>()
        .map_err(|_| format!("{keys} isn't a shortcut Calcine understands"))
}

fn register(app: &AppHandle, keys: &str) -> Result<(), String> {
    app.global_shortcut()
        .register(parse(keys)?)
        .map_err(|_| format!("{keys} is already used by another app"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn understands_the_usual_shortcuts() {
        for keys in [DEFAULT_SHORTCUT, "Alt+Space", "Ctrl+Alt+K", "Super+Shift+Q"] {
            assert!(parse(keys).is_ok(), "{keys}");
        }
        assert!(parse("Ctrl+Nope").is_err());
    }
}
