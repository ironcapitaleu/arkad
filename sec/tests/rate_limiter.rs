//! # Rate Limiter Integration Tests
//!
//! Wall-clock timing tests for the rate limiters. These assert real pacing behavior by measuring
//! elapsed time, so they live here rather than in the unit tests (which stay pure and instant).
//!
//! Each assertion is a **lower bound** (`elapsed >= minimum`): a leaky-bucket limiter can never
//! release permits *faster* than its configured rate, so these cannot fail spuriously under load
//! and run as part of the normal suite.

use std::time::{Duration, Instant};

use sec::shared::rate_limiter::implementations::sec_rate_limiter::constants::MIN_REQUEST_INTERVAL;
use sec::shared::rate_limiter::{GovernorRateLimiter, RateLimiter, SecRateLimiter};

#[tokio::test]
async fn should_pace_callers_to_configured_period_when_issuing_multiple_in_a_row() {
    let period = Duration::from_millis(50);
    let limiter = GovernorRateLimiter::new(period);
    let request_count = 3;

    let start = Instant::now();
    for _ in 0..request_count {
        limiter.await_turn().await;
    }
    let elapsed = start.elapsed();

    // The first caller passes immediately (burst = 1); each subsequent caller waits one period,
    // so `request_count` callers take at least `count - 1` periods.
    let expected_minimum = period * (request_count - 1);

    assert!(
        elapsed >= expected_minimum,
        "Expected at least {expected_minimum:?} to elapse for {request_count} callers, but only {elapsed:?} elapsed"
    );
}

#[tokio::test]
async fn should_pace_callers_to_sec_ceiling_when_issuing_multiple_in_a_row() {
    let limiter = SecRateLimiter::new();
    let request_count = 3;

    let start = Instant::now();
    for _ in 0..request_count {
        limiter.await_turn().await;
    }
    let elapsed = start.elapsed();

    let expected_minimum = MIN_REQUEST_INTERVAL * (request_count - 1);

    assert!(
        elapsed >= expected_minimum,
        "Expected at least {expected_minimum:?} to elapse for {request_count} callers, but only {elapsed:?} elapsed"
    );
}

#[tokio::test]
async fn should_space_second_permit_by_sec_interval_when_issued_back_to_back() {
    let limiter = SecRateLimiter::new();

    let start = Instant::now();
    limiter.await_turn().await;
    limiter.await_turn().await;
    let elapsed = start.elapsed();

    // Two back-to-back permits must be spaced at least MIN_REQUEST_INTERVAL apart, proving the SEC
    // ceiling (rather than an arbitrary faster rate) is what this limiter enforces.
    let expected_minimum = MIN_REQUEST_INTERVAL;

    assert!(
        elapsed >= expected_minimum,
        "Expected the second permit to wait at least {expected_minimum:?}, but only {elapsed:?} elapsed"
    );
}
