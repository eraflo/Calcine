//! Linux: the Hexagon NPU and the Adreno GPU, from `/dev` and `/sys`.
//!
//! - NPU: the compute DSP the FastRPC driver exposes (`/dev/fastrpc-cdsp`),
//!   or a remote processor named `cdsp`.
//! - GPU: a DRM card driven by `msm` (Adreno, upstream), or the KGSL driver
//!   of Qualcomm's own kernels.
//! - SoC: the device tree's `compatible`, e.g. `qcom,x1e80100`.

use std::path::Path;

use calcine_core::hardware::Accelerator;
use calcine_core::models::ComputeUnit;

use crate::snapshot::PlatformDevices;

pub(crate) fn devices() -> PlatformDevices {
    devices_in(Path::new("/"))
}

fn devices_in(root: &Path) -> PlatformDevices {
    let mut accelerators = Vec::new();
    if has_npu(root) {
        accelerators.push(Accelerator {
            unit: ComputeUnit::Npu,
            name: "Qualcomm Hexagon NPU".into(),
            driver_version: None,
        });
    }
    if let Some(name) = gpu(root) {
        accelerators.push(Accelerator {
            unit: ComputeUnit::Gpu,
            name,
            driver_version: None,
        });
    }
    PlatformDevices {
        accelerators,
        cpu_name: soc(root),
    }
}

fn read(root: &Path, path: &str) -> Option<String> {
    std::fs::read_to_string(root.join(path)).ok()
}

/// Entries of a sysfs class folder, like `/sys/class/drm/*`.
fn entries(root: &Path, class: &str) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(root.join(class))
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default()
}

fn has_npu(root: &Path) -> bool {
    root.join("dev/fastrpc-cdsp").exists()
        || root.join("dev/fastrpc-cdsp-secure").exists()
        || entries(root, "sys/class/remoteproc")
            .iter()
            .any(|processor| {
                std::fs::read_to_string(processor.join("name"))
                    .is_ok_and(|name| name.to_ascii_lowercase().contains("cdsp"))
            })
}

fn gpu(root: &Path) -> Option<String> {
    if root.join("sys/class/kgsl/kgsl-3d0").exists() {
        let model = read(root, "sys/class/kgsl/kgsl-3d0/gpu_model")
            .map(|model| model.trim().to_owned())
            .filter(|model| !model.is_empty());
        return Some(model.map_or_else(
            || "Qualcomm Adreno GPU".into(),
            |model| format!("Qualcomm {model}"),
        ));
    }
    entries(root, "sys/class/drm")
        .iter()
        .any(|card| {
            std::fs::read_to_string(card.join("device/uevent"))
                .is_ok_and(|uevent| uevent.lines().any(|line| line == "DRIVER=msm"))
        })
        .then(|| "Qualcomm Adreno GPU".into())
}

/// `qcom,x1e80100` → `Qualcomm X1E80100`. Strings are NUL-separated.
fn soc(root: &Path) -> Option<String> {
    let compatible = std::fs::read(root.join("sys/firmware/devicetree/base/compatible")).ok()?;
    String::from_utf8_lossy(&compatible)
        .split('\0')
        .find_map(|entry| entry.strip_prefix("qcom,"))
        .filter(|chip| !chip.is_empty())
        .map(|chip| format!("Qualcomm {}", chip.to_ascii_uppercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(files: &[(&str, &[u8])]) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("calcine-hw-{}-{}", std::process::id(), files.len()));
        let _ = std::fs::remove_dir_all(&root);
        for (path, contents) in files {
            let path = root.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, contents).unwrap();
        }
        root
    }

    #[test]
    fn finds_a_snapdragon_x_laptop() {
        let root = tree(&[
            ("dev/fastrpc-cdsp", b""),
            (
                "sys/class/drm/card0/device/uevent",
                b"DRIVER=msm\nOF_NAME=gpu\n",
            ),
            (
                "sys/firmware/devicetree/base/compatible",
                b"lenovo,thinkpad-t14s\0qcom,x1e78100\0qcom,x1e80100\0",
            ),
        ]);
        let devices = devices_in(&root);
        let units: Vec<_> = devices.accelerators.iter().map(|a| a.unit).collect();
        assert_eq!(units, [ComputeUnit::Npu, ComputeUnit::Gpu]);
        assert_eq!(devices.cpu_name.as_deref(), Some("Qualcomm X1E78100"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn finds_qualcomm_kernels_and_nothing_elsewhere() {
        let root = tree(&[
            ("sys/class/remoteproc/remoteproc1/name", b"cdsp\n"),
            ("sys/class/kgsl/kgsl-3d0/gpu_model", b"Adreno643v1\n"),
        ]);
        let devices = devices_in(&root);
        assert_eq!(devices.accelerators.len(), 2);
        assert_eq!(devices.accelerators[1].name, "Qualcomm Adreno643v1");
        assert_eq!(devices.cpu_name, None);
        let _ = std::fs::remove_dir_all(&root);

        let empty = tree(&[("sys/class/drm/card0/device/uevent", b"DRIVER=virtio_gpu\n")]);
        assert_eq!(devices_in(&empty).accelerators, Vec::new());
        let _ = std::fs::remove_dir_all(empty);
    }
}
