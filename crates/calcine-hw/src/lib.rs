//! Hardware probes.
//!
//! - `snapshot`: what the machine has (CPU, memory, disk, accelerators)
//! - `usage`: live load, sampled on demand
//! - `power`: power and energy, on devices with energy metering
//! - `system`: CPU, memory and disk, portable (`sysinfo`)
//! - `windows`: Hexagon NPU and Adreno GPU, from the driver database (WMI)
//!   and the performance counters. Other platforms report no accelerators
//!   or accelerator load for now.

mod power;
mod snapshot;
mod system;
mod usage;
#[cfg(windows)]
mod windows;

use std::path::Path;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use calcine_core::hardware::{EnergyReading, HardwareInfo, HardwareProbe, HardwareUsage};
use calcine_core::{Error, Result};

use crate::usage::UsageSampler;

/// Probe for the machine Calcine runs on.
#[derive(Debug, Clone, Default)]
pub struct SystemProbe {
    /// Opened on first use, then kept so each read measures since the last.
    sampler: Arc<Mutex<Option<UsageSampler>>>,
}

impl SystemProbe {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl HardwareProbe for SystemProbe {
    async fn snapshot(&self, models_dir: Option<&Path>) -> Result<HardwareInfo> {
        let models_dir = models_dir.map(Path::to_path_buf);
        // WMI and sysinfo are blocking (WMI also needs COM on its own thread).
        blocking(move || snapshot::snapshot(models_dir.as_deref())).await
    }

    async fn usage(&self) -> Result<HardwareUsage> {
        let sampler = self.sampler.clone();
        blocking(move || {
            let mut sampler = sampler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            sampler.get_or_insert_with(UsageSampler::new).read()
        })
        .await
    }

    async fn energy(&self) -> Result<Option<EnergyReading>> {
        let sampler = self.sampler.clone();
        blocking(move || {
            let mut sampler = sampler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            sampler.get_or_insert_with(UsageSampler::new).energy()
        })
        .await
    }
}

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Result<T> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|err| Error::Io(std::io::Error::other(err)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reports_cpu_memory_and_the_current_drive() {
        let dir = std::env::current_dir().unwrap();
        let info = SystemProbe::new().snapshot(Some(&dir)).await.unwrap();
        assert!(info.memory.total_bytes > 0);
        let cpu = info.cpu.expect("every machine has a CPU");
        assert_ne!(cpu.name, "");
        let disk = info
            .models_disk
            .expect("the current directory is on some drive");
        assert!(disk.total_bytes >= disk.available_bytes);
    }

    #[tokio::test]
    async fn samples_usage_repeatedly() {
        let probe = SystemProbe::new();
        let _ = probe.usage().await.unwrap();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        let usage = probe.usage().await.unwrap();
        assert!((0.0..=100.0).contains(&usage.cpu_percent));
        assert!(usage.memory.total_bytes >= usage.memory.available_bytes);
        if cfg!(windows) {
            assert!(usage.npu_percent.is_some() && usage.gpu_percent.is_some());
        }
    }
}
