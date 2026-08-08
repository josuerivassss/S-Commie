use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const CLOSED: u8 = 0;
const OPEN: u8 = 1;
const HALF_OPEN: u8 = 2;

/// Per-route circuit breaker for shared external services. Opens after
/// `failure_threshold` consecutive failures, stays open for `cooldown_secs`,
/// then lets exactly one probe through before fully closing. Lock-free.
pub struct CircuitBreaker {
    state: AtomicU8,
    consecutive_failures: AtomicU32,
    opened_at: AtomicU64,
    failure_threshold: u32,
    cooldown_secs: u64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown_secs: u64) -> Self {
        Self { state: AtomicU8::new(CLOSED), consecutive_failures: AtomicU32::new(0), opened_at: AtomicU64::new(0), failure_threshold, cooldown_secs }
    }

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
    }

    /// True if the call may proceed. Flips OPEN -> HALF_OPEN once cooldown
    /// elapses, letting only the thread that performs the flip through.
    pub fn allow(&self) -> bool {
        match self.state.load(Ordering::Acquire) {
            CLOSED => true,
            OPEN => {
                let elapsed = Self::now().saturating_sub(self.opened_at.load(Ordering::Acquire));
                elapsed >= self.cooldown_secs
                    && self.state.compare_exchange(OPEN, HALF_OPEN, Ordering::AcqRel, Ordering::Acquire).is_ok()
            }
            _ => false, // HALF_OPEN probe already in flight
        }
    }

    pub fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Release);
        self.state.store(CLOSED, Ordering::Release);
    }

    pub fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::AcqRel) + 1;
        if failures >= self.failure_threshold {
            self.opened_at.store(Self::now(), Ordering::Release);
            self.state.store(OPEN, Ordering::Release);
        }
    }
}