//! # HTML Body
//!
//! Provides the [`HtmlBody`], the body kind of a response that holds one HTML page, for example
//! the SEC page that links the Notes files.

use async_trait::async_trait;
use bytes::Bytes;
use storage::shared::Sha256Digest;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::response::implementations::sec_response::error::{
    ErrorReason, InvalidSecResponse,
};
use crate::shared::status_code::StatusCode;

use super::{ResponseBody, checked_bytes};

/// The bytes of an HTML page, exactly as the server sent them.
///
/// Accepts status 200 only, and the [`ContentType::Html`] content type. If the `Content-Length`
/// header is present, the body has exactly that many bytes.
#[derive(Debug)]
pub struct HtmlBody(Bytes);

#[async_trait]
impl ResponseBody for HtmlBody {
    type Content = [u8];

    /// Checks that the status code is 200 and the content type is `Html`.
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
        if *content_type != ContentType::Html {
            return Err(InvalidSecResponse::new(ErrorReason::InvalidContentType {
                content_type: content_type.clone(),
            }));
        }
        Ok(())
    }

    /// Checks the body length against the `Content-Length` header and hashes the body.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidContentLength`] if the body
    /// length does not match the `Content-Length` header.
    fn from_raw(
        headers: &Headers,
        raw_body: Vec<u8>,
    ) -> Result<(Self, Sha256Digest), InvalidSecResponse> {
        let (bytes, body_digest) = checked_bytes(headers, raw_body)?;
        Ok((Self(bytes), body_digest))
    }

    /// Runs the checks of [`from_raw`](Self::from_raw) on the current task. A page is small, so
    /// the hash does not need the blocking thread pool.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidContentLength`] if the body
    /// length does not match the `Content-Length` header.
    async fn from_read(
        headers: &Headers,
        raw_body: Vec<u8>,
    ) -> Result<(Self, Sha256Digest), InvalidSecResponse> {
        Self::from_raw(headers, raw_body)
    }

    fn content(&self) -> &Self::Content {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::response::SecResponse as SecResponseTrait;
    use crate::shared::response::implementations::sec_response::SecResponse;
    use crate::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;

    use super::HtmlBody;

    const BODY: &[u8] = b"<html></html>";

    fn sample_url() -> Url {
        "https://www.sec.gov/data-research/sec-markets-data/financial-statement-notes-data-sets"
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

    fn html_from_parts(
        headers: Headers,
        status_code: StatusCode,
    ) -> Result<SecResponse<HtmlBody>, InvalidSecResponse> {
        let content_type = headers.content_type().clone();
        SecResponse::from_parts(
            sample_url(),
            headers,
            content_type,
            status_code,
            BODY.to_vec(),
        )
    }

    #[test]
    fn should_keep_the_body_unchanged_when_the_content_type_is_html_with_a_charset() {
        let headers = headers("text/html; charset=utf-8", Some("13"));

        let expected_result = BODY;

        let response = html_from_parts(headers, StatusCode::Ok).expect(
            "Given status 200, text/html, and a matching length, building should always succeed",
        );
        let result = response.body();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_status_code_when_the_status_is_404() {
        let headers = headers("text/html", None);

        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
            status_code: StatusCode::NotFound,
        }));

        let result = html_from_parts(headers, StatusCode::NotFound).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_content_type_when_the_content_type_is_json() {
        let headers = headers("application/json", None);

        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidContentType {
            content_type: ContentType::Json,
        }));

        let result = html_from_parts(headers, StatusCode::Ok).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_content_length_when_the_body_is_longer_than_the_header() {
        let headers = headers("text/html", Some("4"));

        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidContentLength {
            content_length: "4".to_string(),
            body_length: BODY.len(),
        }));

        let result = html_from_parts(headers, StatusCode::Ok).err();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_html_body() {
        implements_auto_traits::<HtmlBody>();
    }
}
