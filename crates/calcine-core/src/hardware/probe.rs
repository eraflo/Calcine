use std::path::Path;

use async_trait::async_trait;

use super::types::{HardwareInfo, HardwareUsage};
use crate::Result;

/// The device's compute units, memory and storage.
#[async_trait]
pub trait HardwareProbe: Send + Sync {
    /// A one-off description of the hardware. `models_dir` selects which drive
    /// to report free space for.
    async fn snapshot(&self, models_dir: Option<&Path>) -> Result<HardwareInfo>;

    /// Current load. Percentages are measured since the previous call, so
    /// the first call may read 0.
    async fn usage(&self) -> Result<HardwareUsage>;
}
