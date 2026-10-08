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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DiskSpace {
    /// Directory measured (the model cache).
    pub path: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}
