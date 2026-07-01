//! # User Agent Errors
//!
//! Provides the [`UserAgentError`] returned when a string fails user-agent validation, and the
//! [`UserAgentErrorReason`] describing why.

use std::fmt::{self, Display, Formatter};

use thiserror::Error;

/// Reports that a string could not be validated as an SEC user agent.
///
/// Carries the [`UserAgentErrorReason`] and the offending input for diagnostics.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[UserAgentError] User agent creation failed, Reason: '{reason}', Input: '{user_agent}'")]
pub struct UserAgentError {
    /// Why the user agent is considered invalid.
    pub reason: UserAgentErrorReason,
    /// The original string that failed validation.
    pub user_agent: String,
}

impl UserAgentError {
    /// Creates a new error from a reason and the offending input.
    pub fn new(reason: UserAgentErrorReason, user_agent: impl Into<String>) -> Self {
        Self {
            reason,
            user_agent: user_agent.into(),
        }
    }
}

/// Why a string failed user-agent validation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UserAgentErrorReason {
    /// The string does not match the SEC's required `Company Name email@domain.com` format.
    InvalidSecFormat,
}

impl Display for UserAgentErrorReason {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InvalidSecFormat => {
                write!(f, "The format required for the SEC API is invalid")
            }
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
            "[UserAgentError] User agent creation failed, Reason: '{reason}', Input: '{user_agent}'"
        );

        let result = format!("{user_agent_error}");

        assert_eq!(result, expected_result);
    }
}
