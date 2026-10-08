//! Qualcomm AI Hub `platform.json`: the chipsets models are compiled for.

use calcine_core::models::Chipset;
use calcine_core::{Error, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Platform {
    #[serde(default)]
    chipsets: Vec<ApiChipset>,
    #[serde(default)]
    devices: Vec<Device>,
}

#[derive(Debug, Deserialize)]
struct ApiChipset {
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    marketing_name: Option<String>,
    #[serde(default)]
    reference_device: String,
}

#[derive(Debug, Deserialize)]
struct Device {
    chipset: String,
    os: Os,
}

#[derive(Debug, Deserialize)]
struct Os {
    ostype: String,
}

/// `os.ostype` values this build runs on, as GenieX filters them.
pub fn host_os_types() -> &'static [&'static str] {
    if cfg!(windows) {
        &["OPERATING_SYSTEM_TYPE_WINDOWS"]
    } else if cfg!(target_os = "linux") {
        &[
            "OPERATING_SYSTEM_TYPE_LINUX",
            "OPERATING_SYSTEM_TYPE_QC_LINUX",
        ]
    } else {
        &[]
    }
}

/// Chipsets with a reference device on one of `os_types` (all of them when
/// `os_types` is empty), sorted by device name like GenieX's picker.
pub fn parse_chipsets(body: &str, os_types: &[&str]) -> Result<Vec<Chipset>> {
    let platform: Platform = serde_json::from_str(body)
        .map_err(|err| Error::Parse(format!("unexpected AI Hub platform list: {err}")))?;
    let runs_here = |chipset: &ApiChipset| {
        let mut devices = platform
            .devices
            .iter()
            .filter(|device| device.chipset == chipset.name)
            .peekable();
        // GenieX keeps chipsets without any device entry.
        devices.peek().is_none()
            || os_types.is_empty()
            || devices.any(|device| os_types.contains(&device.os.ostype.as_str()))
    };
    let mut chipsets: Vec<Chipset> = platform
        .chipsets
        .iter()
        .filter(|chipset| runs_here(chipset))
        .map(|chipset| Chipset {
            id: chipset.name.clone(),
            device: if chipset.reference_device.is_empty() {
                chipset.name.clone()
            } else {
                chipset.reference_device.clone()
            },
            marketing_name: chipset
                .marketing_name
                .as_deref()
                .map(|name| name.replace(['®', '™'], "")),
            aliases: chipset
                .aliases
                .iter()
                .filter(|alias| **alias != chipset.name)
                .cloned()
                .collect(),
        })
        .collect();
    chipsets.sort_by_key(|chipset| chipset.device.to_ascii_lowercase());
    Ok(chipsets)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLATFORM: &str = include_str!("../tests/fixtures/aihub_platform.json");

    #[test]
    fn windows_lists_the_snapdragon_x_family() {
        let chipsets = parse_chipsets(PLATFORM, &["OPERATING_SYSTEM_TYPE_WINDOWS"]).unwrap();
        let ids: Vec<&str> = chipsets.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "qualcomm-snapdragon-x-elite",
                "qualcomm-snapdragon-x-plus-8-core",
                "qualcomm-snapdragon-x2-elite"
            ]
        );
        let elite = &chipsets[0];
        assert_eq!(elite.device, "Snapdragon X Elite CRD");
        assert_eq!(elite.aliases, ["sc8380xp"]);
        assert!(
            elite
                .marketing_name
                .as_deref()
                .is_some_and(|n| !n.contains('®'))
        );
    }

    #[test]
    fn no_filter_keeps_every_chipset() {
        let chipsets = parse_chipsets(PLATFORM, &[]).unwrap();
        assert_eq!(chipsets.len(), 25);
    }
}
