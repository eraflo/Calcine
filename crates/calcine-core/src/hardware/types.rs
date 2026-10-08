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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DiskSpace {
    /// Directory measured (the model cache).
    pub path: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}
