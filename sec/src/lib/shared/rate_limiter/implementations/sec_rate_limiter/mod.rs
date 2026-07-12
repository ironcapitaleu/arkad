//! # SEC Rate Limiter
//!
//! Provides the [`SecRateLimiter`], the rate limiter configured for the SEC EDGAR API.
//!
//! ## Modules
//!
//! - [`constants`]: Tuning constants such as the minimum spacing between requests.

use async_trait::async_trait;

use crate::shared::rate_limiter::implementations::governor::GovernorRateLimiter;
use crate::shared::rate_limiter::traits::RateLimiter;

use self::constants::MIN_REQUEST_INTERVAL;

pub mod constants;

/// A leaky-bucket rate limiter configured for the SEC EDGAR API.
///
/// The SEC EDGAR API enforces a limit of 10 requests per second per originating IP; exceeding it
/// risks the IP being throttled or temporarily blocked. This limiter paces callers to one every
/// [`MIN_REQUEST_INTERVAL`] (≈ 9 requests/second), keeping throughput safely under that ceiling.
///
/// Wraps a generic [`GovernorRateLimiter`], supplying only the SEC-specific pacing policy. It is a
/// **single-process, single-machine, in-memory** limiter — not distributed — so its budget is
/// enforced only among clones that share one instance within a single process.
///
/// # Sharing
///
/// Cloning an [`SecRateLimiter`] shares — rather than duplicates — the underlying limiter state, so
/// every clone draws from **one** common budget. This is what makes the rate cap hold across all
/// clones.
#[derive(Debug, Clone)]
pub struct SecRateLimiter {
    inner: GovernorRateLimiter,
}

impl SecRateLimiter {
    /// Creates a new [`SecRateLimiter`] paced to stay under the SEC's request-rate ceiling.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::rate_limiter::SecRateLimiter;
    ///
    /// let _limiter = SecRateLimiter::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: GovernorRateLimiter::new(MIN_REQUEST_INTERVAL),
        }
    }

    /// Returns a reference to the underlying generic limiter.
    #[must_use]
    pub const fn inner(&self) -> &GovernorRateLimiter {
        &self.inner
    }
}

impl Default for SecRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RateLimiter for SecRateLimiter {
    async fn await_turn(&self) {
        self.inner.await_turn().await;
    }
}

#[cfg(test)]
mod tests {
    use super::SecRateLimiter;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_unpin<T: Unpin>() {}
    fn assert_debug<T: std::fmt::Debug>() {}
    fn assert_clone<T: Clone>() {}

    #[test]
    fn should_be_send() {
        assert_send::<SecRateLimiter>();
    }

    #[test]
    fn should_be_sync() {
        assert_sync::<SecRateLimiter>();
    }

    #[test]
    fn should_be_unpin() {
        assert_unpin::<SecRateLimiter>();
    }

    #[test]
    fn should_implement_debug() {
        assert_debug::<SecRateLimiter>();
    }

    #[test]
    fn should_implement_clone() {
        assert_clone::<SecRateLimiter>();
    }
}
