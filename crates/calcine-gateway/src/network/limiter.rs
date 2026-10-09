//! Slows down key guessing over the network: an address that sends too many
//! invalid keys is turned away for a while.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

/// Invalid keys allowed per address within `WINDOW`.
const MAX_FAILURES: u32 = 10;
const WINDOW: Duration = Duration::from_secs(60);
/// How long an address is turned away after that.
const BLOCKED_FOR: Duration = Duration::from_secs(300);

#[derive(Debug, Default)]
pub struct Limiter {
    addresses: Mutex<HashMap<IpAddr, Record>>,
}

#[derive(Debug, Clone, Copy)]
struct Record {
    failures: u32,
    since: Instant,
    blocked_until: Option<Instant>,
}

impl Limiter {
    /// How long `address` is still turned away, if it is.
    pub fn blocked(&self, address: IpAddr, now: Instant) -> Option<Duration> {
        let addresses = self
            .addresses
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        addresses
            .get(&address)?
            .blocked_until
            .filter(|until| *until > now)
            .map(|until| until - now)
    }

    /// Count an invalid key from `address`.
    pub fn failed(&self, address: IpAddr, now: Instant) {
        let mut addresses = self
            .addresses
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        // Forget what's old, so the map doesn't grow forever.
        addresses.retain(|_, record| {
            now.duration_since(record.since) < WINDOW
                || record.blocked_until.is_some_and(|until| until > now)
        });
        let record = addresses.entry(address).or_insert(Record {
            failures: 0,
            since: now,
            blocked_until: None,
        });
        if now.duration_since(record.since) >= WINDOW {
            *record = Record {
                failures: 0,
                since: now,
                blocked_until: None,
            };
        }
        record.failures += 1;
        if record.failures >= MAX_FAILURES {
            record.blocked_until = Some(now + BLOCKED_FOR);
            tracing::warn!(%address, "too many invalid API keys from the network, blocking");
        }
    }

    /// A valid key clears the count.
    pub fn succeeded(&self, address: IpAddr) {
        self.addresses
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&address);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_too_many_failures() {
        let limiter = Limiter::default();
        let address: IpAddr = "192.168.1.9".parse().unwrap();
        let start = Instant::now();
        for _ in 0..MAX_FAILURES - 1 {
            limiter.failed(address, start);
        }
        assert_eq!(limiter.blocked(address, start), None);
        limiter.failed(address, start);
        assert_eq!(limiter.blocked(address, start), Some(BLOCKED_FOR));
        assert_eq!(limiter.blocked(address, start + BLOCKED_FOR), None);
        // Another address isn't affected.
        assert_eq!(
            limiter.blocked("192.168.1.10".parse().unwrap(), start),
            None
        );
    }

    #[test]
    fn old_failures_and_successes_reset_the_count() {
        let limiter = Limiter::default();
        let address: IpAddr = "10.0.0.2".parse().unwrap();
        let start = Instant::now();
        for _ in 0..MAX_FAILURES - 1 {
            limiter.failed(address, start);
        }
        limiter.failed(address, start + WINDOW);
        assert_eq!(limiter.blocked(address, start + WINDOW), None);
        for _ in 0..MAX_FAILURES - 2 {
            limiter.failed(address, start + WINDOW);
        }
        limiter.succeeded(address);
        limiter.failed(address, start + WINDOW);
        assert_eq!(limiter.blocked(address, start + WINDOW), None);
    }
}
