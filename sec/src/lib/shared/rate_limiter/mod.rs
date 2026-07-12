//! # Rate Limiter
//!
//! Provides rate limiters for pacing callers to an allowed request rate.
//!
//! The limiter is split into a trait layer so the library is not bound to any one rate-limiting
//! crate: a [`RateLimiter`] contract, a generic [`GovernorRateLimiter`] backed by the `governor`
//! crate, and a [`SecRateLimiter`] that configures the generic limiter for the SEC EDGAR API's
//! request-rate ceiling. This layering keeps the pacing mechanism reusable while confining the
//! SEC-specific policy to one type, and lets tests substitute a fake that never blocks.
//!
//! ## Modules
//!
//! - [`traits`]: The [`RateLimiter`] contract.
//! - [`implementations`]: The generic [`GovernorRateLimiter`] and the SEC-configured
//!   [`SecRateLimiter`].
//!
//! ## See Also
//!
//! - [`crate::shared::http_client`]: Uses a [`SecRateLimiter`] to pace outgoing requests.

pub mod implementations;
pub mod traits;

pub use implementations::{GovernorRateLimiter, SecRateLimiter};
pub use traits::RateLimiter;
