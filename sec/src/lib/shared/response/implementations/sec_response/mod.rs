//! # SEC Response
//!
//! Provides the [`SecResponse`], the concrete [`SecResponse`](crate::shared::response::SecResponse)
//! returned by the SEC API, generic over its body kind.
//!
//! ## Modules
//!
//! - [`body`]: The [`ResponseBody`] trait and the [`JsonBody`] and [`ZipBody`] kinds.
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

pub mod body;
pub mod error;

pub use body::{JsonBody, ResponseBody, ZipBody};

/// A validated SEC API response with a body of kind `B`.
///
/// Only ever constructed once the HTTP response clears the checks of `B`, so code holding a
/// `SecResponse` can trust them:
///
/// - [`JsonBody`], the default: a 2xx status, a JSON content type, and a valid JSON body.
/// - [`ZipBody`]: status 200, a ZIP or octet-stream content type, and a body that matches the
///   `Content-Length` header.
///
/// Built from a raw response via [`from_inner`](crate::shared::response::SecResponse::from_inner),
/// or from parts via [`SecResponse::from_parts`]. Equality, ordering, and hashing use the URL,
/// the content type, the status code, and the SHA-256 digest of the body.
#[derive(Debug, Clone)]
pub struct SecResponse<B = JsonBody> {
    url: Url,
    headers: Headers,
    content_type: ContentType,
    status_code: StatusCode,
    body: B,
    body_digest: Sha256Digest,
}

impl<B> PartialEq for SecResponse<B> {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
            && self.content_type == other.content_type
            && self.status_code == other.status_code
            && self.body_digest == other.body_digest
    }
}

impl<B> Eq for SecResponse<B> {}

impl<B> std::hash::Hash for SecResponse<B> {
    // `Headers` and the body do not implement `Hash`.
    // The body is represented by the `Sha256Digest` of its raw bytes.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.content_type.hash(state);
        self.status_code.hash(state);
        self.body_digest.hash(state);
    }
}

impl<B> PartialOrd for SecResponse<B> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<B> Ord for SecResponse<B> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.url
            .cmp(&other.url)
            .then_with(|| self.content_type.cmp(&other.content_type))
            .then_with(|| self.status_code.cmp(&other.status_code))
            .then_with(|| self.body_digest.cmp(&other.body_digest))
    }
}

impl<B: ResponseBody> SecResponse<B> {
    /// Creates a [`SecResponse`] from its parts and the raw body bytes, after the same checks as
    /// the HTTP path.
    ///
    /// The HTTP path runs the same checks, so a body gets the same digest on both paths.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] if a check of `B` fails:
    /// - [`ErrorReason::InvalidStatusCode`] if `B` does not accept the status code.
    /// - [`ErrorReason::InvalidContentType`] if `B` does not accept the content type.
    /// - [`ErrorReason::InvalidBody`] if the raw body of a [`JsonBody`] is not valid JSON.
    /// - [`ErrorReason::InvalidContentLength`] if the raw body of a [`ZipBody`] does not match
    ///   the `Content-Length` header.
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
    /// let response: SecResponse = SecResponse::from_parts(
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
        B::check_status_and_content_type(status_code, &content_type)?;
        let (body, body_digest) = B::from_raw(&headers, raw_body)?;
        Ok(Self {
            url,
            headers,
            content_type,
            status_code,
            body,
            body_digest,
        })
    }

    /// Builds the response from a body that was read from the network.
    ///
    /// The caller has checked the status code and the content type.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] if the raw body fails a check of `B`.
    async fn from_read_body(
        url: Url,
        headers: Headers,
        content_type: ContentType,
        status_code: StatusCode,
        raw_body: Vec<u8>,
    ) -> Result<Self, InvalidSecResponse> {
        let (body, body_digest) = B::from_read(&headers, raw_body).await?;
        Ok(Self {
            url,
            headers,
            content_type,
            status_code,
            body,
            body_digest,
        })
    }
}

impl<B> SecResponse<B> {
    /// Returns the SHA-256 digest of the raw body bytes.
    #[must_use]
    pub const fn body_digest(&self) -> &Sha256Digest {
        &self.body_digest
    }

    /// Replaces the body digest, so a test can tell a stored digest apart from a new hash.
    #[cfg(test)]
    pub(crate) fn with_body_digest(mut self, body_digest: Sha256Digest) -> Self {
        self.body_digest = body_digest;
        self
    }
}

impl SecResponse<JsonBody> {
    /// Returns the body bytes exactly as the server sent them.
    #[must_use]
    pub fn raw_body(&self) -> &[u8] {
        self.body.raw()
    }
}

impl SecResponse<ZipBody> {
    /// Consumes the response and returns the body bytes without a copy.
    #[must_use]
    pub fn into_body(self) -> Vec<u8> {
        self.body.into_bytes()
    }
}

