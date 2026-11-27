//! The quick question window and its shortcut.

use tauri::{AppHandle, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::desktop::quick_ask::{self, QuickAsk, QuickAskStatus};
use crate::ipc::error::{ApiError, ApiResult};

/// Copied text longer than this is cut: it goes into a model's prompt.
const MAX_COPIED_CHARS: usize = 20_000;

/// The shortcut that opens the quick question window, and whether it works.
#[tauri::command]
#[specta::specta]
pub fn quick_ask_status(quick: State<'_, QuickAsk>) -> QuickAskStatus {
    quick.status()
}

/// Open the quick question window with `shortcut` (`None`: never).
#[tauri::command]
#[specta::specta]
pub fn set_quick_ask_shortcut(app: AppHandle, shortcut: Option<String>) -> ApiResult<()> {
    quick_ask::set_shortcut(&app, shortcut).map_err(ApiError::invalid_input)
}

/// Close the quick question window; with `open_calcine`, bring Calcine up
/// (where the question continues).
#[tauri::command]
#[specta::specta]
pub fn hide_quick_ask(app: AppHandle, open_calcine: bool) {
    quick_ask::hide(&app);
    if open_calcine {
        crate::desktop::show_main_window(&app);
    }
}

/// The text on the clipboard, to ask about it. Read only when the quick
/// question window opens; it stays on this PC.
#[tauri::command]
#[specta::specta]
pub fn copied_text(app: AppHandle) -> Option<String> {
    let text = app.clipboard().read_text().ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.chars().take(MAX_COPIED_CHARS).collect())
}
