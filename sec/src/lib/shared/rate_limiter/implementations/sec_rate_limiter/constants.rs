//! # SEC Rate Limiter Constants
//!
//! Tuning constants for pacing SEC EDGAR requests.

use std::time::Duration;

/// Minimum spacing between consecutive SEC requests.
///
/// 110 ms corresponds to ≈ 9.09 requests/second — deliberately under the SEC's 10 req/s ceiling
/// to leave headroom for clock skew and network jitter. Used as the replenish interval of the
/// leaky-bucket quota, which implies a burst size of one.
pub const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(110);
