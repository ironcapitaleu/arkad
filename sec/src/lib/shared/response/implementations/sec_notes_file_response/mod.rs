//! # SEC Notes File Response
//!
//! Provides the [`SecNotesFileResponse`], the validated response to a request for one SEC
//! Financial Statement and Notes file.
//!
//! It does not implement the [`SecResponse`](crate::shared::response::SecResponse) trait, because
//! that trait requires a JSON body. It reuses the [`InvalidSecResponse`] error.

use std::collections::HashMap;

use storage::shared::Sha256Digest;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::response::implementations::sec_response::error::{
    ErrorReason, InvalidSecResponse,
};
use crate::shared::status_code::StatusCode;
use crate::shared::url::Url;

/// The name of the header that states the body length in bytes.
const CONTENT_LENGTH_HEADER: &str = "content-length";

/// A validated response to a Notes file request: status 200, an accepted content type, and the
/// body bytes.
///
/// The accepted content types are [`ContentType::OctetStream`] and [`ContentType::Zip`]. If the
/// `Content-Length` header is present, the body has exactly that many bytes.
#[derive(Debug)]
pub struct SecNotesFileResponse {
    url: Url,
    headers: Headers,
    content_type: ContentType,
    status_code: StatusCode,
    body: Vec<u8>,
    body_digest: Sha256Digest,
}

impl SecNotesFileResponse {
    /// Creates a [`SecNotesFileResponse`] from its parts and checks the status, the content type,
    /// and the body length.
    ///
    /// The content type comes from the `Content-Type` header.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] with:
    /// - [`ErrorReason::InvalidStatusCode`] if the status code is not 200.
    /// - [`ErrorReason::InvalidContentType`] if the content type is not `OctetStream` or `Zip`.
    /// - [`ErrorReason::InvalidContentLength`] if the body length does not match the
    ///   `Content-Length` header.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use sec::shared::headers::Headers;
    /// use sec::shared::response::implementations::sec_notes_file_response::SecNotesFileResponse;
    /// use sec::shared::status_code::StatusCode;
    /// use sec::shared::url::Url;
    ///
    /// let url: Url = "https://www.sec.gov/files/dera/data/financial-statement-and-notes-data-sets/2024q1_notes.zip"
    ///     .parse()
    ///     .expect("Given a hardcoded valid URL, parsing should always succeed");
    /// let headers = Headers::new(HashMap::from([(
    ///     "content-type".to_string(),
    ///     "application/octet-stream".to_string(),
    /// )]));
    /// let response = SecNotesFileResponse::from_parts(url, headers, StatusCode::Ok, b"PK".to_vec())
    ///     .expect("Given status 200 and an octet stream, building the response should always succeed");
    ///
    /// let expected_result = b"PK";
    ///
    /// let result = response.body();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn from_parts(
        url: Url,
        headers: Headers,
        status_code: StatusCode,
        body: Vec<u8>,
    ) -> Result<Self, InvalidSecResponse> {
        let content_type = headers.content_type().clone();
        check_status_and_content_type(status_code, &content_type)?;
        check_content_length(&headers, &body)?;
        let body_digest = Sha256Digest::of(&body);
        Ok(Self {
            url,
            headers,
            content_type,
            status_code,
            body,
            body_digest,
        })
    }

    /// Creates a [`SecNotesFileResponse`] from a raw `reqwest` response, reading the whole body.
    ///
    /// The status code and the content type are checked before the body is read.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] with [`ErrorReason::FailedBodyRead`] if the body cannot
    /// be read, or with any reason of [`SecNotesFileResponse::from_parts`].
    pub async fn from_inner(inner: reqwest::Response) -> Result<Self, InvalidSecResponse> {
        let url = Url::from(inner.url().clone());
        let status_code = StatusCode::from(inner.status());
        let raw_headers: HashMap<String, String> = inner
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let headers = Headers::new(raw_headers);

        check_status_and_content_type(status_code, headers.content_type())?;

        let body = inner.bytes().await.map_err(|e| {
            InvalidSecResponse::new(ErrorReason::FailedBodyRead {
                details: e.to_string(),
            })
        })?;

        Self::from_parts(url, headers, status_code, Vec::from(body))
    }

    /// Returns the URL the response came from.
    #[must_use]
    pub const fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the response headers.
    #[must_use]
    pub const fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Returns the content type, either `OctetStream` or `Zip`.
    #[must_use]
    pub const fn content_type(&self) -> &ContentType {
        &self.content_type
    }

    /// Returns the status code, which is always 200.
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        self.status_code
    }

    /// Returns the body bytes exactly as the server sent them.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Returns the SHA-256 digest of the body bytes.
    #[must_use]
    pub const fn body_digest(&self) -> &Sha256Digest {
        &self.body_digest
    }
}

/// Checks that a response has status 200 and an `OctetStream` or `Zip` content type.
///
/// # Errors
///
/// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidStatusCode`] or
/// [`ErrorReason::InvalidContentType`] for the first check that fails.
fn check_status_and_content_type(
    status_code: StatusCode,
    content_type: &ContentType,
) -> Result<(), InvalidSecResponse> {
    if status_code != StatusCode::Ok {
        return Err(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
            status_code,
        }));
    }
    if !matches!(content_type, ContentType::OctetStream | ContentType::Zip) {
        return Err(InvalidSecResponse::new(ErrorReason::InvalidContentType {
            content_type: content_type.clone(),
        }));
    }
    Ok(())
}

/// Checks that the body length matches the `Content-Length` header, if the header is present.
///
/// # Errors
///
/// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidContentLength`] if the header
/// is not a number or does not match the body length.
fn check_content_length(headers: &Headers, body: &[u8]) -> Result<(), InvalidSecResponse> {
    let Some(content_length) = headers.get(CONTENT_LENGTH_HEADER) else {
        return Ok(());
    };
    if content_length.trim().parse::<usize>() != Ok(body.len()) {
        return Err(InvalidSecResponse::new(ErrorReason::InvalidContentLength {
            content_length: content_length.to_string(),
            body_length: body.len(),
        }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const BODY: &[u8] = b"PK\x03\x04 notes";

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
    fn should_store_the_sha256_of_the_body_for_a_valid_notes_file() {
        let expected_result = Sha256Digest::of(BODY);

        let response = SecNotesFileResponse::from_parts(
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
    fn should_accept_the_body_when_the_content_length_header_is_absent() {
        let expected_result = BODY;

        let response = SecNotesFileResponse::from_parts(
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

        let result = SecNotesFileResponse::from_parts(
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

        let result = SecNotesFileResponse::from_parts(
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

        let result = SecNotesFileResponse::from_parts(
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

        let result = SecNotesFileResponse::from_parts(
            sample_url(),
            headers("application/zip", Some("4")),
            StatusCode::Ok,
            BODY.to_vec(),
        )
        .err();

        assert_eq!(result, expected_result);
    }
}
