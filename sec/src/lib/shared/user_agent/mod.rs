use regex::Regex;

pub mod user_agent_error;
pub use user_agent_error::{UserAgentError, UserAgentErrorReason};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserAgent {
    pub inner: String,
}

impl UserAgent {
    /// Creates a new `UserAgent`.
    ///
    /// The user agent must comply with SEC format requirements.
    ///
    /// # Errors
    /// Returns an error if the user agent string doesn't match the SEC format.
    pub fn new(user_agent: &str) -> Result<Self, UserAgentError> {
        Self::validate_sec_format(user_agent)?;

        Ok(Self {
            inner: user_agent.to_string(),
        })
    }

    /// Validates that the user agent string complies with SEC format.
    ///
    /// The SEC format requires: "Sample Name contact@domain.com"
    /// - Sample Name: one or more words (letters, numbers, spaces, hyphens, periods)
    /// - Single space separator
    /// - Valid email address
    fn validate_sec_format(user_agent: &str) -> Result<(), UserAgentError> {
        // Regex pattern for SEC user agent format:
        // ^(.+?)\s+([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})$
        // - (.+?) - Company name (non-greedy match for any characters)
        // - \s+ - One or more spaces
        // - ([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}) - Email address
        let regex = Regex::new(r"^(.+?)\s+([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})$")
            .expect("Invalid regex pattern");

        if !regex.is_match(user_agent) {
            return Err(UserAgentError::new(
                UserAgentErrorReason::InvalidSecFormat,
                user_agent,
            ));
        }

        Ok(())
    }

    #[must_use]
    pub fn inner(&self) -> &str {
        &self.inner
    }
}

impl Default for UserAgent {
    fn default() -> Self {
        Self {
            inner: "Default Company contact@example.com".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_user_agent_when_format_is_valid() {
        // Arrange
        let user_agent_str = "Sample Name AdminContact@samplecompany.com";

        // Define
        let expected_result = UserAgent {
            inner: user_agent_str.to_string(),
        };

        // Act
        let result = UserAgent::new(user_agent_str);

        // Assert
        assert_eq!(result.unwrap(), expected_result);
    }

    #[test]
    fn should_return_error_when_email_is_missing() {
        // Arrange
        let user_agent_str = "Sample Name";

        // Define
        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, user_agent_str);

        // Act
        let result = UserAgent::new(user_agent_str);

        // Assert
        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_return_error_when_company_name_is_missing() {
        // Arrange
        let user_agent_str = "AdminContact@samplecompany.com";

        // Define
        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, user_agent_str);

        // Act
        let result = UserAgent::new(user_agent_str);

        // Assert
        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_return_error_when_email_format_is_invalid() {
        // Arrange
        let user_agent_str = "Sample Name invalid-email";

        // Define
        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, user_agent_str);

        // Act
        let result = UserAgent::new(user_agent_str);

        // Assert
        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_create_user_agent_when_company_name_has_multiple_words() {
        // Arrange
        let user_agent_str = "Big Tech Corporation Inc. contact@bigtech.com";

        // Define
        let expected_result = UserAgent {
            inner: user_agent_str.to_string(),
        };

        // Act
        let result = UserAgent::new(user_agent_str);

        // Assert
        assert_eq!(result.unwrap(), expected_result);
    }

    #[test]
    fn should_create_user_agent_when_company_name_has_special_characters() {
        // Arrange
        let user_agent_str = "Company & Associates, LLC. contact@smithlaw.com";

        // Define
        let expected_result = UserAgent {
            inner: user_agent_str.to_string(),
        };

        // Act
        let result = UserAgent::new(user_agent_str);

        // Assert
        assert_eq!(result.unwrap(), expected_result);
    }

    #[test]
    fn should_create_user_agent_when_email_has_plus_sign() {
        // Arrange
        let user_agent_str = "Sample Company admin+sec@company.com";

        // Define
        let expected_result = UserAgent {
            inner: user_agent_str.to_string(),
        };

        // Act
        let result = UserAgent::new(user_agent_str);

        // Assert
        assert_eq!(result.unwrap(), expected_result);
    }

    #[test]
    fn should_return_error_when_string_is_empty() {
        // Arrange
        let user_agent_str = "";

        // Define
        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, user_agent_str);

        // Act
        let result = UserAgent::new(user_agent_str);

        // Assert
        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_return_error_when_email_domain_is_incomplete() {
        // Arrange
        let user_agent_str = "Sample Company admin@company";

        // Define
        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, user_agent_str);

        // Act
        let result = UserAgent::new(user_agent_str);

        // Assert
        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_get_inner_string_when_called() {
        // Arrange
        let user_agent_str = "Test Company contact@test.com";
        let user_agent = UserAgent::new(user_agent_str).unwrap();

        // Define
        let expected_result = user_agent_str;

        // Act
        let result = user_agent.inner();

        // Assert
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_default_user_agent_when_default_is_called() {
        // Arrange
        // (No setup needed)

        // Define
        let expected_result = UserAgent {
            inner: "Default Company contact@example.com".to_string(),
        };

        // Act
        let result = UserAgent::default();

        // Assert
        assert_eq!(result, expected_result);
    }
}
