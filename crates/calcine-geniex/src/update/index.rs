//! The official GenieX release index on Qualcomm's bucket: `index.json`
//! lists the tags, `manifest-<tag>.json` lists each release's files with
//! their size and SHA-256. `geniex update` reads the same files.

use calcine_core::runtime::{InstallerAsset, ReleaseChannel, RuntimeRelease, compare_versions};
use calcine_core::{Error, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Index {
    latest_stable: Option<String>,
    latest_prerelease: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    tag: String,
    #[serde(default)]
    is_prerelease: bool,
    released_at: Option<String>,
    #[serde(default)]
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    url: String,
    size: u64,
    sha256: String,
    kind: String,
    platform: String,
    arch: String,
}

/// The newest tag on `channel`. Pre-releases include stable releases: an
/// `rc` older than the latest stable isn't offered.
pub fn latest_tag(index_json: &str, channel: ReleaseChannel) -> Result<Option<String>> {
    let index: Index = serde_json::from_str(index_json)
        .map_err(|err| Error::Parse(format!("unexpected GenieX release index: {err}")))?;
    Ok(match channel {
        ReleaseChannel::Stable => index.latest_stable,
        ReleaseChannel::Prerelease => [index.latest_stable, index.latest_prerelease]
            .into_iter()
            .flatten()
            .max_by(|a, b| compare_versions(a, b)),
    })
}

/// The installer for `platform`/`arch` (`windows`/`arm64`) in a release
/// manifest, or `None` when that release doesn't ship one.
pub fn release(manifest_json: &str, platform: &str, arch: &str) -> Result<Option<RuntimeRelease>> {
    let manifest: Manifest = serde_json::from_str(manifest_json)
        .map_err(|err| Error::Parse(format!("unexpected GenieX release manifest: {err}")))?;
    let Some(asset) = manifest.assets.into_iter().find(|asset| {
        asset.kind == "cli-installer" && asset.platform == platform && asset.arch == arch
    }) else {
        return Ok(None);
    };
    Ok(Some(RuntimeRelease {
        notes_url: Some(format!(
            "https://github.com/qualcomm/GenieX/releases/tag/{}",
            manifest.tag
        )),
        version: manifest.tag,
        prerelease: manifest.is_prerelease,
        released_at: manifest.released_at,
        installer: InstallerAsset {
            name: asset.name,
            url: asset.url,
            size: asset.size,
            sha256: asset.sha256.to_ascii_lowercase(),
        },
    }))
}

/// `windows-signed.txt`: `true` once Qualcomm signs its Windows installers.
pub fn publisher_signed(text: &str) -> bool {
    text.trim().eq_ignore_ascii_case("true")
}

#[cfg(test)]
mod tests {
    use super::*;

    const INDEX: &str = include_str!("../../tests/fixtures/update_index.json");
    const MANIFEST: &str = include_str!("../../tests/fixtures/update_manifest.json");

    #[test]
    fn picks_the_latest_tag_per_channel() {
        assert_eq!(
            latest_tag(INDEX, ReleaseChannel::Stable)
                .unwrap()
                .as_deref(),
            Some("v0.8.0")
        );
        assert_eq!(
            latest_tag(INDEX, ReleaseChannel::Prerelease)
                .unwrap()
                .as_deref(),
            Some("v0.8.1-rc.1")
        );
    }

    #[test]
    fn prereleases_older_than_stable_are_skipped() {
        let index = r#"{"latest_stable":"v0.9.0","latest_prerelease":"v0.9.0-rc.2"}"#;
        assert_eq!(
            latest_tag(index, ReleaseChannel::Prerelease)
                .unwrap()
                .as_deref(),
            Some("v0.9.0")
        );
    }

    #[test]
    fn finds_the_windows_arm64_installer() {
        let release = release(MANIFEST, "windows", "arm64").unwrap().unwrap();
        assert_eq!(release.version, "v0.8.0");
        assert!(!release.prerelease);
        assert_eq!(
            release.installer.name,
            "geniex-cli-setup-windows-arm64-v0.8.0.exe"
        );
        assert_eq!(release.installer.size, 65_196_910);
        assert_eq!(
            release.installer.sha256,
            "fcc4d932f371a9b3ad3d08a1b34912fe21a7fdf7c54d47c869250b50d81ec734"
        );
        assert!(release.notes_url.unwrap().ends_with("/v0.8.0"));
    }

    #[test]
    fn no_installer_for_other_platforms() {
        assert_eq!(release(MANIFEST, "windows", "x64").unwrap(), None);
    }

    #[test]
    fn reads_the_signed_flag() {
        assert!(publisher_signed("true\n"));
        assert!(!publisher_signed("false"));
    }
}
