use dashmap::DashMap;
use std::time::{Duration, Instant};

const WINDOW: Duration = Duration::from_secs(24 * 60 * 60);

/// Per-key daily counter, in memory.
pub struct QuotaTracker {
    hits: DashMap<String, (u32, Instant)>,
}

impl QuotaTracker {
    pub fn new() -> Self {
        Self { hits: DashMap::new() }
    }

    pub fn check(&self, key_hash: &str, daily_limit: u32) -> bool {
        let mut entry = self.hits.entry(key_hash.to_string()).or_insert((0, Instant::now()));
        if entry.1.elapsed() > WINDOW {
            *entry = (0, Instant::now());
        }
        if entry.0 >= daily_limit {
            false
        } else {
            entry.0 += 1;
            true
        }
    }

    /// Bounds memory for keys that go idle. Call periodically.
    pub fn cleanup(&self) {
        self.hits.retain(|_, (_, started)| started.elapsed() <= WINDOW * 2);
    }

    pub fn peek(&self, key_hash: &str) -> u32 {
        match self.hits.get(key_hash) {
            Some(e) if e.value().1.elapsed() <= WINDOW => e.value().0,
            _ => 0,
        }
    }
}