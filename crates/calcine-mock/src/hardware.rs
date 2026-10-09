use std::path::Path;

use async_trait::async_trait;
use calcine_core::Result;
use calcine_core::hardware::{EnergyReading, HardwareInfo, HardwareProbe, HardwareUsage};

use crate::{MockBackend, data};

#[async_trait]
impl HardwareProbe for MockBackend {
    async fn snapshot(&self, models_dir: Option<&Path>) -> Result<HardwareInfo> {
        Ok(data::hardware(models_dir))
    }

    async fn usage(&self) -> Result<HardwareUsage> {
        Ok(data::usage(self.started.elapsed().as_secs_f32()))
    }

    /// Idle draw since start, plus what the mock server's generations cost.
    async fn energy(&self) -> Result<Option<EnergyReading>> {
        let seconds = self.started.elapsed().as_secs_f64();
        Ok(Some(EnergyReading {
            joules: data::IDLE_WATTS * seconds + crate::server::generation_joules(),
            seconds,
        }))
    }
}
