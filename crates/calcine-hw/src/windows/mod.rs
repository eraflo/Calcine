//! Windows-only probes.
//!
//! - `devices`: Hexagon NPU, Adreno GPU and the CPU name, from the driver
//!   database (WMI)
//! - `gpu_engines`: NPU and GPU load, from the `GPU Engine` performance
//!   counters (PDH), like Task Manager
//! - `energy`: power drawn by the system, CPU clusters and GPU, from the
//!   `Energy Meter` counters (the SoC's energy metering)

pub(crate) mod devices;
pub(crate) mod energy;
pub(crate) mod gpu_engines;
mod pdh;
