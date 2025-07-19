use thiserror::Error;

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
        // Arrange
        let user_agent = "TestUserAgent".to_string();
        let reason = UserAgentErrorReason::InvalidSecFormat;
        let user_agent_error = UserAgentError::new(reason.clone(), &user_agent);

        // Define
        let expected_result = format!(
            "[UserAgentError] User agent creation failed: Reason: '{reason}'. Input: '{user_agent}'."
        );

        // Act
        let result = format!("{user_agent_error}");

        // Assert
        assert_eq!(result, expected_result);
    }
}
