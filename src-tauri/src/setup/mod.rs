//! Build the app state at startup and tear it down on exit.

mod gateway;
mod server_options;
mod services;

use std::sync::Arc;

use calcine_core::Services;
use calcine_gateway::Gateway;
use tauri::path::BaseDirectory;
use tauri::{App, AppHandle, Manager};

pub use self::server_options::ServerOptionsFile;
use self::services::{AppPaths, services_from_env};

/// Build the services, start the gateway, and forward state changes to the
/// webview. Returns the services, also managed as Tauri state.
pub fn start(app: &App) -> tauri::Result<Services> {
    let data_dir = app.path().app_data_dir()?;
    let local_dir = app.path().app_local_data_dir()?;
    let paths = AppPaths {
        runtime_cache: local_dir.join("runtime-cache"),
        bench_tools: local_dir.join("geniex-bench"),
        bench_history: data_dir.join("benchmarks.json"),
        // Only release builds bundle it (`tauri.release.conf.json`).
        bundled_geniex: app
            .path()
            .resolve("geniex/geniex-cli-setup.exe", BaseDirectory::Resource)
            .ok()
            .filter(|path| path.is_file()),
    };
    let services = services_from_env(&paths);
    tracing::info!(backend = ?services.backend, "starting Calcine");
    app.manage(services.clone());

    // The server isn't running yet: this only sets what it starts with.
    let options_file = ServerOptionsFile::new(&data_dir);
    let options = options_file.load();
    if let Err(err) = tauri::async_runtime::block_on(services.server.set_options(options)) {
        tracing::warn!(%err, "ignoring the saved server options");
    }
    app.manage(options_file);
    let gateway = Arc::new(gateway::create(services.clone(), &data_dir));
    app.manage(gateway.clone());
    crate::ipc::events::forward(app.handle(), &services.jobs, &gateway);
    tauri::async_runtime::spawn(async move {
        if let Err(err) = gateway.start().await {
            tracing::error!(%err, "gateway didn't start");
        }
    });
    Ok(services)
}

/// Don't leave `geniex serve` (and a loaded model) running after quitting.
pub fn shutdown(handle: &AppHandle) {
    let services = handle
        .try_state::<Services>()
        .map(|state| state.inner().clone());
    let gateway = handle
        .try_state::<Arc<Gateway>>()
        .map(|state| state.inner().clone());
    tauri::async_runtime::block_on(async move {
        if let Some(gateway) = gateway {
            gateway.stop().await;
        }
        if let Some(services) = services {
            let _ = services.server.stop().await;
        }
    });
}
