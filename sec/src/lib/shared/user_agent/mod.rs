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
use serde::{Deserialize, Serialize};

pub mod constants;
pub mod user_agent_error;
pub use user_agent_error::{UserAgentError, UserAgentErrorReason};

/// A `User-Agent` string validated against the SEC's required format.
///
/// The SEC requires the form `Company Name email@domain.com`; wrapping it in a newtype turns that
/// requirement into a type-level guarantee. Build one with [`UserAgent::new`].
///
/// A `UserAgent` serializes as its string. Deserializing goes through [`UserAgent::new`], so a
/// user agent read from a file passes the same check.
///
/// # Examples
///
/// The field is private, so code outside this module cannot build a `UserAgent` without the check:
///
/// ```compile_fail,E0451
/// use sec::shared::user_agent::UserAgent;
///
/// let agent = UserAgent {
///     inner: String::new(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct UserAgent {
    /// The validated user-agent string.
    inner: String,
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
    ///
    /// let expected_result = "Sample Company contact@example.com";
    ///
    /// let result = agent.inner();
    ///
    /// assert_eq!(result, expected_result);
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

    /// Validates that the company name part is not empty and holds only visible ASCII characters
    /// and spaces, the characters an HTTP header value can carry.
    ///
    /// # Errors
    /// Returns an error if the company name is empty or invalid.
    fn validate_company_name(
        company_name: &str,
        original_user_agent: &str,
    ) -> Result<(), UserAgentError> {
        let has_header_characters_only = company_name
            .chars()
            .all(|character| character == ' ' || character.is_ascii_graphic());
        if company_name.trim().is_empty() || !has_header_characters_only {
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

impl TryFrom<String> for UserAgent {
    type Error = UserAgentError;

    /// Validates an owned [`String`] into a [`UserAgent`], as [`UserAgent::new`] does.
    ///
    /// # Errors
    ///
    /// Returns [`UserAgentError`] ([`UserAgentErrorReason::InvalidSecFormat`]) if the string is not
    /// of the form `Company Name email@domain.com`.
    fn try_from(user_agent: String) -> Result<Self, Self::Error> {
        Self::new(&user_agent)
    }
}

impl From<UserAgent> for String {
    /// Returns the user-agent string of a [`UserAgent`].
    fn from(user_agent: UserAgent) -> Self {
        user_agent.inner
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

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
    fn should_return_error_when_company_name_has_a_control_character() {
        let user_agent_str = "Sample\nCompany admin@samplecompany.com";

        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, user_agent_str);

        let result = UserAgent::new(user_agent_str);

        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_return_error_when_company_name_has_a_non_ascii_character() {
        let user_agent_str = "Müller GmbH admin@mueller.com";

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

    #[test]
    fn should_serialize_as_its_string_for_a_user_agent() {
        let agent = UserAgent::new("Sample Company contact@example.com")
            .expect("A hardcoded SEC-format string should always validate");

        let expected_result = serde_json::json!("Sample Company contact@example.com");

        let result = serde_json::to_value(agent)
            .expect("Given a user agent, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_same_user_agent_when_deserializing_a_serialized_user_agent() {
        let agent = UserAgent::new("Sample Company contact@example.com")
            .expect("A hardcoded SEC-format string should always validate");
        let json = serde_json::to_string(&agent)
            .expect("Given a user agent, serialization should always succeed");

        let expected_result = agent;

        let result: UserAgent = serde_json::from_str(&json)
            .expect("Given JSON this type produced, deserialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_deserialize_when_the_string_has_no_email() {
        let json = r#""Sample Company""#;

        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, "Sample Company")
                .to_string();

        let result = serde_json::from_str::<UserAgent>(json)
            .expect_err("Given a user agent without an email, deserialization should always fail")
            .to_string();

        assert!(result.starts_with(&expected_result));
    }
}
