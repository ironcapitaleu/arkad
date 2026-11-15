//! # Retry Policy
//!
//! Provides the [`RetryPolicy`] that sets how often an [`SecClient`](super::SecClient) sends one
//! request, and how long it waits between the attempts.

use std::time::Duration;

use serde::Serialize;

/// The attempts and the waits of one SEC request.
///
/// The wait before retry `n` is `first_wait * factor^(n - 1)`, at most `max_wait`. Jitter then
/// picks a wait between half of that value and the whole value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct RetryPolicy {
    /// The number of sends of one request, the first included. A value of 0 sends once.
    pub max_attempts: u32,
    /// The wait before the first retry, before jitter.
    pub first_wait: Duration,
    /// The factor by which each wait grows.
    pub factor: u32,
    /// The longest wait before jitter.
    pub max_wait: Duration,
}

impl Default for RetryPolicy {
    /// Returns 3 attempts, with waits of 1 s and 2 s before jitter, and a cap of 4 s.
    fn default() -> Self {
        Self {
            max_attempts: 3,
            first_wait: Duration::from_secs(1),
            factor: 2,
            max_wait: Duration::from_secs(4),
        }
    }
}

impl RetryPolicy {
    /// The policy that sends a request once and never retries.
    pub const SEND_ONCE: Self = Self {
        max_attempts: 1,
        first_wait: Duration::ZERO,
        factor: 1,
        max_wait: Duration::ZERO,
    };

    /// Returns the wait before retry `retry`, counted from 1, before jitter.
    #[must_use]
    pub fn backoff(&self, retry: u32) -> Duration {
        let growth = self.factor.saturating_pow(retry.saturating_sub(1));
        self.first_wait.saturating_mul(growth).min(self.max_wait)
    }

    /// Returns a wait between half of `backoff` and the whole `backoff`, where `fraction` in
    /// `[0, 1]` picks the point. A `fraction` of NaN counts as 0.
    #[must_use]
    pub fn jitter(backoff: Duration, fraction: f64) -> Duration {
        let fraction = if fraction.is_nan() {
            0.0
        } else {
            fraction.clamp(0.0, 1.0)
        };
        let half = backoff / 2;
        half + half.mul_f64(fraction)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use pretty_assertions::assert_eq;

    use super::RetryPolicy;

    #[test]
    fn should_double_the_wait_for_each_retry_of_the_default_policy() {
        let policy = RetryPolicy::default();

        let expected_result = vec![
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(4),
        ];

        let result: Vec<Duration> = (1..=3).map(|retry| policy.backoff(retry)).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cap_the_wait_when_the_backoff_passes_the_max_wait() {
        let policy = RetryPolicy::default();

        let expected_result = Duration::from_secs(4);

        let result = policy.backoff(10);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cap_the_wait_when_the_growth_overflows() {
        let policy = RetryPolicy::default();

        let expected_result = Duration::from_secs(4);

        let result = policy.backoff(u32::MAX);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_wait_half_the_backoff_when_the_jitter_fraction_is_zero() {
        let backoff = Duration::from_secs(2);

        let expected_result = Duration::from_secs(1);

        let result = RetryPolicy::jitter(backoff, 0.0);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_wait_the_whole_backoff_when_the_jitter_fraction_is_one() {
        let backoff = Duration::from_secs(2);

        let expected_result = Duration::from_secs(2);

        let result = RetryPolicy::jitter(backoff, 1.0);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_wait_within_the_backoff_when_the_jitter_fraction_is_out_of_range() {
        let backoff = Duration::from_secs(2);

        let expected_result = Duration::from_secs(2);

        let result = RetryPolicy::jitter(backoff, 7.5);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_wait_half_the_backoff_when_the_jitter_fraction_is_nan() {
        let backoff = Duration::from_secs(2);

        let expected_result = Duration::from_secs(1);

        let result = RetryPolicy::jitter(backoff, f64::NAN);

        assert_eq!(result, expected_result);
    }
}
