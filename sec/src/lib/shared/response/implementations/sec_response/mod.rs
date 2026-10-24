//! # SEC Response
//!
//! Provides the [`SecResponse`], the concrete [`SecResponse`](crate::shared::response::SecResponse)
//! returned by the SEC API.
//!
//! ## Modules
//!
//! - [`error`]: The [`InvalidSecResponse`] error raised during validation.

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use async_trait::async_trait;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use storage::shared::Sha256Digest;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::status_code::StatusCode;
use crate::shared::url::Url;

use super::super::traits::SecResponse as SecResponseTrait;

use self::error::{ErrorReason, InvalidSecResponse};

pub mod error;

/// A validated SEC API response.
///
/// Only ever constructed once the HTTP response clears validation — a 2xx status, a JSON content
/// type, and a syntactically valid JSON body — so code holding an `SecResponse` can trust those
/// invariants. Built from a raw response via
/// [`from_inner`](crate::shared::response::SecResponse::from_inner), or from parts via
/// [`SecResponse::from_parts`].
#[derive(Debug, Clone)]
pub struct SecResponse {
    url: Url,
    headers: Headers,
    content_type: ContentType,
    status_code: StatusCode,
    body: serde_json::Value,
    raw_body: Vec<u8>,
    body_digest: Sha256Digest,
}

impl PartialEq for SecResponse {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
            && self.content_type == other.content_type
            && self.status_code == other.status_code
            && self.body_digest == other.body_digest
    }
}

impl Eq for SecResponse {}

impl std::hash::Hash for SecResponse {
    // `Headers` and `serde_json::Value` do not implement `Hash`.
    // The body is represented by the `Sha256Digest` of its raw bytes.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.content_type.hash(state);
        self.status_code.hash(state);
        self.body_digest.hash(state);
    }
}

impl PartialOrd for SecResponse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SecResponse {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.url
            .cmp(&other.url)
            .then_with(|| self.content_type.cmp(&other.content_type))
            .then_with(|| self.status_code.cmp(&other.status_code))
            .then_with(|| self.body_digest.cmp(&other.body_digest))
    }
}

impl SecResponse {
    /// Creates an [`SecResponse`] from its parts and the raw body bytes, after the same checks as
    /// the HTTP path.
    ///
    /// The HTTP path calls this function too, so a body gets the same digest on both paths.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] with:
    /// - [`ErrorReason::InvalidStatusCode`] if the status code is not in the 2xx range.
    /// - [`ErrorReason::InvalidContentType`] if the content type is not JSON.
    /// - [`ErrorReason::InvalidBody`] if the raw body is not valid JSON.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use sec::shared::content_type::ContentType;
    /// use sec::shared::headers::Headers;
    /// use sec::shared::response::implementations::sec_response::SecResponse;
    /// use sec::shared::status_code::StatusCode;
    /// use sec::shared::url::Url;
    ///
    /// let url: Url = "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
    ///     .parse()
    ///     .expect("Given a hardcoded valid URL, parsing should always succeed");
    /// let response = SecResponse::from_parts(
    ///     url,
    ///     Headers::new(HashMap::new()),
    ///     ContentType::Json,
    ///     StatusCode::Ok,
    ///     b"{ \"price\": 1.50 }".to_vec(),
    /// )
    /// .expect("Given a valid JSON body, building the response should always succeed");
    ///
    /// let expected_result = b"{ \"price\": 1.50 }";
    ///
    /// let result = response.raw_body();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn from_parts(
        url: Url,
        headers: Headers,
        content_type: ContentType,
        status_code: StatusCode,
        raw_body: Vec<u8>,
    ) -> Result<Self, InvalidSecResponse> {
        check_status_and_content_type(status_code, &content_type)?;
        let body_digest = Sha256Digest::of(&raw_body);
        let body = serde_json::from_slice(&raw_body).map_err(|e| {
            InvalidSecResponse::new(ErrorReason::InvalidBody {
                details: e.to_string(),
            })
        })?;
        Ok(Self {
            url,
            headers,
            content_type,
            status_code,
            body,
            raw_body,
            body_digest,
        })
    }

    /// Returns the body bytes exactly as the server sent them.
    #[must_use]
    pub fn raw_body(&self) -> &[u8] {
        &self.raw_body
    }

    /// Returns the SHA-256 digest of the raw body bytes.
    #[must_use]
    pub const fn body_digest(&self) -> &Sha256Digest {
        &self.body_digest
    }
}

