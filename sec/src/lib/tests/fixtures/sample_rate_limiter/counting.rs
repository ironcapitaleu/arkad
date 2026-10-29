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
