use std::cmp::Ordering as CmpOrdering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use crate::shared::rate_limiter::traits::RateLimiter;

/// A fake rate limiter that never waits and counts each call to `await_turn`.
///
/// Used for testing how often code waits on its [`RateLimiter`]. Clones share one counter.
#[derive(Debug, Default, Clone)]
pub struct CountingRateLimiter {
    /// The number of calls to `await_turn`, shared by all clones.
    pub turns: Arc<AtomicUsize>,
}

#[async_trait]
impl RateLimiter for CountingRateLimiter {
    async fn await_turn(&self) {
        self.turns.fetch_add(1, Ordering::SeqCst);
    }
}

// Deviation: a client needs `Eq`, `Ord`, and `Hash` to implement `SecClient`. The count changes
// through `&self`, so every limiter compares equal and adds nothing to a hash. Same rule as
// `OutputSlot`.

impl PartialEq for CountingRateLimiter {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for CountingRateLimiter {}

impl PartialOrd for CountingRateLimiter {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for CountingRateLimiter {
    fn cmp(&self, _other: &Self) -> CmpOrdering {
        CmpOrdering::Equal
    }
}

impl Hash for CountingRateLimiter {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
