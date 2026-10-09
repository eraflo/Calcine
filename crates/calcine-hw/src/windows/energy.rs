//! Energy from the `Energy Meter` performance counters, Windows' view of the
//! SoC's energy metering (EMI). The meters themselves only open for
//! administrators; the counters are readable by everyone.
//!
//! On Snapdragon X there are channels for the CPU clusters, the GPU and the
//! whole system (`SYS`), but none for the NPU: what the NPU costs shows in
//! `SYS`.

use std::collections::HashMap;

use super::pdh::{Query, translated_path};

/// Joules in a picowatt-hour.
const JOULES_PER_PICOWATT_HOUR: f64 = 3.6e-9;

/// Energy a channel has used since its meter started.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ChannelEnergy {
    pub name: String,
    pub joules: f64,
    /// When it was measured, in seconds since the meter started.
    pub seconds: f64,
}

/// Live `Energy Meter(*)\Energy` and `\Time` queries.
#[derive(Debug)]
pub(crate) struct EnergyMeters {
    energy: Query,
    time: Query,
}

impl EnergyMeters {
    /// An error on devices without energy metering.
    pub(crate) fn open() -> Result<Self, String> {
        let energy = Query::open_translated(&translated_path("Energy Meter", "Energy")?)?;
        let time = Query::open_translated(&translated_path("Energy Meter", "Time")?)?;
        let meters = Self { energy, time };
        if meters.read()?.is_empty() {
            return Err("this device has no energy metering".into());
        }
        Ok(meters)
    }

    /// Energy of every channel, `_Total` left out (it isn't a sum here).
    pub(crate) fn read(&self) -> Result<Vec<ChannelEnergy>, String> {
        let times: HashMap<String, f64> = self.time.read()?.into_iter().collect();
        Ok(self
            .energy
            .read()?
            .into_iter()
            .filter(|(name, _)| name != "_Total")
            .map(|(name, picowatt_hours)| ChannelEnergy {
                // Milliseconds since the meter started.
                seconds: times.get(&name).copied().unwrap_or(0.0) / 1000.0,
                joules: picowatt_hours * JOULES_PER_PICOWATT_HOUR,
                name,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "needs a device with energy metering"]
    fn reads_the_meters() {
        let meters = EnergyMeters::open().unwrap();
        let first = meters.read().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(3));
        let second = meters.read().unwrap();
        for (a, b) in first.iter().zip(&second) {
            let watts = (b.joules - a.joules) / (b.seconds - a.seconds);
            println!(
                "{}: {watts:.2} W over {:.3} s",
                a.name,
                b.seconds - a.seconds
            );
        }
    }
}