impl Serialize for SecResponse {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("SecResponse", 4)?;
        state.serialize_field("url", &self.url.to_string())?;
        state.serialize_field("status_code", &self.status_code.to_string())?;
        state.serialize_field("content_type", &self.content_type.to_string())?;
        state.serialize_field("headers", &self.headers)?;
        state.end()
    }
}

impl Display for SecResponse {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {}", self.status_code, self.url)
    }
}

#[async_trait]
impl SecResponseTrait for SecResponse {
    type Inner = reqwest::Response;
    type Url = Url;
    type Headers = Headers;
    type StatusCode = StatusCode;
    type ContentType = ContentType;
    type Error = InvalidSecResponse;

    async fn from_inner(inner: Self::Inner) -> Result<Self, Self::Error> {
        let url = Url::from(inner.url().clone());
        let status_code = StatusCode::from(inner.status());
        let raw_headers: HashMap<String, String> = inner
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let headers = Headers::new(raw_headers);
        let content_type = headers.content_type().clone();

        // Run the checks first, so `from_inner` never reads the body of a rejected response.
        check_status_and_content_type(status_code, &content_type)?;

        let raw_body = inner.bytes().await.map_err(|e| {
            InvalidSecResponse::new(ErrorReason::FailedBodyRead {
                details: e.to_string(),
            })
        })?;

        Self::from_parts(url, headers, content_type, status_code, Vec::from(raw_body))
    }

    fn url(&self) -> &Self::Url {
        &self.url
    }

    fn headers(&self) -> &Self::Headers {
        &self.headers
    }

    fn status_code(&self) -> Self::StatusCode {
        self.status_code
    }

    fn content_type(&self) -> Self::ContentType {
        self.content_type.clone()
    }

    fn body(&self) -> &serde_json::Value {
        &self.body
    }
}

/// Checks that a response has a 2xx status code and a JSON content type.
///
/// # Errors
///
/// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidStatusCode`] or
/// [`ErrorReason::InvalidContentType`] for the first check that fails.
fn check_status_and_content_type(
    status_code: StatusCode,
    content_type: &ContentType,
) -> Result<(), InvalidSecResponse> {
    if !status_code.is_success() {
        return Err(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
            status_code,
        }));
    }
    if *content_type != ContentType::Json {
        return Err(InvalidSecResponse::new(ErrorReason::InvalidContentType {
            content_type: content_type.clone(),
        }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const RAW_BODY: &[u8] = b"{ \"price\":   1.50 }";

    fn sample_url() -> Url {
        "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn response_from_parts(raw_body: &[u8]) -> Result<SecResponse, InvalidSecResponse> {
        SecResponse::from_parts(
            sample_url(),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            raw_body.to_vec(),
        )
    }

    #[test]
    fn should_keep_the_raw_body_unchanged_when_the_body_has_extra_whitespace() {
        let expected_result = RAW_BODY;

        let response = response_from_parts(RAW_BODY)
            .expect("Given a valid JSON body, building the response should always succeed");
        let result = response.raw_body();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_digest_the_raw_body_when_building_from_parts() {
        let expected_result = Sha256Digest::of(RAW_BODY);

        let response = response_from_parts(RAW_BODY)
            .expect("Given a valid JSON body, building the response should always succeed");
        let result = response.body_digest().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_status_code_when_building_from_parts_with_a_404() {
        let expected_result = Err(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
            status_code: StatusCode::NotFound,
        }));

        let result = SecResponse::from_parts(
            sample_url(),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::NotFound,
            RAW_BODY.to_vec(),
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_content_type_when_building_from_parts_with_html() {
        let expected_result = Err(InvalidSecResponse::new(ErrorReason::InvalidContentType {
            content_type: ContentType::Html,
        }));

        let result = SecResponse::from_parts(
            sample_url(),
            Headers::new(HashMap::new()),
            ContentType::Html,
            StatusCode::Ok,
            RAW_BODY.to_vec(),
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_body_when_the_raw_body_is_not_json() {
        let details = serde_json::from_slice::<serde_json::Value>(b"not json")
            .expect_err("Given bytes that are not JSON, parsing should always fail")
            .to_string();

        let expected_result = Err(InvalidSecResponse::new(ErrorReason::InvalidBody {
            details,
        }));

        let result = response_from_parts(b"not json");

        assert_eq!(result, expected_result);
    }
}
