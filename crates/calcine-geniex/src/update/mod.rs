//! Installing and updating GenieX through its official installer.
//!
//! - `index`: the release index and manifests `geniex update` reads
//! - `download`: fetching an installer with progress and SHA-256 check
//! - `signature`: Authenticode check of the installer
//!
//! The installer is per-user Inno Setup: it uninstalls the previous version
//! (killing any running `geniex.exe`) and installs to
//! `%LOCALAPPDATA%\GenieX CLI`. Installers are kept in a cache to roll back
//! or repair without downloading.

mod download;
mod index;
mod signature;

use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use calcine_core::jobs::{JobCtx, JobPhase, JobProgress};
use calcine_core::runtime::{
    CachedInstaller, InstallSource, ReleaseChannel, RuntimeInstaller, RuntimeManager,
    RuntimeUpdateCheck, compare_versions,
};
use calcine_core::{Error, Result};

use crate::Geniex;

/// Where Qualcomm publishes GenieX releases.
pub const RELEASE_ENDPOINT: &str =
    "https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-geniex";

/// The installer runs silently; a slow disk can still take a while.
const INSTALL_TIMEOUT: Duration = Duration::from_mins(10);

/// Installers kept in the cache, newest first (plus the bundled one).
const KEPT_INSTALLERS: usize = 3;

/// An installer shipped inside Calcine's own installer.
#[derive(Debug, Clone)]
pub struct BundledInstaller {
    pub path: PathBuf,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct InstallerConfig {
    pub endpoint: String,
    /// Downloaded installers, kept for rolling back.
    pub cache_dir: PathBuf,
    pub bundled: Option<BundledInstaller>,
}

/// [`RuntimeInstaller`] for the official GenieX installer.
#[derive(Debug)]
pub struct GeniexInstaller {
    geniex: Geniex,
    config: InstallerConfig,
    http: reqwest::Client,
}

impl GeniexInstaller {
    /// # Errors
    ///
    /// If the TLS backend can't be initialised.
    pub fn new(geniex: Geniex, config: InstallerConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(concat!("Calcine/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(20))
            .build()
            .map_err(|err| Error::Network(format!("couldn't set up HTTPS: {err}")))?;
        Ok(Self {
            geniex,
            config,
            http,
        })
    }

    async fn get_text(&self, file: &str) -> Result<String> {
        let url = format!("{}/{file}", self.config.endpoint);
        let response = self
            .http
            .get(&url)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|err| Error::Network(format!("couldn't reach GenieX releases: {err}")))?;
        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "GenieX releases answered {} for {file}",
                response.status()
            )));
        }
        response
            .text()
            .await
            .map_err(|err| Error::Network(format!("GenieX releases reply was cut short: {err}")))
    }

    async fn current_version(&self) -> Option<String> {
        self.geniex.info().await.ok().map(|info| info.cli_version)
    }

    fn cached_path(&self, version: &str) -> PathBuf {
        self.config
            .cache_dir
            .join(format!("geniex-cli-setup-{version}.exe"))
    }

    /// The installer file for `source`, downloading and verifying it if needed.
    async fn installer_file(&self, source: &InstallSource, ctx: &JobCtx) -> Result<PathBuf> {
        match source {
            InstallSource::Release { release } => {
                let path = self.cached_path(&release.version);
                download::fetch(&self.http, &release.installer, &path, ctx).await?;
                Ok(path)
            }
            InstallSource::Cached { version } => {
                let cached = self.cached_path(version);
                if cached.is_file() {
                    return Ok(cached);
                }
                match &self.config.bundled {
                    Some(bundled) if bundled.version == *version && bundled.path.is_file() => {
                        Ok(bundled.path.clone())
                    }
                    _ => Err(Error::InvalidInput(format!(
                        "the GenieX {version} installer isn't on this PC anymore"
                    ))),
                }
            }
        }
    }

    /// Keep the newest downloaded installers.
    fn prune_cache(&self) {
        let mut installers = self.cached_downloads();
        installers.sort_by(|a, b| compare_versions(&b.0, &a.0));
        for (_, path) in installers.into_iter().skip(KEPT_INSTALLERS) {
            let _ = std::fs::remove_file(path);
        }
    }

    fn cached_downloads(&self) -> Vec<(String, PathBuf)> {
        let Ok(entries) = std::fs::read_dir(&self.config.cache_dir) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name().into_string().ok()?;
                let version = name
                    .strip_prefix("geniex-cli-setup-")?
                    .strip_suffix(".exe")?
                    .to_owned();
                Some((version, entry.path()))
            })
            .collect()
    }
}

