//! Hardware probes.
//!
//! - `system`: CPU, memory and disk, portable (`sysinfo`)
//! - `windows`: Hexagon NPU, Adreno GPU and the CPU name, from the Windows
//!   driver database (WMI). Other platforms report no accelerators for now.

mod system;
#[cfg(windows)]
mod windows;

use std::path::Path;

use async_trait::async_trait;
use calcine_core::hardware::{Accelerator, HardwareInfo, HardwareProbe};
use calcine_core::{Error, Result};

/// Probe for the machine Calcine runs on.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemProbe;

#[async_trait]
impl HardwareProbe for SystemProbe {
    async fn snapshot(&self, models_dir: Option<&Path>) -> Result<HardwareInfo> {
        let models_dir = models_dir.map(Path::to_path_buf);
        // WMI and sysinfo are blocking (WMI also needs COM on its own thread).
        tokio::task::spawn_blocking(move || snapshot(models_dir.as_deref()))
            .await
            .map_err(|err| Error::Io(std::io::Error::other(err)))
    }
}

fn snapshot(models_dir: Option<&Path>) -> HardwareInfo {
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
struct PlatformDevices {
    accelerators: Vec<Accelerator>,
    cpu_name: Option<String>,
}

#[cfg(windows)]
fn platform_devices() -> PlatformDevices {
    windows::devices().unwrap_or_else(|err| {
        tracing::warn!(%err, "couldn't query WMI for NPUs, GPUs and the CPU name");
        PlatformDevices::default()
    })
}

#[cfg(not(windows))]
fn platform_devices() -> PlatformDevices {
    PlatformDevices::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reports_cpu_memory_and_the_current_drive() {
        let dir = std::env::current_dir().unwrap();
        let info = SystemProbe.snapshot(Some(&dir)).await.unwrap();
        assert!(info.memory.total_bytes > 0);
        let cpu = info.cpu.expect("every machine has a CPU");
        assert_ne!(cpu.name, "");
        let disk = info
            .models_disk
            .expect("the current directory is on some drive");
        assert!(disk.total_bytes >= disk.available_bytes);
    }
}