/// Serializes the URL, the status code, the content type, the headers, and the SHA-256 of the
/// body, without the body bytes.
impl<B> Serialize for SecResponse<B> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("SecResponse", 5)?;
        state.serialize_field("url", &self.url.to_string())?;
        state.serialize_field("status_code", &self.status_code.to_string())?;
        state.serialize_field("content_type", &self.content_type.to_string())?;
        state.serialize_field("headers", &self.headers)?;
        state.serialize_field("body_sha256", &self.body_digest)?;
        state.end()
    }
}

impl<B> Display for SecResponse<B> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {}", self.status_code, self.url)
    }
}

#[async_trait]
impl<B: ResponseBody> SecResponseTrait for SecResponse<B> {
    type Inner = reqwest::Response;
    type Url = Url;
    type Headers = Headers;
    type StatusCode = StatusCode;
    type ContentType = ContentType;
    type Body = B::Content;
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
        B::check_status_and_content_type(status_code, &content_type)?;

        let raw_body = inner.bytes().await.map_err(|e| {
            InvalidSecResponse::new(ErrorReason::FailedBodyRead {
                details: e.to_string(),
            })
        })?;

        Self::from_read_body(url, headers, content_type, status_code, Vec::from(raw_body)).await
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

    fn body(&self) -> &Self::Body {
        self.body.content()
    }
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

