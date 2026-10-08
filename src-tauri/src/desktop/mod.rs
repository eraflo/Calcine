//! Desktop integration: tray, window behaviour and child process lifetime.

pub mod process;
pub mod tray;

use tauri::{AppHandle, Manager, Window, WindowEvent};

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
