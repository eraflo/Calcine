//! Battery or mains, asked of the OS at most every [`FRESH_FOR`]: the
//! gateway checks before each request.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use calcine_core::hardware::PowerSource;

const FRESH_FOR: Duration = Duration::from_secs(10);

#[derive(Debug, Default)]
pub(crate) struct BatteryCache {
    last: Mutex<Option<(Instant, Option<PowerSource>)>>,
}

impl BatteryCache {
    pub fn read(&self) -> Option<PowerSource> {
        let mut last = self
            .last
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((at, source)) = *last
            && at.elapsed() < FRESH_FOR
        {
            return source;
        }
        let source = read_now();
        *last = Some((Instant::now(), source));
        source
    }
}

#[cfg(windows)]
fn read_now() -> Option<PowerSource> {
    crate::windows::battery::power_source()
        .inspect_err(|err| tracing::debug!(%err, "couldn't read the battery"))
        .ok()
        .flatten()
}

#[cfg(target_os = "linux")]
fn read_now() -> Option<PowerSource> {
    crate::linux::power_source()
}

#[cfg(not(any(windows, target_os = "linux")))]
fn read_now() -> Option<PowerSource> {
    None
}
