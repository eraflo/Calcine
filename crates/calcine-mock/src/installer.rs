//! Simulated GenieX updates: a stable and a pre-release newer than the
//! "installed" v0.8.0, installed with fake download progress.

use async_trait::async_trait;
use calcine_core::jobs::{JobCtx, JobPhase, JobProgress};
use calcine_core::runtime::{
    CachedInstaller, InstallSource, InstallerAsset, ReleaseChannel, RuntimeInstaller,
    RuntimeRelease, RuntimeUpdateCheck, compare_versions,
};
use calcine_core::{Error, Result};

use crate::MockBackend;

const INSTALLER_BYTES: u64 = 65_196_910;
const STEPS: u64 = 20;

fn release(version: &str, prerelease: bool) -> RuntimeRelease {
    RuntimeRelease {
        version: version.into(),
        prerelease,
        released_at: Some("2026-10-06T19:36:37Z".into()),
        installer: InstallerAsset {
            name: format!("geniex-cli-setup-windows-arm64-{version}.exe"),
            url: format!("mock://geniex/{version}"),
            size: INSTALLER_BYTES,
            sha256: "0".repeat(64),
        },
        notes_url: Some(format!(
            "https://github.com/qualcomm/GenieX/releases/tag/{version}"
        )),
    }
}

#[async_trait]
impl RuntimeInstaller for MockBackend {
    async fn check(&self, channel: ReleaseChannel) -> Result<RuntimeUpdateCheck> {
        let current = self.cli_version();
        let latest = match channel {
            ReleaseChannel::Stable => release("v0.8.1", false),
            ReleaseChannel::Prerelease => release("v0.9.0-rc.1", true),
        };
        Ok(RuntimeUpdateCheck {
            channel,
            update_available: compare_versions(&latest.version, &current)
                == std::cmp::Ordering::Greater,
            current: Some(current),
            latest: Some(latest),
            publisher_signed: true,
        })
    }

    async fn install(&self, source: InstallSource, ctx: JobCtx) -> Result<()> {
        if let InstallSource::Release { .. } = source {
            let tick = self.pull_duration / u32::try_from(STEPS).unwrap_or(1);
            for step in 0..=STEPS {
                ctx.report(JobProgress {
                    done_bytes: INSTALLER_BYTES * step / STEPS,
                    total_bytes: Some(INSTALLER_BYTES),
                    bytes_per_second: Some(INSTALLER_BYTES / 4),
                    phase: Some(JobPhase::Downloading),
                    step: None,
                });
                tokio::select! {
                    () = tokio::time::sleep(tick) => {}
                    () = ctx.cancelled() => return Err(Error::Cancelled),
                }
            }
        }
        for phase in [JobPhase::Verifying, JobPhase::Installing] {
            ctx.report(JobProgress {
                phase: Some(phase),
                ..JobProgress::default()
            });
            tokio::time::sleep(self.pull_duration / 6).await;
        }
        source.version().clone_into(&mut self.version());
        Ok(())
    }

    fn cached(&self) -> Vec<CachedInstaller> {
        vec![CachedInstaller {
            version: "v0.8.0".into(),
            bundled: true,
            size_bytes: INSTALLER_BYTES,
        }]
    }
}
