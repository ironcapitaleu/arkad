use std::fmt::Debug;

use async_trait::async_trait;

/// A rate limiter: paces callers so they do not exceed an allowed request rate.
///
/// Abstracts over concrete rate-limiting strategies so the domain layer is not bound to any
/// specific rate-limiting crate, and so a fake that never blocks can be substituted in tests.
#[async_trait]
pub trait RateLimiter: Send + Sync + Debug {
    /// Suspends the caller until the rate limiter permits the caller to proceed.
    ///
    /// Returns immediately when a permit is already available; otherwise it yields until the
    /// next permit becomes available, without busy-waiting.
    async fn await_turn(&self);
}
