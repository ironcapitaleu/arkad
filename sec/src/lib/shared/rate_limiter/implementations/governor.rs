//! # Governor Rate Limiter
//!
//! Provides the [`GovernorRateLimiter`], a [`RateLimiter`] backed by the `governor` crate's GCRA
//! implementation, configured as a leaky bucket (burst = 1).

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter as GovernorLimiter};

use crate::shared::rate_limiter::traits::RateLimiter;

/// A leaky-bucket rate limiter backed by the `governor` crate.
///
/// Paces callers to one call every configured period, e.g. one every 100 ms yields ≈ 10 permits per
/// second. A [`Quota::with_period`] quota implies a burst size of one, giving strict leaky-bucket
/// pacing rather than token-bucket bursting.
///
/// The pacing period is a construction parameter, so this limiter carries no policy of its own —
/// callers choose the rate that suits their target.
///
/// # Sharing
///
/// The underlying limiter is held behind an `Arc`, so cloning a [`GovernorRateLimiter`] shares —
/// rather than duplicates — the limiter state. Every clone therefore draws from **one** common
/// budget, which is what makes the rate cap hold across all clones.
#[derive(Debug, Clone)]
pub struct GovernorRateLimiter {
    inner: Arc<DefaultDirectRateLimiter>,
}

impl GovernorRateLimiter {
    /// Creates a new [`GovernorRateLimiter`] that permits one caller every `period`.
    ///
    /// # Panics
    ///
    /// Panics if `period` is zero, since a zero replenish interval cannot form a valid quota.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use sec::shared::rate_limiter::GovernorRateLimiter;
    ///
    /// let _limiter = GovernorRateLimiter::new(Duration::from_millis(100));
    /// ```
    #[must_use]
    pub fn new(period: Duration) -> Self {
        let quota =
            Quota::with_period(period).expect("The pacing period should be a non-zero interval");
        Self {
            inner: Arc::new(GovernorLimiter::direct(quota)),
        }
    }
}

#[async_trait]
impl RateLimiter for GovernorRateLimiter {
    async fn await_turn(&self) {
        self.inner.until_ready().await;
    }
}

#[cfg(test)]
mod tests {
    use super::GovernorRateLimiter;

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}
    const fn implements_unpin<T: Unpin>() {}
    const fn implements_debug<T: std::fmt::Debug>() {}
    const fn implements_clone<T: Clone>() {}

    #[test]
    const fn should_be_send_for_governor_rate_limiter() {
        implements_send::<GovernorRateLimiter>();
    }

    #[test]
    const fn should_be_sync_for_governor_rate_limiter() {
        implements_sync::<GovernorRateLimiter>();
    }

    #[test]
    const fn should_be_unpin_for_governor_rate_limiter() {
        implements_unpin::<GovernorRateLimiter>();
    }

    #[test]
    const fn should_implement_debug_for_governor_rate_limiter() {
        implements_debug::<GovernorRateLimiter>();
    }

    #[test]
    const fn should_implement_clone_for_governor_rate_limiter() {
        implements_clone::<GovernorRateLimiter>();
    }
}
