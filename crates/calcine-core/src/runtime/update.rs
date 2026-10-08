//! Installing, updating and repairing the GenieX runtime.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Result;
use crate::jobs::JobCtx;

/// Which GenieX releases to follow.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseChannel {
    /// Stable releases only (what `geniex update` installs).
    #[default]
    Stable,
    /// Release candidates and alphas too.
    Prerelease,
}

/// A GenieX version that can be installed on this machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeRelease {
    /// Tag, e.g. `v0.8.1-rc.1`.
    pub version: String,
    pub prerelease: bool,
    /// ISO 8601 date.
    pub released_at: Option<String>,
    pub installer: InstallerAsset,
    /// Release notes page.
    pub notes_url: Option<String>,
}

/// The installer for this OS and architecture, as listed in the official
/// release manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct InstallerAsset {
    pub name: String,
    pub url: String,
    pub size: u64,
    /// Lowercase hex SHA-256 the download must match.
    pub sha256: String,
}

/// Whether a newer GenieX is available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeUpdateCheck {
    pub channel: ReleaseChannel,
    /// Installed version (`None` when GenieX is missing).
    pub current: Option<String>,
    /// Newest release on the channel.
    pub latest: Option<RuntimeRelease>,
    /// `latest` is newer than `current`, or GenieX is missing.
    pub update_available: bool,
    /// Qualcomm marks its Windows installers as code-signed
    /// (`windows-signed.txt`). `geniex update` refuses to install otherwise.
    pub publisher_signed: bool,
}

/// An installer kept on disk, to reinstall without downloading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CachedInstaller {
    pub version: String,
    /// Shipped inside Calcine's installer (used to repair GenieX offline).
    pub bundled: bool,
    pub size_bytes: u64,
}

/// What to install.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum InstallSource {
    /// Download a release, verifying its SHA-256.
    Release { release: RuntimeRelease },
    /// Reinstall an installer already on disk (roll back, repair).
    Cached { version: String },
}

impl InstallSource {
    pub fn version(&self) -> &str {
        match self {
            Self::Release { release } => &release.version,
            Self::Cached { version } => version,
        }
    }
}

/// Installs GenieX through its official installer.
#[async_trait]
pub trait RuntimeInstaller: Send + Sync {
    /// The newest release on `channel`, compared with what's installed.
    async fn check(&self, channel: ReleaseChannel) -> Result<RuntimeUpdateCheck>;

    /// Download (or take from the cache), verify and run the installer,
    /// then check the installed version. The caller stops `geniex serve`
    /// first: the installer kills every running `geniex.exe`.
    async fn install(&self, source: InstallSource, ctx: JobCtx) -> Result<()>;

    /// Installers on disk, newest first.
    fn cached(&self) -> Vec<CachedInstaller>;
}

/// Compare GenieX tags (`v0.8.0`, `v0.8.1-rc.1`). Unparsable tags sort first.
pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    parse_version(a).cmp(&parse_version(b))
}

/// `(major, minor, patch, is_release, pre-release parts)`: a release sorts
/// after its pre-releases, like semver.
fn parse_version(tag: &str) -> Option<(u64, u64, u64, bool, Vec<PreRelease>)> {
    let tag = tag.trim().trim_start_matches('v');
    let (core, pre) = match tag.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (tag, None),
    };
    let mut numbers = core.split('.').map(str::parse::<u64>);
    let (Some(Ok(major)), Some(Ok(minor)), Some(Ok(patch))) =
        (numbers.next(), numbers.next(), numbers.next())
    else {
        return None;
    };
    let pre_parts = pre
        .map(|pre| {
            pre.split('.')
                .map(|part| {
                    part.parse::<u64>()
                        .map_or_else(|_| PreRelease::Text(part.to_owned()), PreRelease::Number)
                })
                .collect()
        })
        .unwrap_or_default();
    Some((major, minor, patch, pre.is_none(), pre_parts))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum PreRelease {
    /// Numbers sort before text, as in semver.
    Number(u64),
    Text(String),
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn orders_releases_like_semver() {
        let mut tags = vec![
            "v0.8.0",
            "v0.8.1-rc.1",
            "v0.7.1",
            "v0.8.1-alpha.2",
            "v0.8.1",
            "v0.10.0",
        ];
        tags.sort_by(|a, b| compare_versions(a, b));
        assert_eq!(
            tags,
            [
                "v0.7.1",
                "v0.8.0",
                "v0.8.1-alpha.2",
                "v0.8.1-rc.1",
                "v0.8.1",
                "v0.10.0"
            ]
        );
    }

    #[test]
    fn unparsable_tags_sort_first() {
        assert_eq!(compare_versions("garbage", "v0.0.1"), Ordering::Less);
        assert_eq!(compare_versions("v0.8.0", "0.8.0"), Ordering::Equal);
    }
}
