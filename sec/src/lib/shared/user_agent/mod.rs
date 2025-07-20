//! # User Agent Utilities
//!
//! This module provides the [`UserAgent`] type and related utilities for creating and validating
//! SEC-compliant user agent strings. It is used throughout the SEC state machine library to ensure that
//! HTTP requests to SEC API endpoints include properly formatted user agent headers.
//!
//! ## Modules
//! - [`user_agent_error`]: Error types and reasons for invalid user agent strings.
//!
//! ## Types
//! - [`UserAgent`]: Strongly-typed wrapper for a validated user agent string that complies with SEC format requirements.
//! - [`UserAgentError`], [`UserAgentErrorReason`]: Error types for reporting user agent validation failures.
//!
//! ## Usage
//! The [`UserAgent`] type is used by HTTP client implementations and state machine logic to ensure that
//! all requests to SEC API endpoints include properly formatted user agent strings. The SEC requires
//! user agent strings to follow the format "Company Name email@domain.com".
//!
//! ## See Also
//! - [`crate::shared`]: Shared domain types and utilities used across the SEC state machine library.
//! - [`crate::shared::sec_client`]: SEC client utilities that use user agent strings for HTTP requests.
//! - [`crate::error`]: Error types that may reference [`UserAgentError`] and [`UserAgentErrorReason`] for detailed diagnostics.

use regex::Regex;

pub mod user_agent_error;
pub use user_agent_error::{UserAgentError, UserAgentErrorReason};

/// Strongly-typed wrapper for a validated SEC-compliant user agent string.
///
/// The `UserAgent` type ensures that only valid, SEC-compliant user agent strings are constructed and used
/// throughout the SEC state machine library. The SEC requires user agent strings to follow the format
/// "Company Name email@domain.com". Use [`UserAgent::new`] to construct and validate a user agent value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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
