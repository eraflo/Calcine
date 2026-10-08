//! Finding `geniex-bench` in GenieX's GitHub release. Qualcomm attaches it
//! to every release with a `.sha256` file; GitHub also reports each asset's
//! SHA-256 (`digest`). Both must agree.

use calcine_core::runtime::InstallerAsset;
use calcine_core::{Error, Result};
use serde_json::Value;

/// Where GenieX releases are published on GitHub.
pub const RELEASES_API: &str = "https://api.github.com/repos/qualcomm/GenieX";

/// The archive for this platform and GenieX version.
pub fn archive_name(version: &str) -> String {
    format!("geniex-bench-windows-arm64-{version}.zip")
}

/// The archive and the URL of its `.sha256` file, from a release's JSON.
pub fn find(release: &Value, version: &str) -> Result<(InstallerAsset, Option<String>)> {
    let name = archive_name(version);
    let assets = release
        .get("assets")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Parse("the GenieX release lists no files".into()))?;
    let asset = |wanted: &str| {
        assets
            .iter()
            .find(|asset| asset.get("name").and_then(Value::as_str) == Some(wanted))
    };
    let archive = asset(&name).ok_or_else(|| {
        Error::InvalidInput(format!(
            "GenieX {version} has no benchmark tool for this PC ({name})"
        ))
    })?;
    let sha256 = archive
        .get("digest")
        .and_then(Value::as_str)
        .and_then(|digest| digest.strip_prefix("sha256:"))
        .filter(|hex| hex.len() == 64)
        .ok_or_else(|| Error::Parse(format!("GitHub gives no checksum for {name}")))?
        .to_ascii_lowercase();
    let url = archive
        .get("browser_download_url")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Parse(format!("GitHub gives no link for {name}")))?
        .to_owned();
    let size = archive.get("size").and_then(Value::as_u64).unwrap_or(0);
    let checksum_url = asset(&format!("{name}.sha256"))
        .and_then(|file| file.get("browser_download_url"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    Ok((
        InstallerAsset {
            name,
            url,
            size,
            sha256,
        },
        checksum_url,
    ))
}

/// The hash in a `.sha256` file (`<hex>  <file name>`).
pub fn checksum_file_hash(text: &str) -> Option<String> {
    text.split_whitespace()
        .next()
        .filter(|hex| hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const DIGEST: &str = "f7d3df090edff588e5da8d9cf97ee7aef8f8ad2ae5fe7911aa52340f719e054a";

    #[test]
    fn finds_the_archive_and_its_checksums() {
        let release = json!({ "assets": [
            { "name": "geniex-cli-setup-windows-arm64-v0.8.0.exe", "digest": "sha256:00", "size": 1, "browser_download_url": "x" },
            { "name": "geniex-bench-windows-arm64-v0.8.0.zip", "digest": format!("sha256:{DIGEST}"), "size": 84_772_012, "browser_download_url": "https://github.com/a.zip" },
            { "name": "geniex-bench-windows-arm64-v0.8.0.zip.sha256", "browser_download_url": "https://github.com/a.zip.sha256" },
        ]});
        let (asset, checksum) = find(&release, "v0.8.0").unwrap();
        assert_eq!(asset.sha256, DIGEST);
        assert_eq!(asset.size, 84_772_012);
        assert_eq!(asset.url, "https://github.com/a.zip");
        assert_eq!(checksum.as_deref(), Some("https://github.com/a.zip.sha256"));
    }

    #[test]
    fn needs_a_digest() {
        let release = json!({ "assets": [
            { "name": "geniex-bench-windows-arm64-v0.8.0.zip", "size": 1, "browser_download_url": "x" },
        ]});
        assert!(find(&release, "v0.8.0").is_err());
        assert!(find(&json!({ "assets": [] }), "v0.8.0").is_err());
    }

    #[test]
    fn reads_sha256_files() {
        let text = format!("{DIGEST}  geniex-bench-windows-arm64-v0.8.0.zip\n");
        assert_eq!(checksum_file_hash(&text).as_deref(), Some(DIGEST));
        assert_eq!(checksum_file_hash("not a hash"), None);
    }
}
