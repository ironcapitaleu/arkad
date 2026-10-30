use std::cmp::Ordering as CmpOrdering;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use crate::shared::rate_limiter::traits::RateLimiter;

/// A fake rate limiter that never waits and counts each call to `await_turn`.
///
/// Used for testing how often code waits on its [`RateLimiter`].
#[derive(Debug, Default)]
pub struct CountingRateLimiter {
    /// The number of calls to `await_turn`.
    pub turns: AtomicUsize,
}

#[async_trait]
impl RateLimiter for CountingRateLimiter {
    async fn await_turn(&self) {
        self.turns.fetch_add(1, Ordering::SeqCst);
    }
}

impl CountingRateLimiter {
    fn count(&self) -> usize {
        self.turns.load(Ordering::SeqCst)
    }
}

// Deviation: `AtomicUsize` implements none of `Clone`, `Eq`, `Ord`, and `Hash`. A client needs
// them to implement `SecClient`, so these impls use the current count.

impl Clone for CountingRateLimiter {
    fn clone(&self) -> Self {
        Self {
            turns: AtomicUsize::new(self.count()),
        }
    }
}

impl PartialEq for CountingRateLimiter {
    fn eq(&self, other: &Self) -> bool {
        self.count() == other.count()
    }
}

impl Eq for CountingRateLimiter {}

impl PartialOrd for CountingRateLimiter {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for CountingRateLimiter {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        self.count().cmp(&other.count())
    }
}

impl Hash for CountingRateLimiter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.count().hash(state);
    }
}
