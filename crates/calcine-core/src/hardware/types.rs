use serde::{Deserialize, Serialize};
use specta::Type;

use crate::models::ComputeUnit;

/// What this device offers for inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    pub cpu: Option<Processor>,
    pub memory: MemoryInfo,
    /// NPUs and GPUs found on the system.
    pub accelerators: Vec<Accelerator>,
    /// Space on the drive holding the model cache.
    pub models_disk: Option<DiskSpace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Processor {
    pub name: String,
    pub cores: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Accelerator {
    pub unit: ComputeUnit,
    pub name: String,
    pub driver_version: Option<String>,
}

/// Live load of the compute units, sampled about once a second.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HardwareUsage {
    /// Average over all cores, 0-100.
    pub cpu_percent: f32,
    /// Busiest GPU engine, 0-100. `None` when the OS doesn't report it.
    pub gpu_percent: Option<f32>,
    /// Hexagon NPU, 0-100. `None` when the OS doesn't report it.
    pub npu_percent: Option<f32>,
    pub memory: MemoryInfo,
    /// Power drawn, on devices with energy metering.
    pub power: Option<PowerDraw>,
}

/// Power drawn since the previous sample, in watts. Snapdragon X meters the
/// CPU clusters, the GPU and the whole system, not the NPU on its own.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PowerDraw {
    pub system_watts: f32,
    pub cpu_watts: Option<f32>,
    pub gpu_watts: Option<f32>,
}

/// Energy the whole system has used since its meter started.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyReading {
    pub joules: f64,
    /// When it was measured, in seconds since the meter started.
    pub seconds: f64,
}

impl EnergyReading {
    /// Average power between `self` and a later reading. `None` when no time
    /// passed.
    pub fn watts_until(&self, later: &Self) -> Option<f64> {
        let seconds = later.seconds - self.seconds;
        (seconds > 0.0).then(|| (later.joules - self.joules) / seconds)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DiskSpace {
    /// Directory measured (the model cache).
    pub path: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}
