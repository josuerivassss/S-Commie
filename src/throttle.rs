use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Spaces out calls through `wait()` to at least `min_interval` apart.
pub struct Throttle {
    min_interval: Duration,
    last: Mutex<Instant>,
}

impl Throttle {
    pub fn new(min_interval: Duration) -> Self {
        // seed `last` far enough in the past so the very first call never waits
        Self { min_interval, last: Mutex::new(Instant::now() - min_interval) }
    }

    /// Blocks (only this call, not the whole server) until `min_interval` has
    /// passed since the last call that went through this throttle.
    pub async fn wait(&self) {
        let mut last = self.last.lock().await;
        let elapsed = last.elapsed();
        if elapsed < self.min_interval {
            tokio::time::sleep(self.min_interval - elapsed).await;
        }
        *last = Instant::now();
    }
}