#[async_trait]
impl RuntimeInstaller for GeniexInstaller {
    async fn check(&self, channel: ReleaseChannel) -> Result<RuntimeUpdateCheck> {
        let current = self.current_version().await;
        let index_json = self.get_text("index.json").await?;
        let latest = match index::latest_tag(&index_json, channel)? {
            Some(tag) => {
                let manifest = self.get_text(&format!("manifest-{tag}.json")).await?;
                index::release(&manifest, "windows", "arm64")?
            }
            None => None,
        };
        let publisher_signed = match self.get_text("windows-signed.txt").await {
            Ok(text) => index::publisher_signed(&text),
            Err(err) => {
                tracing::warn!(%err, "couldn't read windows-signed.txt");
                false
            }
        };
        let update_available = latest.as_ref().is_some_and(|latest| {
            current.as_deref().is_none_or(|current| {
                compare_versions(&latest.version, current) == std::cmp::Ordering::Greater
            })
        });
        Ok(RuntimeUpdateCheck {
            channel,
            current,
            latest,
            update_available,
            publisher_signed,
        })
    }

    async fn install(&self, source: InstallSource, ctx: JobCtx) -> Result<()> {
        let version = source.version().to_owned();
        let installer = self.installer_file(&source, &ctx).await?;

        ctx.report(phase(JobPhase::Verifying));
        match signature::verify(&installer).await? {
            signature::Signature::Signed { subject } => {
                tracing::info!(%subject, "GenieX installer signature is valid");
            }
            signature::Signature::Unsigned => {
                tracing::info!("GenieX installer isn't code-signed; its SHA-256 was verified");
            }
            signature::Signature::NotChecked => {}
        }

        // Past this point the job can't be cancelled: stopping the installer
        // halfway could leave GenieX half removed.
        ctx.report(phase(JobPhase::Installing));
        run_installer(&installer).await?;

        let reported = self.current_version().await.ok_or_else(|| Error::Command {
            command: "version".into(),
            message: "GenieX isn't found after installing it".into(),
        })?;
        if compare_versions(&reported, &version) != std::cmp::Ordering::Equal {
            return Err(Error::Command {
                command: "version".into(),
                message: format!("the installer finished but GenieX reports {reported}"),
            });
        }
        self.prune_cache();
        Ok(())
    }

    fn cached(&self) -> Vec<CachedInstaller> {
        let mut installers: Vec<CachedInstaller> = self
            .cached_downloads()
            .into_iter()
            .map(|(version, path)| CachedInstaller {
                size_bytes: std::fs::metadata(path).map_or(0, |meta| meta.len()),
                version,
                bundled: false,
            })
            .collect();
        if let Some(bundled) = &self.config.bundled
            && bundled.path.is_file()
            && installers
                .iter()
                .all(|cached| cached.version != bundled.version)
        {
            installers.push(CachedInstaller {
                version: bundled.version.clone(),
                bundled: true,
                size_bytes: std::fs::metadata(&bundled.path).map_or(0, |meta| meta.len()),
            });
        }
        installers.sort_by(|a, b| compare_versions(&b.version, &a.version));
        installers
    }
}

fn phase(phase: JobPhase) -> JobProgress {
    JobProgress {
        phase: Some(phase),
        ..JobProgress::default()
    }
}

/// Run the Inno Setup installer silently and wait for it.
async fn run_installer(installer: &Path) -> Result<()> {
    let mut command = tokio::process::Command::new(installer);
    command
        .args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"])
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    let mut child = command.spawn()?;
    let status = tokio::time::timeout(INSTALL_TIMEOUT, child.wait())
        .await
        .map_err(|_| Error::Timeout {
            command: "GenieX installer".into(),
            seconds: INSTALL_TIMEOUT.as_secs(),
        })??;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Command {
            command: "GenieX installer".into(),
            message: format!("exited with {status}"),
        })
    }
}
