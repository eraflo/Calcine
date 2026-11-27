use std::path::Path;

use async_trait::async_trait;
use calcine_core::Result;
use calcine_core::hardware::{
    EnergyReading, HardwareInfo, HardwareProbe, HardwareUsage, PowerSource,
};

use crate::{MockBackend, data};

#[async_trait]
impl HardwareProbe for MockBackend {
    async fn snapshot(&self, models_dir: Option<&Path>) -> Result<HardwareInfo> {
        Ok(data::hardware(models_dir))
    }

    async fn usage(&self) -> Result<HardwareUsage> {
        Ok(data::usage(self.started.elapsed().as_secs_f32()))
    }

    /// A laptop on mains, or on battery with `CALCINE_MOCK_ON_BATTERY=1`.
    async fn power_source(&self) -> Result<Option<PowerSource>> {
        Ok(Some(PowerSource {
            on_battery: std::env::var("CALCINE_MOCK_ON_BATTERY").is_ok_and(|value| value == "1"),
            battery_percent: Some(72),
        }))
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
