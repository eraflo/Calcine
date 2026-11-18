//! Live load: CPU and memory from `sysinfo`, NPU and GPU from the OS.

use calcine_core::hardware::{EnergyReading, HardwareUsage, MemoryInfo};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, System};

use crate::power::PowerMeter;
#[cfg(windows)]
use crate::windows::gpu_engines::GpuEngines;

/// Keeps the previous sample so each read measures the time since the last.
#[derive(Debug)]
pub(crate) struct UsageSampler {
    /// `None` when sysinfo couldn't open its system handles.
    system: Option<System>,
    #[cfg(windows)]
    engines: Option<GpuEngines>,
    power: PowerMeter,
}

impl UsageSampler {
    pub(crate) fn new() -> Self {
        let system = System::new()
            .inspect_err(|err| tracing::warn!(%err, "CPU and memory load won't be shown"))
            .ok()
            .map(|mut system| {
                system.refresh_cpu_usage();
                system
            });
        Self {
            system,
            #[cfg(windows)]
            engines: GpuEngines::open()
                .inspect_err(|err| tracing::warn!(%err, "NPU and GPU load won't be shown"))
                .ok(),
            power: PowerMeter::open(),
        }
    }

    pub(crate) fn read(&mut self) -> HardwareUsage {
        let (cpu_percent, memory) = match self.system.as_mut() {
            Some(system) => {
                system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage());
                system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
                (
                    system.global_cpu_usage().clamp(0.0, 100.0),
                    MemoryInfo {
                        total_bytes: system.total_memory(),
                        available_bytes: system.available_memory(),
                    },
                )
            }
            None => (
                0.0,
                MemoryInfo {
                    total_bytes: 0,
                    available_bytes: 0,
                },
            ),
        };
        let (npu_percent, gpu_percent) = self.engine_load();
        HardwareUsage {
            cpu_percent,
            gpu_percent,
            npu_percent,
            memory,
            power: self.power.draw(),
        }
    }

    pub(crate) fn energy(&self) -> Option<EnergyReading> {
        self.power.system_energy()
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
