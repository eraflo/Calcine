//! Follows the request log: reads the energy meter when a request starts
//! and ends, counts the request once it's finished, and saves now and then.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::time::Duration;

use calcine_core::hardware::{EnergyReading, HardwareProbe};
use tokio::sync::broadcast::error::RecvError;

use super::Usage;
use crate::log::{RequestEntry, RequestLog};

/// How often recorded usage is written to disk.
const SAVE_EVERY: Duration = Duration::from_secs(60);

/// Record finished requests into `usage` until the log closes.
pub(super) fn spawn(
    usage: Arc<Usage>,
    log: &RequestLog,
    hardware: Arc<dyn HardwareProbe>,
) -> Option<tokio::task::JoinHandle<()>> {
    let runtime = tokio::runtime::Handle::try_current().ok()?;
    let mut events = log.subscribe();
    Some(runtime.spawn(async move {
        // Meter readings of the requests in flight.
        let mut started: HashMap<u32, Option<EnergyReading>> = HashMap::new();
        let mut save = tokio::time::interval(SAVE_EVERY);
        loop {
            tokio::select! {
                event = events.recv() => match event {
                    Ok(entry) => on_event(&usage, hardware.as_ref(), &mut started, &entry).await,
                    Err(RecvError::Lagged(missed)) => {
                        tracing::debug!(missed, "usage missed some request updates");
                    }
                    Err(RecvError::Closed) => break,
                },
                _ = save.tick() => usage.save(),
            }
        }
        usage.save();
    }))
}

async fn on_event(
    usage: &Usage,
    hardware: &dyn HardwareProbe,
    started: &mut HashMap<u32, Option<EnergyReading>>,
    entry: &RequestEntry,
) {
    let finished = entry.status.is_some() && entry.duration_ms.is_some();
    if !finished {
        if let Entry::Vacant(slot) = started.entry(entry.id) {
            slot.insert(read_meter(hardware).await);
        }
        return;
    }
    let Some(start) = started.remove(&entry.id) else {
        // Already counted, or started before Calcine followed the log.
        return;
    };
    let energy = match (start, read_meter(hardware).await) {
        (Some(start), Some(end)) if end.joules >= start.joules => Some(end.joules - start.joules),
        _ => None,
    };
    usage.record(entry, energy);
}

async fn read_meter(hardware: &dyn HardwareProbe) -> Option<EnergyReading> {
    hardware.energy().await.ok().flatten()
}
