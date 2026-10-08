//! Pick the backend (real GenieX or the mock) and assemble the services.

use std::path::PathBuf;
use std::sync::Arc;

use calcine_core::jobs::JobManager;
use calcine_core::runtime::{InstallerAsset, RuntimeRelease, ServerOptions};
use calcine_core::{BackendKind, Services};
use calcine_geniex::update::RELEASE_ENDPOINT;
use calcine_geniex::{
    BenchConfig, BundledInstaller, Geniex, GeniexBench, GeniexConfig, GeniexInstaller,
    GeniexServer, InstallerConfig,
};
use calcine_hub::{HubConfig, HubDirectory};
use calcine_hw::SystemProbe;
use calcine_mock::MockBackend;
use serde::Deserialize;

/// Environment variable selecting the backend: `mock` or `geniex` (default).
const BACKEND_ENV: &str = "CALCINE_BACKEND";

/// The GenieX version Calcine was tested with and installs on first launch
/// (`runtime/geniex.json`), with its official download and checksum.
const PINNED_GENIEX: &str = include_str!("../../../runtime/geniex.json");

/// Where Calcine keeps files the services need.
#[derive(Debug, Clone)]
pub struct AppPaths {
    /// Downloaded GenieX installers, for rolling back.
    pub runtime_cache: PathBuf,
    /// The GenieX installer shipped in Calcine's installer, if any.
    pub bundled_geniex: Option<PathBuf>,
    /// `geniex-bench` versions, downloaded on demand.
    pub bench_tools: PathBuf,
    /// Benchmark history.
    pub bench_history: PathBuf,
}

pub fn services_from_env(paths: &AppPaths) -> Services {
    match std::env::var(BACKEND_ENV).as_deref() {
        Ok("mock") => MockBackend::services(),
        Ok(other) if other != "geniex" => {
            tracing::warn!(
                value = other,
                "unknown {BACKEND_ENV}, falling back to geniex"
            );
            geniex_services(paths)
        }
        _ => geniex_services(paths),
    }
}

fn geniex_services(paths: &AppPaths) -> Services {
    let geniex = Geniex::new(GeniexConfig::default());
    let server = Arc::new(GeniexServer::new(geniex.clone(), ServerOptions::default()));
    let installer = GeniexInstaller::new(
        geniex.clone(),
        InstallerConfig {
            endpoint: RELEASE_ENDPOINT.to_owned(),
            cache_dir: paths.runtime_cache.clone(),
            bundled: paths.bundled_geniex.clone().map(|path| BundledInstaller {
                path,
                version: pinned_release().version,
            }),
        },
    )
    .expect("the system TLS stack should load");
    let bench = GeniexBench::new(
        geniex.clone(),
        BenchConfig {
            tools_dir: paths.bench_tools.clone(),
            history_path: paths.bench_history.clone(),
        },
    )
    .expect("the system TLS stack should load");
    let geniex = Arc::new(geniex);
    Services {
        backend: BackendKind::Geniex,
        models: geniex.clone(),
        catalog: geniex.clone(),
        directory: Arc::new(
            HubDirectory::new(HubConfig::default()).expect("the system TLS stack should load"),
        ),
        runtime: geniex,
        server,
        hardware: Arc::new(SystemProbe::new()),
        installer: Arc::new(installer),
        bench: Arc::new(bench),
        jobs: JobManager::new(),
    }
}

/// The pinned GenieX release. Its SHA-256 ships inside Calcine, so the
/// download is checked against a value from the signed app itself.
pub fn pinned_release() -> RuntimeRelease {
    #[derive(Deserialize)]
    struct Pin {
        version: String,
        assets: Assets,
    }
    #[derive(Deserialize)]
    struct Assets {
        #[serde(rename = "windows-arm64")]
        windows_arm64: Asset,
    }
    #[derive(Deserialize)]
    struct Asset {
        name: String,
        url: String,
        size: u64,
        sha256: String,
    }
    let pin: Pin = serde_json::from_str(PINNED_GENIEX).expect("runtime/geniex.json is valid");
    let asset = pin.assets.windows_arm64;
    RuntimeRelease {
        notes_url: Some(format!(
            "https://github.com/qualcomm/GenieX/releases/tag/{}",
            pin.version
        )),
        version: pin.version,
        prerelease: false,
        released_at: None,
        installer: InstallerAsset {
            name: asset.name,
            url: asset.url,
            size: asset.size,
            sha256: asset.sha256.to_ascii_lowercase(),
        },
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_the_pinned_release() {
        let release = super::pinned_release();
        assert!(release.version.starts_with('v'));
        assert!(release.installer.url.starts_with("https://"));
        assert_eq!(release.installer.sha256.len(), 64);
        assert!(release.installer.name.contains(&release.version));
    }
}
