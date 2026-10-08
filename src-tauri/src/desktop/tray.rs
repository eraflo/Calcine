//! System tray: Calcine keeps running (and serving apps) when its window is closed.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, Wry};

use super::locale::Language;
use super::show_main_window;

const OPEN: &str = "open";
const QUIT: &str = "quit";

/// Menu entries kept to relabel them when the language changes.
struct TrayItems {
    open: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

pub fn install(app: &App) -> tauri::Result<()> {
    let strings = Language::default().strings();
    let open = MenuItem::with_id(app, OPEN, strings.open, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT, strings.quit, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &PredefinedMenuItem::separator(app)?, &quit])?;
    app.manage(TrayItems { open, quit });

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Calcine")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            OPEN => show_main_window(app),
            QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// Relabel the menu in `language`.
pub fn set_language(app: &AppHandle, language: Language) {
    let strings = language.strings();
    if let Some(items) = app.try_state::<TrayItems>() {
        let _ = items.open.set_text(strings.open);
        let _ = items.quit.set_text(strings.quit);
    }
}
