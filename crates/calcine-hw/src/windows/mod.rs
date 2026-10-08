//! Windows-only probes.
//!
//! - `devices`: Hexagon NPU, Adreno GPU and the CPU name, from the driver
//!   database (WMI)
//! - `gpu_engines`: NPU and GPU load, from the `GPU Engine` performance
//!   counters (PDH), like Task Manager

pub(crate) mod devices;
pub(crate) mod gpu_engines;
