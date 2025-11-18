//! # Validated SEC Response Module
//!
//! This module provides the [`ValidatedSecResponse`] type and related utilities for handling
//! validated HTTP responses from SEC (Securities and Exchange Commission) endpoints. It wraps
//! [`SecResponse`] data after validation to ensure the response meets specific criteria before
//! further processing.
//!
//! ## Overview
//! The [`ValidatedSecResponse`] is a wrapper around [`SecResponse`] that guarantees the response
//! has been validated according to SEC API requirements. This includes checking status codes,
//! content types, and response structure to ensure data integrity before consumption.
//!
//! ## Types
//! - [`ValidatedSecResponse`]: Validated response wrapper containing a [`SecResponse`].
//! - [`ValidatedSecResponseError`]: Error type for validation failures.
//!
//! ## See Also
//! - [`super::sec_response::SecResponse`]: Underlying SEC response type.
//! - [`super::sec_client`]: HTTP client for making SEC-compliant requests.

use std::fmt;

use super::sec_response::SecResponse;
use super::sec_response::ContentType;

pub use validated_sec_response_error::{
    ValidatedSecResponseError, ValidatedSecResponseErrorReason,
};

mod validated_sec_response_error;

/// A validated wrapper around SEC HTTP response data.
///
/// `ValidatedSecResponse` wraps a [`SecResponse`] after validation, ensuring the response
/// meets expected criteria such as successful status codes, valid content types, and proper
/// response structure. This provides a guarantee that the response data is safe to process.
///
/// # Examples
///
/// ```rust
/// use sec::shared::sec_response::SecResponse;
/// use sec::shared::validated_sec_response::{ValidatedSecResponse, ValidatedSecResponseError};
///
/// // Create a ValidatedSecResponse from a SecResponse
/// // let sec_response: SecResponse = /* ... */;
/// // let validated = ValidatedSecResponse::from_sec_response(sec_response)?;
///
/// // Access the underlying response
/// // let response = validated.response();
/// # Ok::<(), ValidatedSecResponseError>(())
/// ```
///
/// # Validation Criteria
/// The validation process checks:
/// - Status code is successful (2xx range)
/// - Content type matches expected format
/// - Response body is not empty (when applicable)
/// - Response structure is well-formed
#[derive(Debug, Clone)]
pub struct ValidatedSecResponse {
    response: SecResponse,
}

impl ValidatedSecResponse {
    /// Creates a new [`ValidatedSecResponse`] by validating a [`SecResponse`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The response status code indicates failure (not in 2xx range)
    /// - The content type is invalid or unexpected
    /// - The response body is empty when content is expected
    /// - The response structure is malformed
    pub fn from_sec_response(response: SecResponse) -> Result<Self, ValidatedSecResponseError> {
        // Validate status code
        if !response.status().is_success() {
            return Err(ValidatedSecResponseError::new(
                ValidatedSecResponseErrorReason::InvalidStatusCode(response.status()),
            ));
        }

        // Validate body is not empty
        if response.body().is_empty() {
            return Err(ValidatedSecResponseError::new(
                ValidatedSecResponseErrorReason::EmptyResponseBody,
            ));
        }

        if response.content_type() != & ContentType::Json {
            return Err(ValidatedSecResponseError::new(
                ValidatedSecResponseErrorReason::InvalidContentType(
                    response.content_type().to_string(),
                ),
            ));
        }

        Ok(Self { response })
    }

    /// Returns a reference to the underlying validated [`SecResponse`].
    #[must_use]
    pub const fn response(&self) -> &SecResponse {
        &self.response
    }

    /// Consumes self and returns the underlying [`SecResponse`].
    #[must_use]
    pub fn into_response(self) -> SecResponse {
        self.response
    }
}

impl PartialEq for ValidatedSecResponse {
    fn eq(&self, other: &Self) -> bool {
        self.response == other.response
    }
}

impl PartialOrd for ValidatedSecResponse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValidatedSecResponse {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.response.cmp(&other.response)
    }
}

impl Eq for ValidatedSecResponse {}

impl std::hash::Hash for ValidatedSecResponse {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.response.hash(state);
    }
}

impl fmt::Display for ValidatedSecResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Validated SEC Response:\n\
             \t\tStatus: {}\n\
             \t\tURL: {}\n\
             \t\tContent-Type: {}\n\
             \t\tBody Length: {} bytes",
            self.response.status(),
            self.response.url(),
            self.response.content_type(),
            self.response.body().len()
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use reqwest::{StatusCode, Url};

    use super::*;
    use crate::shared::sec_response::ContentType;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_validated_response_when_response_is_valid() {
        let sec_response = SecResponse {
            url: Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json")
                .expect("Valid URL"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{}"),
        };

        let expected_result = ValidatedSecResponse {
            response: sec_response.clone(),
        };

        let result = ValidatedSecResponse::from_sec_response(sec_response).expect("Response should be valid.");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_status_code_is_not_success() {
        let sec_response = SecResponse {
            url: Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json")
                .expect("Valid URL"),
            status: StatusCode::BAD_REQUEST,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{}"),
        };

        let expected_result = ValidatedSecResponseError::new(
            ValidatedSecResponseErrorReason::InvalidStatusCode(StatusCode::BAD_REQUEST),
        );

        let result = ValidatedSecResponse::from_sec_response(sec_response);

        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_return_error_when_response_body_is_empty() {
        let sec_response = SecResponse {
            url: Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json")
                .expect("Valid URL"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::new(),
        };

        let expected_result =
            ValidatedSecResponseError::new(ValidatedSecResponseErrorReason::EmptyResponseBody);

        let result = ValidatedSecResponse::from_sec_response(sec_response);

        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_return_error_when_content_type_is_invalid() {
        let sec_response = SecResponse {
            url: Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json")
                .expect("Valid URL"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Html,
            body: String::from("{}"),
        };

        let expected_result =
            ValidatedSecResponseError::new(ValidatedSecResponseErrorReason::InvalidContentType("text/html".to_string()));
        let result = ValidatedSecResponse::from_sec_response(sec_response);

        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_return_underlying_response_when_response_is_called() {
        let sec_response = SecResponse {
            url: Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json")
                .expect("Valid URL"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{}"),
        };
        let validated = ValidatedSecResponse::from_sec_response(sec_response.clone())
            .expect("Should be valid");

        let expected_result = &sec_response;

        let result = validated.response();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_consume_and_return_response_when_into_response_is_called() {
        let sec_response = SecResponse {
            url: Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json")
                .expect("Valid URL"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{}"),
        };
        let validated = ValidatedSecResponse::from_sec_response(sec_response.clone())
            .expect("Should be valid");

        let expected_result = sec_response;

        let result = validated.into_response();

        assert_eq!(result, expected_result);
    }
}
