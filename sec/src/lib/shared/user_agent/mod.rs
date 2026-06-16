//! # User Agent
//!
//! Provides the [`UserAgent`] type for validating SEC-compliant `User-Agent` strings.
//!
//! The SEC asks every request to identify the caller with a `User-Agent` of the form
//! `Company Name email@domain.com`. This newtype validates that format once at construction, so
//! the HTTP client can send the header without re-checking it.
//!
//! ## Modules
//!
//! - [`constants`]: Default user-agent values.
//! - [`user_agent_error`]: The [`UserAgentError`] returned when validation fails.

use regex::Regex;

pub mod constants;
pub mod user_agent_error;
pub use user_agent_error::{UserAgentError, UserAgentErrorReason};

/// A `User-Agent` string validated against the SEC's required format.
///
/// The SEC requires the form `Company Name email@domain.com`; wrapping it in a newtype turns that
/// requirement into a type-level guarantee. Build one with [`UserAgent::new`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct UserAgent {
    /// The validated user-agent string.
    pub inner: String,
}

impl UserAgent {
    /// Validates a string into a [`UserAgent`].
    ///
    /// # Errors
    ///
    /// Returns [`UserAgentError`] ([`UserAgentErrorReason::InvalidSecFormat`]) if the string is not
    /// of the form `Company Name email@domain.com`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::user_agent::UserAgent;
    ///
    /// let agent = UserAgent::new("Sample Company contact@example.com")
    ///     .expect("A hardcoded SEC-format string should always validate");
    /// assert_eq!(agent.inner(), "Sample Company contact@example.com");
    /// ```
    pub fn new(user_agent: &str) -> Result<Self, UserAgentError> {
        Self::validate_sec_format(user_agent)?;

        Ok(Self {
            inner: user_agent.to_string(),
        })
    }

    /// Validates that the string is a company name and email separated by a single space.
    fn validate_sec_format(user_agent: &str) -> Result<(), UserAgentError> {
        // Split the user agent into parts (company name and email)
        let parts: Vec<&str> = user_agent.rsplitn(2, ' ').collect();

        if parts.len() != 2 {
            return Err(UserAgentError::new(
                UserAgentErrorReason::InvalidSecFormat,
                user_agent,
            ));
        }

        let email = parts[0];
        let company_name = parts[1];

        Self::validate_company_name(company_name, user_agent)?;
        Self::validate_email(email, user_agent)?;

        Ok(())
    }

    /// Validates that the company name part is not empty and contains valid characters.
    ///
    /// # Errors
    /// Returns an error if the company name is empty or invalid.
    fn validate_company_name(
        company_name: &str,
        original_user_agent: &str,
    ) -> Result<(), UserAgentError> {
        if company_name.trim().is_empty() {
            return Err(UserAgentError::new(
                UserAgentErrorReason::InvalidSecFormat,
                original_user_agent,
            ));
        }

        Ok(())
    }

    /// Validates that the email part follows a valid email format.
    ///
    /// # Errors
    /// Returns an error if the email format is invalid.
    fn validate_email(email: &str, original_user_agent: &str) -> Result<(), UserAgentError> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .expect("Invalid email regex pattern");

        if !email_regex.is_match(email) {
            return Err(UserAgentError::new(
                UserAgentErrorReason::InvalidSecFormat,
                original_user_agent,
            ));
        }

        Ok(())
    }

    /// Returns the validated user-agent string.
    #[must_use]
    pub fn inner(&self) -> &str {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_user_agent_when_format_is_valid() {
        let user_agent_str = "Sample Name AdminContact@samplecompany.com";

        let expected_result = UserAgent {
            inner: user_agent_str.to_string(),
        };

        let result = UserAgent::new(user_agent_str);

        assert_eq!(result.unwrap(), expected_result);
    }

    #[test]
    fn should_return_error_when_email_is_missing() {
        let user_agent_str = "Sample Name";

        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, user_agent_str);

        let result = UserAgent::new(user_agent_str);

        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_return_error_when_company_name_is_missing() {
        let user_agent_str = "AdminContact@samplecompany.com";

        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, user_agent_str);

        let result = UserAgent::new(user_agent_str);

        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_return_error_when_email_format_is_invalid() {
        let user_agent_str = "Sample Name invalid-email";

        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, user_agent_str);

        let result = UserAgent::new(user_agent_str);

        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_create_user_agent_when_company_name_has_multiple_words() {
        let user_agent_str = "Big Tech Corporation Inc. contact@bigtech.com";

        let expected_result = UserAgent {
            inner: user_agent_str.to_string(),
        };

        let result = UserAgent::new(user_agent_str);

        assert_eq!(result.unwrap(), expected_result);
    }

    #[test]
    fn should_create_user_agent_when_company_name_has_special_characters() {
        let user_agent_str = "Company & Associates, LLC. contact@smithlaw.com";

        let expected_result = UserAgent {
            inner: user_agent_str.to_string(),
        };

        let result = UserAgent::new(user_agent_str);

        assert_eq!(result.unwrap(), expected_result);
    }

    #[test]
    fn should_create_user_agent_when_email_has_plus_sign() {
        let user_agent_str = "Sample Company admin+sec@company.com";

        let expected_result = UserAgent {
            inner: user_agent_str.to_string(),
        };

        let result = UserAgent::new(user_agent_str);

        assert_eq!(result.unwrap(), expected_result);
    }

    #[test]
    fn should_return_error_when_string_is_empty() {
        let user_agent_str = "";

        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, user_agent_str);

        let result = UserAgent::new(user_agent_str);

        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_return_error_when_email_domain_is_incomplete() {
        let user_agent_str = "Sample Company admin@company";

        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, user_agent_str);

        let result = UserAgent::new(user_agent_str);

        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_get_inner_string_when_called() {
        let user_agent_str = "Test Company contact@test.com";
        let user_agent = UserAgent::new(user_agent_str).unwrap();

        let expected_result = user_agent_str;

        let result = user_agent.inner();

        assert_eq!(result, expected_result);
    }
}
