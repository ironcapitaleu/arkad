use std::fmt;

use thiserror::Error;

/// Error details for an invalid URL.
///
/// This struct provides both the reason for the failure and the offending URL string.
#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
#[error("[UrlError] Invalid URL, Reason: '{reason}', Input: '{invalid_url}'")]
pub struct UrlError {
    /// The reason why the URL is considered invalid.
    pub reason: InvalidUrlReason,
    /// The invalid URL string that was provided.
    pub invalid_url: String,
}

impl UrlError {
    /// Creates a new `UrlError`.
    pub fn new(reason: InvalidUrlReason, invalid_url: impl Into<String>) -> Self {
        Self {
            reason,
            invalid_url: invalid_url.into(),
        }
    }
}

/// Enum representing the reason for an invalid URL.
///
/// This enum is marked as non-exhaustive to allow for future extension.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidUrlReason {
    /// The input string could not be parsed as a valid URL.
    FailedToParse,
}

impl fmt::Display for InvalidUrlReason {
    /// Formats the reason for display.
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
