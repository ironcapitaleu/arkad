//! # User Agent Error Types
//!
//! This module defines error types and reasons for user agent validation failures.
//! It is used throughout the [`crate::shared::user_agent`] module and by state machine implementations
//! that require robust error reporting for user agent string validation failures.
//!
//! ## Types
//! - [`UserAgentError`]: Error struct containing the [`UserAgentErrorReason`] and the user agent string that caused the failure. This allows precise diagnostics about why a user agent string is invalid.
//! - [`UserAgentErrorReason`]: Enum describing specific reasons for user agent validation failure, such as not following the required SEC format.
//!
//! ## Usage
//! These error types are returned by user agent validation routines and are used in state data modules
//! to provide detailed diagnostics and error handling for SEC-compliant user agent string validation.
//! They are also used as domain errors for the general state machine error logic in [`crate::error`] and may be wrapped by state-level errors.
//!
//! ## See Also
//! - [`crate::shared::user_agent`]: Main user agent utilities module.
//! - [`crate::error`]: Error types that may reference user agent errors for reporting.

use thiserror::Error;

/// Error details for user agent validation failures.
///
/// This struct provides both the reason for the failure and the user agent string that was provided.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[UserAgentError] User agent creation failed: Reason: '{reason}'. Input: '{user_agent}'.")]
pub struct UserAgentError {
    /// The reason why the user agent couldn't be created.
    pub reason: UserAgentErrorReason,
    /// The user agent string that was provided.
    pub user_agent: String,
}

impl UserAgentError {
    /// Creates a new `UserAgentError`.
    pub fn new(reason: UserAgentErrorReason, user_agent: impl Into<String>) -> Self {
        Self {
            reason,
            user_agent: user_agent.into(),
        }
    }
}

/// Enum representing the reason for a user agent creation failure.
///
/// This enum is marked as non-exhaustive to allow for future extension.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UserAgentErrorReason {
    /// The format required for the SEC api is invalid.
    InvalidSecFormat,
}

impl std::fmt::Display for UserAgentErrorReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSecFormat => write!(f, "The format required for the SEC api is invalid."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_format_display_as_expected_when_reason_is_invalid_sec_format() {
        let user_agent = "TestUserAgent".to_string();
        let reason = UserAgentErrorReason::InvalidSecFormat;
        let user_agent_error = UserAgentError::new(reason.clone(), &user_agent);

        let expected_result = format!(
            "[UserAgentError] User agent creation failed: Reason: '{reason}'. Input: '{user_agent}'."
        );

        let result = format!("{user_agent_error}");

        assert_eq!(result, expected_result);
    }
}
