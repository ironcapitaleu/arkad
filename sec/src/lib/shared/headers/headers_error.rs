use std::collections::HashMap;
use std::fmt;

use thiserror::Error;

/// Error details for HTTP header validation failures.
///
/// This struct provides the reason for the failure and the raw headers
/// that were provided, preserved for diagnostics.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("[HeadersError] Header validation failed: Reason: '{reason}'.")]
pub struct HeadersError {
    /// The reason why header validation failed.
    pub reason: InvalidHeadersReason,
    /// The raw headers that were provided, preserved for diagnostics.
    pub raw_headers: HashMap<String, String>,
}

impl HeadersError {
    /// Creates a new `HeadersError`.
    #[must_use]
    pub const fn new(reason: InvalidHeadersReason, raw_headers: HashMap<String, String>) -> Self {
        Self {
            reason,
            raw_headers,
        }
    }
}

/// Enum representing the reason for a header validation failure.
///
/// This enum is marked as non-exhaustive to allow for future extension.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidHeadersReason {
    /// A required header is missing from the response.
    MissingHeader { header_name: String },
    /// A header value is present but cannot be parsed or is invalid.
    InvalidHeader { header_name: String, value: String },
}

impl fmt::Display for InvalidHeadersReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHeader { header_name } => {
                write!(f, "Required header '{header_name}' is missing")
            }
            Self::InvalidHeader { header_name, value } => {
                write!(f, "Header '{header_name}' has an invalid value: '{value}'")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_format_display_as_expected_when_reason_is_missing_header() {
        let reason = InvalidHeadersReason::MissingHeader {
            header_name: "etag".to_string(),
        };
        let raw_headers = HashMap::new();
        let error = HeadersError::new(reason.clone(), raw_headers);

        let expected_result =
            format!("[HeadersError] Header validation failed: Reason: '{reason}'.");

        let result = format!("{error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_reason_is_invalid_header() {
        let reason = InvalidHeadersReason::InvalidHeader {
            header_name: "content-type".to_string(),
            value: "invalid".to_string(),
        };
        let raw_headers = HashMap::new();
        let error = HeadersError::new(reason.clone(), raw_headers);

        let expected_result =
            format!("[HeadersError] Header validation failed: Reason: '{reason}'.");

        let result = format!("{error}");

        assert_eq!(result, expected_result);
    }
}
