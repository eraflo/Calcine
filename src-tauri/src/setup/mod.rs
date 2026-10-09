//! Build the app state at startup and tear it down on exit.

mod gateway;
mod server_options;
mod services;

use std::path::PathBuf;
use std::sync::Arc;

use calcine_core::Services;
use calcine_gateway::Gateway;
use tauri::path::BaseDirectory;
use tauri::{App, AppHandle, Manager};

pub use self::server_options::ServerOptionsFile;
pub use self::services::pinned_release;
use self::services::{AppPaths, services_from_env};

/// The app's identifier, which names its data folders (as in
/// `tauri.conf.json`).
pub const IDENTIFIER: &str = "com.eraflo.calcine";

/// Where Calcine keeps its files.
#[derive(Debug, Clone)]
pub struct DataDirs {
    /// Settings, keys, history (roaming on Windows).
    pub data: PathBuf,
    /// Caches: GenieX installers, the benchmark tool.
    pub local: PathBuf,
}

impl DataDirs {
    /// The folders Tauri gives the window, found without Tauri (for
    /// `calcine-cli`): `%APPDATA%` and `%LOCALAPPDATA%` on Windows,
    /// `$XDG_DATA_HOME` (or `~/.local/share`) elsewhere.
    pub fn from_env() -> Option<Self> {
        let var = |name: &str| std::env::var_os(name).map(PathBuf::from);
        let (data, local) = if cfg!(windows) {
            (var("APPDATA")?, var("LOCALAPPDATA")?)
        } else {
            let share = var("XDG_DATA_HOME")
                .or_else(|| var("HOME").map(|home| home.join(".local").join("share")))?;
            (share.clone(), share)
        };
        Some(Self {
            data: data.join(IDENTIFIER),
            local: local.join(IDENTIFIER),
        })
    }
}

/// What both the window and `calcine-cli serve` run on.
pub struct Core {
    pub services: Services,
    pub gateway: Arc<Gateway>,
    pub options_file: ServerOptionsFile,
}

/// Build the services and the gateway (not started yet) the same way for
/// the window and for `calcine-cli`.
pub async fn build(dirs: &DataDirs, bundled_geniex: Option<PathBuf>) -> Core {
    let paths = AppPaths {
        runtime_cache: dirs.local.join("runtime-cache"),
        bench_tools: dirs.local.join("geniex-bench"),
        bench_history: dirs.data.join("benchmarks.json"),
        bundled_geniex,
    };
    let services = services_from_env(&paths);
    tracing::info!(backend = ?services.backend, "starting Calcine");

    // The server isn't running yet: this only sets what it starts with.
    let options_file = ServerOptionsFile::new(&dirs.data);
    if let Err(err) = services.server.set_options(options_file.load()).await {
        tracing::warn!(%err, "ignoring the saved server options");
    }
    let gateway = Arc::new(gateway::create(services.clone(), &dirs.data));
    Core {
        services,
        gateway,
        options_file,
    }
}

/// Build the services, start the gateway, and forward state changes to the
/// webview. Returns the services, also managed as Tauri state.
pub fn start(app: &App) -> tauri::Result<Services> {
    let dirs = DataDirs {
        data: app.path().app_data_dir()?,
        local: app.path().app_local_data_dir()?,
    };
    // Only release builds bundle it (`tauri.release.conf.json`).
    let bundled = app
        .path()
        .resolve("geniex/geniex-cli-setup.exe", BaseDirectory::Resource)
        .ok()
        .filter(|path| path.is_file());
    let Core {
        services,
        gateway,
        options_file,
    } = tauri::async_runtime::block_on(build(&dirs, bundled));
    app.manage(services.clone());
    app.manage(options_file);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_identifier_matches_the_tauri_config() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../../tauri.conf.json")).unwrap();
        assert_eq!(config["identifier"], IDENTIFIER);
    }

    #[test]
    #[cfg(windows)]
    fn finds_the_same_folders_as_tauri() {
        let dirs = DataDirs::from_env().unwrap();
        assert!(dirs.data.ends_with(IDENTIFIER));
        assert!(dirs.data.starts_with(std::env::var_os("APPDATA").unwrap()));
        assert!(
            dirs.local
                .starts_with(std::env::var_os("LOCALAPPDATA").unwrap())
        );
    }
}
