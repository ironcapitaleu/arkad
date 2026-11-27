//! # Failed API Key Generation
//!
//! Provides [`FailedApiKeyGeneration`], the error raised when the random source of the operating
//! system fails.

use thiserror::Error;

/// Error indicating that the operating system gave no random bytes for a new API key.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error(
    "[FailedApiKeyGeneration] Failed to read 32 random bytes for an API key, Reason: '{reason}'"
)]
pub struct FailedApiKeyGeneration {
    /// The error text of the random source.
    pub reason: String,
}

impl FailedApiKeyGeneration {
    /// Creates a new [`FailedApiKeyGeneration`] error with the error text of the random source.
    #[must_use]
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_failed_api_key_generation() {
        implements_auto_traits::<FailedApiKeyGeneration>();
    }

    #[test]
    fn should_name_the_reason_when_displaying_failed_api_key_generation() {
        let error = FailedApiKeyGeneration::new("getrandom: this target is not supported");

        let expected_result = "[FailedApiKeyGeneration] Failed to read 32 random bytes for an API key, Reason: 'getrandom: this target is not supported'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
