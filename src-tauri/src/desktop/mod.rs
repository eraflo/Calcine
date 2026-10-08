//! Desktop integration: tray, window behaviour, notifications and child
//! process lifetime.

pub mod locale;
pub mod notifications;
pub mod process;
pub mod tray;

use tauri::webview::{PermissionKind, PermissionResponse};
use tauri::{AppHandle, Manager, Webview, Window, WindowEvent};

/// Bring the main window back (from the tray or a second launch).
pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Closing the window keeps Calcine in the tray; quit from the tray menu.
pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window.hide();
    }
}

/// Calcine's own UI may record from the microphone (voice messages to vision
/// models). Nothing else is granted: the webview only ever loads Calcine.
// Tauri passes the webview by value.
#[allow(clippy::needless_pass_by_value)]
pub fn on_permission_request(webview: Webview, kind: PermissionKind) -> PermissionResponse {
    match kind {
        PermissionKind::Microphone if webview.label() == "main" => PermissionResponse::Allow,
        _ => PermissionResponse::Deny,
    }
}
