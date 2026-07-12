//! # Rate Limiter Traits
//!
//! The rate-limiting contract abstracting how callers are paced to an allowed request rate.
//!
//! ## Modules
//!
//! - [`rate_limiter`]: The [`RateLimiter`] trait abstracting a concrete rate-limiting strategy.

pub mod rate_limiter;

pub use rate_limiter::RateLimiter;
