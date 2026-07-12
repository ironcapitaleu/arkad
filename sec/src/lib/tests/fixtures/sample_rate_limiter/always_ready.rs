use async_trait::async_trait;

use crate::shared::rate_limiter::traits::RateLimiter;

/// A fake rate limiter that always permits requests immediately.
///
/// Used for testing code that depends on a [`RateLimiter`] without incurring any real pacing
/// delay, by simulating a limiter that never blocks.
#[derive(Debug, Clone, PartialEq)]
pub struct AlwaysReadyRateLimiter;

#[async_trait]
impl RateLimiter for AlwaysReadyRateLimiter {
    async fn await_turn(&self) {}
}

#[cfg(test)]
mod tests {
    use super::AlwaysReadyRateLimiter;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_unpin<T: Unpin>() {}
    fn assert_debug<T: std::fmt::Debug>() {}
    fn assert_clone<T: Clone>() {}
    fn assert_partial_eq<T: PartialEq>() {}

    #[test]
    fn should_be_send() {
        assert_send::<AlwaysReadyRateLimiter>();
    }

    #[test]
    fn should_be_sync() {
        assert_sync::<AlwaysReadyRateLimiter>();
    }

    #[test]
    fn should_be_unpin() {
        assert_unpin::<AlwaysReadyRateLimiter>();
    }

    #[test]
    fn should_implement_debug() {
        assert_debug::<AlwaysReadyRateLimiter>();
    }

    #[test]
    fn should_implement_clone() {
        assert_clone::<AlwaysReadyRateLimiter>();
    }

    #[test]
    fn should_implement_partial_eq() {
        assert_partial_eq::<AlwaysReadyRateLimiter>();
    }
}
