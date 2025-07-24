//! # SEC Client Error Types
//!
//! This module defines error types and reasons for SEC client creation failures.
//! It is used throughout the [`crate::shared::sec_client`] module and by state machine implementations
//! that require robust error reporting for client initialization failures.
//!
//! ## Types
//! - [`SecClientError`]: Error struct containing the [`SecClientErrorReason`] and the user agent string that caused the failure. This allows precise diagnostics about why a client couldn't be created.
//! - [`SecClientErrorReason`]: Enum describing specific reasons for client creation failure, such as reqwest client creation issues or invalid user agent strings.
//!
//! ## Usage
//! These error types are returned by SEC client creation routines and are used in state data modules
//! to provide detailed diagnostics and error handling for HTTP client initialization.
//! They are also used as domain errors for the general state machine error logic in [`crate::error`] and may be wrapped by state-level errors.
//!
//! ## See Also
//! - [`crate::shared::sec_client`]: Main SEC client utilities module.
//! - [`crate::error`]: Error types that may reference SEC client errors for reporting.

use thiserror::Error;

/// Error details for SEC client creation failures.
///
/// This struct provides both the reason for the failure and the user agent string that was provided.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[SecClientError] Client creation failed: Reason: '{reason}'. Input: '{user_agent}'.")]
pub struct SecClientError {
    /// The reason why the client couldn't be created.
    pub reason: SecClientErrorReason,
    /// The user agent string that was provided.
    pub user_agent: String,
}

impl SecClientError {
    /// Creates a new `SecClientError`.
    pub fn new(reason: SecClientErrorReason, user_agent: impl Into<String>) -> Self {
        Self {
            reason,
            user_agent: user_agent.into(),
        }
    }
}

/// Enum representing the reason for a client creation failure.
///
/// This enum is marked as non-exhaustive to allow for future extension.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecClientErrorReason {
    /// The reqwest client could not be created.
    ReqwestClientCreationFailed,
    /// The user agent string is invalid.
    InvalidUserAgent,
}

impl std::fmt::Display for SecClientErrorReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReqwestClientCreationFailed => write!(
                f,
                "Reqwest client could not be created due to an invalid configuration."
            ),
            Self::InvalidUserAgent => write!(f, "The user agent string is invalid."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_format_display_as_expected_when_reason_is_present() {
        let user_agent = "TestUserAgent".to_string();
        let reason = SecClientErrorReason::ReqwestClientCreationFailed;
        let client_error = SecClientError::new(reason.clone(), &user_agent);

        let expected_result = format!(
            "[SecClientError] Client creation failed: Reason: '{reason}'. Input: '{user_agent}'."
        );

        let result = format!("{client_error}");

        assert_eq!(result, expected_result);
    }
}
