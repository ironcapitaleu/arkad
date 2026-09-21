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

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}
    const fn implements_unpin<T: Unpin>() {}
    const fn implements_debug<T: std::fmt::Debug>() {}
    const fn implements_clone<T: Clone>() {}
    const fn implements_partial_eq<T: PartialEq>() {}

    #[test]
    const fn should_be_send_for_always_ready_rate_limiter() {
        implements_send::<AlwaysReadyRateLimiter>();
    }

    #[test]
    const fn should_be_sync_for_always_ready_rate_limiter() {
        implements_sync::<AlwaysReadyRateLimiter>();
    }

    #[test]
    const fn should_be_unpin_for_always_ready_rate_limiter() {
        implements_unpin::<AlwaysReadyRateLimiter>();
    }

    #[test]
    const fn should_implement_debug_for_always_ready_rate_limiter() {
        implements_debug::<AlwaysReadyRateLimiter>();
    }

    #[test]
    const fn should_implement_clone_for_always_ready_rate_limiter() {
        implements_clone::<AlwaysReadyRateLimiter>();
    }

    #[test]
    const fn should_implement_partial_eq_for_always_ready_rate_limiter() {
        implements_partial_eq::<AlwaysReadyRateLimiter>();
    }
}
