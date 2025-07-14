use thiserror::Error;

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
}

impl std::fmt::Display for SecClientErrorReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReqwestClientCreationFailed => write!(
                f,
                "Reqwest client could not be created due to an invalid configuration."
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_format_display_as_expected_when_reason_is_reqwest_failure() {
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
