//! The device: CPU, NPU, GPU, memory and storage.

mod probe;
mod types;

pub use probe::HardwareProbe;
pub use types::{Accelerator, DiskSpace, HardwareInfo, HardwareUsage, MemoryInfo, Processor};
