use dashmap::DashMap;
use std::{
    hash::Hash,
    time::{Duration, Instant},
};

/// Fixed-window rate limiter keyed by anything hashable (typically an `IpAddr`).
/// In-memory only — resets on restart and isn't shared across instances if you
/// ever scale horizontally, but that's a fine tradeoff for a single-instance API.
pub struct RateLimiter<K: Eq + Hash + Clone> {
    max_requests: u32,
    window: Duration,
    hits: DashMap<K, (u32, Instant)>,
}

impl<K: Eq + Hash + Clone> RateLimiter<K> {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self { max_requests, window, hits: DashMap::new() }
    }

    /// Returns `true` if this call is allowed under the limit, `false` if the
    /// caller should be rejected (e.g. with a 429).
    pub fn check(&self, key: K) -> bool {
        let mut entry = self.hits.entry(key).or_insert((0, Instant::now()));
        if entry.1.elapsed() > self.window {
            *entry = (0, Instant::now());
        }
        if entry.0 >= self.max_requests {
            false
        } else {
            entry.0 += 1;
            true
        }
    }

    pub fn cleanup(&self) {
        let window = self.window;
        self.hits.retain(|_, (_, started)| started.elapsed() <= window * 2);
    }
}