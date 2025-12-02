//! # JSON Response Module
//!
//! This module provides the [`JsonResponse`] type and related utilities for handling
//! validated HTTP responses from SEC (Securities and Exchange Commission) endpoints. It extracts
//! and validates JSON data from [`SecResponse`] to ensure the response meets specific criteria
//! before further processing.
//!
//! ## Overview
//! The [`JsonResponse`] contains validated JSON data extracted from a [`SecResponse`].
//! It guarantees the response has been validated to contain valid JSON, including
//! checking status codes, content types, and JSON structure to ensure data integrity before
//! consumption.
//!
//! ## Types
//! - [`JsonResponse`]: Response containing parsed and validated JSON body.
//! - [`JsonResponseError`]: Error type for validation failures.
//!
//! ## See Also
//! - [`super::sec_response::SecResponse`]: Underlying SEC response type.
//! - [`super::sec_client`]: HTTP client for making SEC-compliant requests.

use std::fmt;

use serde_json;

use super::sec_response::ContentType;
use super::sec_response::SecResponse;

pub use json_response_error::{
    JsonResponseError, JsonResponseErrorReason,
};

mod json_response_error;

/// A validated SEC HTTP response containing parsed JSON data.
///
/// `JsonResponse` contains validated and parsed JSON data extracted from a
/// [`SecResponse`]. It guarantees the response meets expected criteria such as successful
/// status codes, valid JSON content type, non-empty body, and valid JSON structure.
/// This provides a guarantee that the JSON data is safe to process.
///
/// # Examples
///
/// ```rust
/// use sec::shared::sec_response::SecResponse;
/// use sec::shared::json_response::{JsonResponse, JsonResponseError};
///
/// // Create a JsonResponse from a SecResponse
/// // let sec_response: SecResponse = /* ... */;
/// // let validated = JsonResponse::from_sec_response(sec_response)?;
///
/// // Access the validated JSON body
/// // let json_body = validated.body();
/// # Ok::<(), JsonResponseError>(())
/// ```
///
/// # Validation Criteria
/// The validation process checks:
/// - Status code is successful (2xx range)
/// - Content type is JSON (`application/json`)
/// - Response body is not empty
/// - Response body contains valid JSON structure
#[derive(Debug, Clone, Default)]
pub struct JsonResponse {
    body: serde_json::Value,
}

impl JsonResponse {
    /// Creates a new [`JsonResponse`] by validating a [`SecResponse`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The response status code indicates failure (not in 2xx range)
    /// - The content type is invalid or unexpected
    /// - The response body is empty when content is expected
    /// - The response structure is malformed
    pub fn from_sec_response(response: &SecResponse) -> Result<Self, JsonResponseError> {
        // Validate status code
        if !response.status().is_success() {
            return Err(JsonResponseError::new(
                JsonResponseErrorReason::InvalidStatusCode(response.status()),
            ));
        }

        // Validate body is not empty
        if response.body().is_empty() {
            return Err(JsonResponseError::new(
                JsonResponseErrorReason::EmptyResponseBody,
            ));
        }

        // Validate that the returned content is JSON
        if response.content_type() != &ContentType::Json {
            return Err(JsonResponseError::new(
                JsonResponseErrorReason::InvalidContentType(
                    response.content_type().to_string(),
                ),
            ));
        }

        let body = serde_json::from_str(response.body()).map_err(|e| {
            JsonResponseError::new(JsonResponseErrorReason::InvalidJsonStructure(
                e.to_string(),
            ))
        })?;

        Ok(Self { body })
    }

    /// Returns a reference to the validated JSON body.
    #[must_use]
    pub const fn body(&self) -> &serde_json::Value {
        &self.body
    }

    /// Consumes self and returns the validated JSON body.
    #[must_use]
    pub fn into_body(self) -> serde_json::Value {
        self.body
    }
}

impl PartialEq for JsonResponse {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body
    }
}

impl PartialOrd for JsonResponse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JsonResponse {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare JSON values by their string representation
        self.body.to_string().cmp(&other.body.to_string())
    }
}

impl Eq for JsonResponse {}

impl std::hash::Hash for JsonResponse {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.body.to_string().hash(state);
    }
}

impl fmt::Display for JsonResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JSON Response:\n\t\tBody: {}", self.body)
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
            body: String::from("{\"key\": \"value\"}"),
        };

        let expected_result = JsonResponse {
            body: serde_json::json!({"key": "value"}),
        };

        let result = JsonResponse::from_sec_response(&sec_response)
            .expect("Response should be valid.");

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

        let expected_result = JsonResponseError::new(
            JsonResponseErrorReason::InvalidStatusCode(StatusCode::BAD_REQUEST),
        );

        let result = JsonResponse::from_sec_response(&sec_response);

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
            JsonResponseError::new(JsonResponseErrorReason::EmptyResponseBody);

        let result = JsonResponse::from_sec_response(&sec_response);

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

        let expected_result = JsonResponseError::new(
            JsonResponseErrorReason::InvalidContentType("text/html".to_string()),
        );
        let result = JsonResponse::from_sec_response(&sec_response);

        assert_eq!(result.unwrap_err(), expected_result);
    }

    #[test]
    fn should_return_error_when_json_structure_is_invalid() {
        let sec_response = SecResponse {
            url: Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json")
                .expect("Valid URL"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{invalid json}"),
        };

        let result = JsonResponse::from_sec_response(&sec_response);

        assert!(result.is_err());
        match result.unwrap_err().reason {
            JsonResponseErrorReason::InvalidJsonStructure(_) => {}
            other => panic!("Expected InvalidJsonStructure, got: {:?}", other),
        }
    }

    #[test]
    fn should_return_underlying_body_when_body_is_called() {
        let sec_response = SecResponse {
            url: Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json")
                .expect("Valid URL"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"test\": true}"),
        };
        let validated =
            JsonResponse::from_sec_response(&sec_response).expect("Should be valid");

        let expected_result = &serde_json::json!({"test": true});

        let result = validated.body();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_consume_and_return_body_when_into_body_is_called() {
        let sec_response = SecResponse {
            url: Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json")
                .expect("Valid URL"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"data\": [1, 2, 3]}"),
        };
        let validated =
            JsonResponse::from_sec_response(&sec_response).expect("Should be valid");

        let expected_result = serde_json::json!({"data": [1, 2, 3]});

        let result = validated.into_body();

        assert_eq!(result, expected_result);
    }
}
