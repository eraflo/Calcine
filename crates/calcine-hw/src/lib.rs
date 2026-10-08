//! Hardware probes.
//!
//! CPU, memory and disk come from `sysinfo`; NPUs and GPUs come from the
//! Windows driver database (WMI `Win32_PnPSignedDriver`), where the Hexagon NPU
//! is a `ComputeAccelerator` device and the Adreno GPU a `Display` device.

use std::path::Path;

use async_trait::async_trait;
use calcine_core::hardware::{Accelerator, DiskSpace, HardwareInfo, MemoryInfo, Processor};
use calcine_core::traits::HardwareProbe;
use calcine_core::{Error, Result};
use sysinfo::{CpuRefreshKind, Disks, System};

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
    let mut system = System::new();
    system.refresh_memory();
    system.refresh_cpu_list(CpuRefreshKind::nothing());

    let platform = platform_devices();
    let cpu = system.cpus().first().map(|cpu| {
        // sysinfo reads the brand string, which is empty on Windows ARM64.
        let brand = cpu.brand().trim();
        let name = if brand.is_empty() {
            platform.cpu_name.clone()
        } else {
            Some(brand.to_owned())
        };
        Processor {
            name: name.unwrap_or_else(|| "Unknown processor".to_owned()),
            cores: u32::try_from(System::physical_core_count().unwrap_or(system.cpus().len()))
                .unwrap_or(u32::MAX),
        }
    });

    HardwareInfo {
        cpu,
        memory: MemoryInfo {
            total_bytes: system.total_memory(),
            available_bytes: system.available_memory(),
        },
        accelerators: platform.accelerators,
        models_disk: models_dir.and_then(disk_space),
    }
}

/// Devices only the OS driver database knows about.
#[derive(Debug, Default)]
struct PlatformDevices {
    accelerators: Vec<Accelerator>,
    cpu_name: Option<String>,
}

/// Free space on the drive that holds `dir` (the longest matching mount point).
fn disk_space(dir: &Path) -> Option<DiskSpace> {
    let target = normalize(dir);
    let disks = Disks::new_with_refreshed_list();
    disks
        .list()
        .iter()
        .filter(|disk| target.starts_with(&normalize(disk.mount_point())))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .map(|disk| DiskSpace {
            path: dir.display().to_string(),
            total_bytes: disk.total_space(),
            available_bytes: disk.available_space(),
        })
}

/// Compare paths case-insensitively on Windows (`c:\` vs `C:\`).
fn normalize(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) {
        text.to_lowercase()
    } else {
        text
    }
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

#[cfg(windows)]
mod windows {
    use calcine_core::hardware::Accelerator;
    use calcine_core::model::ComputeUnit;
    use serde::Deserialize;

    use super::PlatformDevices;

    #[derive(Debug, Deserialize)]
    #[serde(rename = "Win32_PnPSignedDriver", rename_all = "PascalCase")]
    struct SignedDriver {
        device_name: Option<String>,
        device_class: Option<String>,
        driver_version: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename = "Win32_Processor", rename_all = "PascalCase")]
    struct Processor {
        name: Option<String>,
    }

    pub(super) fn devices() -> wmi::WMIResult<PlatformDevices> {
        let connection = wmi::WMIConnection::new()?;
        let drivers: Vec<SignedDriver> = connection.raw_query(
            "SELECT DeviceName, DeviceClass, DriverVersion FROM Win32_PnPSignedDriver \
             WHERE DeviceClass = 'COMPUTEACCELERATOR' OR DeviceClass = 'DISPLAY'",
        )?;
        let processors: Vec<Processor> =
            connection.raw_query("SELECT Name FROM Win32_Processor")?;
        Ok(PlatformDevices {
            accelerators: drivers.into_iter().filter_map(to_accelerator).collect(),
            cpu_name: processors
                .into_iter()
                .find_map(|processor| processor.name)
                .map(|name| name.trim().to_owned())
                .filter(|name| !name.is_empty()),
        })
    }

    fn to_accelerator(driver: SignedDriver) -> Option<Accelerator> {
        let name = driver.device_name?.trim().to_owned();
        let unit = match driver.device_class?.to_ascii_uppercase().as_str() {
            "COMPUTEACCELERATOR" => ComputeUnit::Npu,
            // Skip virtual adapters (remote desktop, basic display driver).
            "DISPLAY" if !name.starts_with("Microsoft") => ComputeUnit::Gpu,
            _ => return None,
        };
        Some(Accelerator {
            unit,
            name,
            driver_version: driver.driver_version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reports_memory_and_the_current_drive() {
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

    #[test]
    fn paths_compare_case_insensitively_on_windows() {
        if cfg!(windows) {
            assert_eq!(
                normalize(Path::new(r"C:\Users")),
                normalize(Path::new("c:/users"))
            );
        }
    }
}
