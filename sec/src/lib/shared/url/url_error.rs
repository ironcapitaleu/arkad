//! # URL Errors
//!
//! Provides the [`UrlError`] returned when a string fails URL validation, and the
//! [`InvalidUrlReason`] describing why.

use std::fmt;

use thiserror::Error;

/// Reports that a string could not be validated as a URL.
///
/// Carries the [`InvalidUrlReason`] and the offending input for diagnostics.
#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
#[error("[UrlError] Invalid URL, Reason: '{reason}', Input: '{invalid_url}'")]
pub struct UrlError {
    /// Why the URL is considered invalid.
    pub reason: InvalidUrlReason,
    /// The original string that failed validation.
    pub invalid_url: String,
}

impl UrlError {
    /// Creates a new error from a reason and the offending input.
    pub fn new(reason: InvalidUrlReason, invalid_url: impl Into<String>) -> Self {
        Self {
            reason,
            invalid_url: invalid_url.into(),
        }
    }
}

/// Why a string failed URL validation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidUrlReason {
    /// The input could not be parsed as a URL.
    FailedToParse,
}

impl fmt::Display for InvalidUrlReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FailedToParse => write!(f, "Input could not be parsed as a valid URL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_format_display_as_expected_when_reason_is_failed_to_parse() {
        let reason = InvalidUrlReason::FailedToParse;
        let invalid_url = "not a url";
        let error = UrlError::new(reason.clone(), invalid_url);

        let expected_result =
            format!("[UrlError] Invalid URL, Reason: '{reason}', Input: '{invalid_url}'");

        let result = format!("{error}");

        assert_eq!(result, expected_result);
    }
}
