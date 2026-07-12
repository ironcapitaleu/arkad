//! # Rate Limiter Implementations
//!
//! Concrete limiters implementing the [`traits`](super::traits) contract.
//!
//! ## Modules
//!
//! - [`governor`]: The [`GovernorRateLimiter`], a generic leaky-bucket limiter backed by the
//!   `governor` crate.
//! - [`sec_rate_limiter`]: The [`SecRateLimiter`], the [`GovernorRateLimiter`] configured for the
//!   SEC EDGAR API's request-rate ceiling.

pub mod governor;
pub mod sec_rate_limiter;

pub use governor::GovernorRateLimiter;
pub use sec_rate_limiter::SecRateLimiter;
