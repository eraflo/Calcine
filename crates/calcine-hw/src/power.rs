//! Power and energy from the device's energy metering, when it has some.
//!
//! The channels are named by the platform: on Snapdragon X, `SYS` is the
//! whole system, `CPU_CLUSTER_0`… the CPU clusters and `GPU` the Adreno GPU.
//! There is no channel for the NPU: its work shows in `SYS`.

use calcine_core::hardware::{EnergyReading, PowerDraw};

#[cfg(windows)]
use crate::windows::energy::EnergyMeters;

/// Energy a channel has used since its meter started.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Channel {
    pub name: String,
    pub joules: f64,
    pub seconds: f64,
}

/// Reads the meters and remembers the last reading, so each draw is the
/// average power since the previous one.
#[derive(Debug, Default)]
pub(crate) struct PowerMeter {
    #[cfg(windows)]
    meters: Option<EnergyMeters>,
    previous: Option<Vec<Channel>>,
}

impl PowerMeter {
    pub(crate) fn open() -> Self {
        Self {
            #[cfg(windows)]
            meters: EnergyMeters::open()
                .inspect_err(|err| tracing::info!(%err, "power won't be shown"))
                .ok(),
            previous: None,
        }
    }

    #[cfg(windows)]
    fn read(&self) -> Option<Vec<Channel>> {
        let channels = self.meters.as_ref()?.read();
        channels
            .inspect_err(|err| tracing::debug!(%err, "couldn't read the energy meters"))
            .ok()
            .map(|channels| {
                channels
                    .into_iter()
                    .map(|channel| Channel {
                        name: channel.name,
                        joules: channel.joules,
                        seconds: channel.seconds,
                    })
                    .collect()
            })
    }

    #[cfg(not(windows))]
    #[allow(clippy::unused_self)]
    fn read(&self) -> Option<Vec<Channel>> {
        None
    }

    /// Power since the previous call. `None` without metering, and on the
    /// first call.
    pub(crate) fn draw(&mut self) -> Option<PowerDraw> {
        let current = self.read()?;
        let previous = self.previous.replace(current.clone())?;
        draw_between(&previous, &current)
    }

    /// Energy the whole system has used so far.
    pub(crate) fn system_energy(&self) -> Option<EnergyReading> {
        system(&self.read()?)
    }
}

fn is_system(name: &str) -> bool {
    name.eq_ignore_ascii_case("SYS")
}

fn is_cpu(name: &str) -> bool {
    name.to_ascii_uppercase().starts_with("CPU")
}

fn is_gpu(name: &str) -> bool {
    name.eq_ignore_ascii_case("GPU")
}

fn system(channels: &[Channel]) -> Option<EnergyReading> {
    channels
        .iter()
        .find(|channel| is_system(&channel.name))
        .map(|channel| EnergyReading {
            joules: channel.joules,
            seconds: channel.seconds,
        })
}

/// Average power of each kind of channel between two readings.
#[allow(clippy::cast_possible_truncation)]
fn draw_between(before: &[Channel], after: &[Channel]) -> Option<PowerDraw> {
    let watts = |pick: fn(&str) -> bool| -> Option<f32> {
        let mut total = None;
        for channel in after.iter().filter(|channel| pick(&channel.name)) {
            let earlier = before.iter().find(|other| other.name == channel.name)?;
            let seconds = channel.seconds - earlier.seconds;
            if seconds <= 0.0 {
                return None;
            }
            let channel_watts = (channel.joules - earlier.joules) / seconds;
            *total.get_or_insert(0.0) += channel_watts;
        }
        total.map(|watts: f64| watts.max(0.0) as f32)
    };
    Some(PowerDraw {
        system_watts: watts(is_system)?,
        cpu_watts: watts(is_cpu),
        gpu_watts: watts(is_gpu),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channel(name: &str, joules: f64, seconds: f64) -> Channel {
        Channel {
            name: name.into(),
            joules,
            seconds,
        }
    }

    #[test]
    fn averages_each_kind_of_channel() {
        let before = [
            channel("SYS", 100.0, 10.0),
            channel("CPU_CLUSTER_0", 10.0, 10.0),
            channel("CPU_CLUSTER_1", 20.0, 10.0),
            channel("GPU", 1.0, 10.0),
            channel("PSU_USB", 5.0, 10.0),
        ];
        let after = [
            channel("SYS", 130.0, 12.0),
            channel("CPU_CLUSTER_0", 16.0, 12.0),
            channel("CPU_CLUSTER_1", 22.0, 12.0),
            channel("GPU", 1.0, 12.0),
            channel("PSU_USB", 50.0, 12.0),
        ];
        assert_eq!(
            draw_between(&before, &after),
            Some(PowerDraw {
                system_watts: 15.0,
                cpu_watts: Some(4.0),
                gpu_watts: Some(0.0),
            })
        );
        assert_eq!(
            system(&after),
            Some(EnergyReading {
                joules: 130.0,
                seconds: 12.0
            })
        );
    }

    #[test]
    fn needs_a_system_channel_and_time_passing() {
        let gpu = [channel("GPU", 1.0, 10.0)];
        assert_eq!(draw_between(&gpu, &gpu), None);
        let sys = [channel("SYS", 1.0, 10.0)];
        assert_eq!(draw_between(&sys, &sys), None);
        assert_eq!(system(&gpu), None);
    }
}
