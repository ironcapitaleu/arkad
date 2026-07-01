//! # Header Errors
//!
//! Provides the [`HeadersError`] reported when response headers fail validation, and the
//! [`InvalidHeadersReason`] describing why.

use std::collections::HashMap;
use std::fmt;

use thiserror::Error;

/// Reports that a set of response headers failed validation.
///
/// Carries the [`InvalidHeadersReason`] and the raw headers as received, preserved for diagnostics.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("[HeadersError] Header validation failed, Reason: '{reason}'")]
pub struct HeadersError {
    /// Why header validation failed.
    pub reason: InvalidHeadersReason,
    /// The raw headers as received, preserved for diagnostics.
    pub raw_headers: HashMap<String, String>,
}

impl HeadersError {
    /// Creates a new error from a reason and the raw headers that failed.
    #[must_use]
    pub const fn new(reason: InvalidHeadersReason, raw_headers: HashMap<String, String>) -> Self {
        Self {
            reason,
            raw_headers,
        }
    }
}

/// Why a set of response headers failed validation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidHeadersReason {
    /// A required header is absent from the response.
    MissingHeader {
        /// The name of the missing header.
        header_name: String,
    },
    /// A header is present but its value could not be parsed.
    InvalidHeader {
        /// The name of the offending header.
        header_name: String,
        /// The value that could not be parsed.
        value: String,
    },
}

impl fmt::Display for InvalidHeadersReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
            format!("[HeadersError] Header validation failed, Reason: '{reason}'");

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
            format!("[HeadersError] Header validation failed, Reason: '{reason}'");

        let result = format!("{error}");

        assert_eq!(result, expected_result);
    }
}
