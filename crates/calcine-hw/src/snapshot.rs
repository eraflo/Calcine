//! A one-off description of the hardware.

use std::path::Path;

use calcine_core::hardware::{Accelerator, HardwareInfo};

use crate::system;

pub(crate) fn snapshot(models_dir: Option<&Path>) -> HardwareInfo {
    let platform = platform_devices();
    let system = system::read(platform.cpu_name.as_deref());
    HardwareInfo {
        cpu: system.cpu,
        memory: system.memory,
        accelerators: platform.accelerators,
        models_disk: models_dir.and_then(system::disk_space),
    }
}

/// Devices only the OS driver database knows about.
#[derive(Debug, Default)]
pub(crate) struct PlatformDevices {
    pub accelerators: Vec<Accelerator>,
    pub cpu_name: Option<String>,
}

#[cfg(windows)]
fn platform_devices() -> PlatformDevices {
    crate::windows::devices::devices().unwrap_or_else(|err| {
        tracing::warn!(%err, "couldn't query WMI for NPUs, GPUs and the CPU name");
        PlatformDevices::default()
    })
}

#[cfg(target_os = "linux")]
fn platform_devices() -> PlatformDevices {
    crate::linux::devices()
}

#[cfg(not(any(windows, target_os = "linux")))]
fn platform_devices() -> PlatformDevices {
    PlatformDevices::default()
}
