//! Live load: CPU and memory from `sysinfo`, NPU and GPU from the OS.

use calcine_core::hardware::{HardwareUsage, MemoryInfo};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, System};

#[cfg(windows)]
use crate::windows::gpu_engines::GpuEngines;

/// Keeps the previous sample so each read measures the time since the last.
#[derive(Debug)]
pub(crate) struct UsageSampler {
    system: System,
    #[cfg(windows)]
    engines: Option<GpuEngines>,
}

impl UsageSampler {
    pub(crate) fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_usage();
        Self {
            system,
            #[cfg(windows)]
            engines: GpuEngines::open()
                .inspect_err(|err| tracing::warn!(%err, "NPU and GPU load won't be shown"))
                .ok(),
        }
    }

    pub(crate) fn read(&mut self) -> HardwareUsage {
        self.system
            .refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage());
        self.system
            .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        let (npu_percent, gpu_percent) = self.engine_load();
        HardwareUsage {
            cpu_percent: self.system.global_cpu_usage().clamp(0.0, 100.0),
            gpu_percent,
            npu_percent,
            memory: MemoryInfo {
                total_bytes: self.system.total_memory(),
                available_bytes: self.system.available_memory(),
            },
        }
    }

    #[cfg(windows)]
    fn engine_load(&self) -> (Option<f32>, Option<f32>) {
        match self.engines.as_ref().map(GpuEngines::read) {
            Some(Ok(load)) => (Some(load.npu), Some(load.gpu)),
            Some(Err(err)) => {
                tracing::debug!(%err, "couldn't read NPU and GPU load");
                (None, None)
            }
            None => (None, None),
        }
    }

    #[cfg(not(windows))]
    #[allow(clippy::unused_self)]
    fn engine_load(&self) -> (Option<f32>, Option<f32>) {
        (None, None)
    }
}
