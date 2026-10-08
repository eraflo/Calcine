//! Build the app state at startup and tear it down on exit.

mod gateway;
mod services;

use std::sync::Arc;

use calcine_core::Services;
use calcine_gateway::Gateway;
use tauri::path::BaseDirectory;
use tauri::{App, AppHandle, Manager};

use self::services::{AppPaths, services_from_env};

/// Build the services, start the gateway, and forward state changes to the
/// webview. Returns the services, also managed as Tauri state.
pub fn start(app: &App) -> tauri::Result<Services> {
    let paths = AppPaths {
        runtime_cache: app.path().app_local_data_dir()?.join("runtime-cache"),
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

    let data_dir = app.path().app_data_dir()?;
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