        let result = SecResponse::<JsonBody>::from_parts(
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

        let result = SecResponse::<JsonBody>::from_parts(
            sample_url(),
            Headers::new(HashMap::new()),
            ContentType::Html,
            StatusCode::Ok,
            RAW_BODY.to_vec(),
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_body_digest_but_not_the_body_for_a_json_response() {
        let response = response_from_parts(RAW_BODY)
            .expect("Given a valid JSON body, building the response should always succeed");

        let expected_result = vec![
            "body_sha256",
            "content_type",
            "headers",
            "status_code",
            "url",
        ];

        let value = serde_json::to_value(&response).expect(
            "Given a response of strings and a digest, serialization should always succeed",
        );
        let result: Vec<&str> = value
            .as_object()
            .map(|fields| fields.keys().map(String::as_str).collect())
            .unwrap_or_default();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_parsed_json_body_when_building_from_a_raw_response() {
        let inner = http::Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(RAW_BODY.to_vec())
            .expect("Given a valid status code and header names, building the response should always succeed");

        let expected_result = serde_json::json!({ "price": 1.50 });

        let response = SecResponse::<JsonBody>::from_inner(reqwest::Response::from(inner))
            .await
            .expect("Given status 200 and a valid JSON body, building the response should always succeed");
        let result = response.body().clone();

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

#[cfg(test)]
mod zip_tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    use pretty_assertions::assert_eq;

    use super::*;

    const BODY: &[u8] = b"PK\x03\x04 notes";

    /// The size in bytes of a body whose hash takes long enough for the ticker to tick many times.
    ///
    /// The same value as in `storage/src/tests/fixtures/ticker`.
    const LARGE_BODY_SIZE: usize = 8 * 1024 * 1024;

    /// Builds a raw `reqwest` response with status 200, a ZIP content type, and the body.
    fn zip_inner_response(body: Vec<u8>) -> reqwest::Response {
        let response = http::Response::builder()
            .status(200)
            .header("content-type", "application/zip")
            .body(body)
            .expect("Given a valid status code and header names, building the response should always succeed");
        reqwest::Response::from(response)
    }

    /// Yields to the task until `is_done` is `true`, and returns the number of times it ran.
    ///
    /// Joined with a future on one task, it ticks only while that future waits. A future that blocks
    /// the task to its end lets it tick at most once.
    ///
    /// A copy of `storage/src/tests/fixtures/ticker`, which `sec` cannot import. Keep both the same.
    async fn count_ticks_until(is_done: &AtomicBool) -> u64 {
        let mut ticks = 0;
        while !is_done.load(Ordering::Relaxed) {
            ticks += 1;
            tokio::task::yield_now().await;
        }
        ticks
    }

    #[tokio::test]
    async fn should_store_the_sha256_of_the_body_when_building_from_a_raw_response() {
        let expected_result = Sha256Digest::of(BODY);

        let response = SecResponse::<ZipBody>::from_inner(zip_inner_response(BODY.to_vec()))
            .await
            .expect("Given status 200 and a ZIP, building the response should always succeed");
        let result = response.body_digest().clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_body_unchanged_when_building_from_a_raw_response() {
        let expected_result = BODY.to_vec();

        let response = SecResponse::<ZipBody>::from_inner(zip_inner_response(BODY.to_vec()))
            .await
            .expect("Given status 200 and a ZIP, building the response should always succeed");
        let result = response.into_body();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_content_length_when_a_raw_response_body_is_longer_than_the_header()
     {
        let response = http::Response::builder()
            .status(200)
            .header("content-type", "application/zip")
            .header("content-length", "4")
            .body(BODY.to_vec())
            .expect("Given a valid status code and header names, building the response should always succeed");

        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidContentLength {
            content_length: "4".to_string(),
            body_length: BODY.len(),
        }));

        let result = SecResponse::<ZipBody>::from_inner(reqwest::Response::from(response))
            .await
            .err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_poll_another_future_when_hashing_a_large_read_body() {
        let headers = headers("application/zip", None);
        let body = vec![0_u8; LARGE_BODY_SIZE];
        let build_is_done = AtomicBool::new(false);
        // Starts after the body read, so every tick comes from the hash.
        let build = async {
            let outcome = SecResponse::<ZipBody>::from_read_body(
                sample_url(),
                headers,
                ContentType::Zip,
                StatusCode::Ok,
                body,
            )
            .await;
            build_is_done.store(true, Ordering::Relaxed);
            outcome
        };
        let ticker = count_ticks_until(&build_is_done);

        let expected_result = true;

        let (_, ticks) = tokio::join!(build, ticker);
        let result = ticks > 1;

        assert_eq!(result, expected_result);
    }

    /// Builds a [`SecResponse<ZipBody>`] with the content type of the `Content-Type` header.
    fn zip_from_parts(
        url: Url,
        headers: Headers,
        status_code: StatusCode,
        body: Vec<u8>,
    ) -> Result<SecResponse<ZipBody>, InvalidSecResponse> {
        let content_type = headers.content_type().clone();
        SecResponse::from_parts(url, headers, content_type, status_code, body)
    }

    fn sample_url() -> Url {
        "https://www.sec.gov/files/dera/data/financial-statement-and-notes-data-sets/2024q1_notes.zip"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn headers(content_type: &str, content_length: Option<&str>) -> Headers {
        let mut raw = HashMap::from([("content-type".to_string(), content_type.to_string())]);
        if let Some(content_length) = content_length {
            raw.insert("content-length".to_string(), content_length.to_string());
        }
        Headers::new(raw)
    }

    #[test]
    fn should_serialize_every_field_but_the_body_for_a_valid_notes_file() {
        let response = zip_from_parts(
            sample_url(),
            headers("application/zip", None),
            StatusCode::Ok,
            BODY.to_vec(),
        )
        .expect("Given status 200 and a ZIP, building the response should always succeed");

        let expected_result = vec![
            "body_sha256",
            "content_type",
            "headers",
            "status_code",
            "url",
        ];

        let value = serde_json::to_value(&response).expect(
            "Given a response of strings and a digest, serialization should always succeed",
        );
        let result: Vec<&str> = value
            .as_object()
            .map(|fields| fields.keys().map(String::as_str).collect())
            .unwrap_or_default();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_store_the_sha256_of_the_body_for_a_valid_notes_file() {
        let expected_result = Sha256Digest::of(BODY);

        let response = zip_from_parts(
            sample_url(),
            headers("application/zip", Some(&BODY.len().to_string())),
            StatusCode::Ok,
            BODY.to_vec(),
        )
        .expect("Given status 200, a ZIP and a matching length, building should always succeed");
        let result = response.body_digest().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_body_unchanged_when_consuming_the_response() {
        let response = zip_from_parts(
            sample_url(),
            headers("application/zip", None),
            StatusCode::Ok,
            BODY.to_vec(),
        )
        .expect("Given status 200 and a ZIP, building the response should always succeed");

        let expected_result = BODY;

        let result = response.into_body();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_the_body_when_the_content_length_header_is_absent() {
        let expected_result = BODY;

        let response = zip_from_parts(
            sample_url(),
            headers("application/octet-stream", None),
            StatusCode::Ok,
            BODY.to_vec(),
        )
        .expect("Given status 200, an octet stream and no length header, building should always succeed");
        let result = response.body();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_status_code_when_the_status_is_206_partial_content() {
        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
            status_code: StatusCode::Other(206),
        }));

        let result = zip_from_parts(
            sample_url(),
            headers("application/zip", None),
            StatusCode::Other(206),
            BODY.to_vec(),
        )
        .err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_content_length_when_the_header_is_not_a_number() {
        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidContentLength {
            content_length: "eleven".to_string(),
            body_length: BODY.len(),
        }));

        let result = zip_from_parts(
            sample_url(),
            headers("application/zip", Some("eleven")),
            StatusCode::Ok,
            BODY.to_vec(),
        )
        .err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_content_type_when_the_content_type_header_is_missing() {
        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidContentType {
            content_type: ContentType::Unknown,
        }));

        let result = zip_from_parts(
            sample_url(),
            Headers::new(HashMap::new()),
            StatusCode::Ok,
            BODY.to_vec(),
        )
        .err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_content_length_when_the_body_is_longer_than_the_header() {
        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidContentLength {
            content_length: "4".to_string(),
            body_length: BODY.len(),
        }));

        let result = zip_from_parts(
            sample_url(),
            headers("application/zip", Some("4")),
            StatusCode::Ok,
            BODY.to_vec(),
        )
        .err();

        assert_eq!(result, expected_result);
    }
}
