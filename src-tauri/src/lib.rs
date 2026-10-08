//! Calcine desktop shell.
//!
//! Wires the shared [`Services`](calcine_core::Services) and the HTTP gateway
//! into Tauri. Domain logic lives in `calcine-core`, the backend crates and
//! the gateway, so it can be tested without Tauri.
//!
//! - `ipc`: commands and events the webview uses (typed bindings)
//! - `setup`: building the app state at startup, tearing it down on exit
//! - `desktop`: tray, window behaviour, child process lifetime

mod desktop;
mod ipc;
mod setup;

use tauri::RunEvent;
use tracing_subscriber::EnvFilter;

/// Start the desktop app.
///
/// # Panics
///
/// If Tauri fails to start (for example, no WebView2 runtime).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();
    desktop::process::kill_children_on_exit();

    let specta = ipc::builder();
    #[cfg(debug_assertions)]
    if let Err(err) = ipc::export(&specta) {
        tracing::warn!(%err, "couldn't export TypeScript bindings");
    }

    let app = tauri::Builder::default()
        // Must come first: a second launch focuses the running instance.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            desktop::show_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .manage(desktop::locale::Locale::default())
        .invoke_handler(specta.invoke_handler())
        .setup(move |app| {
            specta.mount_events(app);
            let services = setup::start(app)?;
            desktop::notifications::notify_finished_jobs(app.handle(), &services.jobs);
            desktop::tray::install(app)?;
            Ok(())
        })
        .on_window_event(desktop::on_window_event)
        .on_permission_request(desktop::on_permission_request)
        .build(tauri::generate_context!())
        .expect("error while building Calcine");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            setup::shutdown(handle);
        }
    });
}

/// Regenerate `src/lib/bindings.ts` (used by `tests/bindings.rs`).
#[doc(hidden)]
pub fn export_bindings() -> Result<(), specta_typescript::Error> {
    ipc::export(&ipc::builder())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
