//! Calcine desktop shell.
//!
//! Wires the shared [`Services`](calcine_core::Services) and the HTTP gateway
//! into Tauri. Domain logic lives in `calcine-core`, the backend crates and
//! the gateway, so it can be tested without Tauri.
//!
//! - `ipc`: commands and events the webview uses (typed bindings)
//! - `setup`: building the app state at startup, tearing it down on exit
//! - `desktop`: tray, window behaviour, child process lifetime
//! - [`cli`]: `calcine-cli`, the API without the window

pub mod cli;
mod desktop;
mod ipc;
mod setup;
#[cfg(windows)]
mod user_path;

use tauri::RunEvent;
use tracing_subscriber::EnvFilter;

/// Start the desktop app.
///
/// # Panics
///
/// If Tauri fails to start (for example, no WebView2 runtime).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing("info");
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
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![desktop::MINIMIZED_ARG]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(desktop::locale::Locale::default())
        .invoke_handler(specta.invoke_handler())
        .setup(move |app| {
            specta.mount_events(app);
            let services = setup::start(app)?;
            desktop::notifications::notify_finished_jobs(app.handle(), &services.jobs);
            desktop::tray::install(app)?;
            // Started with Windows: stay in the tray until opened.
            if !std::env::args().any(|arg| arg == desktop::MINIMIZED_ARG) {
                desktop::show_main_window(app.handle());
            }
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

/// Entry point of `calcine-cli`: run the command in the arguments and
/// return the exit code. Logs only warnings, to stderr.
pub fn cli_main() -> i32 {
    init_tracing("warn");
    desktop::process::kill_children_on_exit();
    let args: Vec<String> = std::env::args().skip(1).collect();
    cli::main(&args)
}

/// Regenerate `src/lib/bindings.ts` (used by `tests/bindings.rs`).
#[doc(hidden)]
pub fn export_bindings() -> Result<(), specta_typescript::Error> {
    ipc::export(&ipc::builder())
}

fn init_tracing(default_level: &str) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
