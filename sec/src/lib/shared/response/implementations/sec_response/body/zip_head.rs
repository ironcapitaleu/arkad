//! # ZIP Head Body
//!
//! Provides the [`ZipHeadBody`], the body kind of the answer to a `HEAD` request for a ZIP file.

use async_trait::async_trait;
use storage::shared::Sha256Digest;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::response::implementations::sec_response::error::InvalidSecResponse;
use crate::shared::status_code::StatusCode;

use super::{ResponseBody, ZipBody};

/// The empty body of the answer to a `HEAD` request for a ZIP file.
///
/// Accepts the status and the content types of a [`ZipBody`]. A `HEAD` answer has no body, but its
/// `Content-Length` header states the length of the file. So this kind does not check the length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZipHeadBody;

#[async_trait]
impl ResponseBody for ZipHeadBody {
    type Content = ();

    /// Runs the checks of [`ZipBody`]: status 200, and a ZIP or octet-stream content type.
    ///
    /// # Errors
    ///
    /// Returns the [`InvalidSecResponse`] of [`ZipBody::check_status_and_content_type`].
    fn check_status_and_content_type(
        status_code: StatusCode,
        content_type: &ContentType,
    ) -> Result<(), InvalidSecResponse> {
        ZipBody::check_status_and_content_type(status_code, content_type)
    }

    /// Hashes the raw body and drops it.
    ///
    /// # Errors
    ///
    /// Never returns an error.
    fn from_raw(
        _headers: &Headers,
        raw_body: Vec<u8>,
    ) -> Result<(Self, Sha256Digest), InvalidSecResponse> {
        Ok((Self, Sha256Digest::of(&raw_body)))
    }

    /// Runs [`from_raw`](Self::from_raw). The body is empty, so the hash takes no time.
    ///
    /// # Errors
    ///
    /// Never returns an error.
    async fn from_read(
        headers: &Headers,
        raw_body: Vec<u8>,
    ) -> Result<(Self, Sha256Digest), InvalidSecResponse> {
        Self::from_raw(headers, raw_body)
    }

    fn content(&self) -> &Self::Content {
        &()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::response::implementations::sec_response::SecResponse;
    use crate::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;

    use super::ZipHeadBody;

    fn sample_url() -> Url {
        "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2025q3_notes.zip"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn head_from_parts(
        content_type: &str,
        status_code: StatusCode,
    ) -> Result<SecResponse<ZipHeadBody>, InvalidSecResponse> {
        // The SEC sends the length of the file with a `HEAD` answer, and no body.
        let headers = Headers::new(HashMap::from([
            ("content-type".to_string(), content_type.to_string()),
            ("content-length".to_string(), "463365283".to_string()),
        ]));
        let content_type = headers.content_type().clone();
        SecResponse::from_parts(sample_url(), headers, content_type, status_code, Vec::new())
    }

    #[test]
    fn should_accept_the_answer_when_the_file_exists_and_the_body_is_empty() {
        let content_type = "application/octet-stream";

        let expected_result = None;

        let result = head_from_parts(content_type, StatusCode::Ok).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_status_code_when_the_status_is_404() {
        let content_type = "text/html; charset=utf-8";

        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
            status_code: StatusCode::NotFound,
        }));

        let result = head_from_parts(content_type, StatusCode::NotFound).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_content_type_when_the_status_is_200_with_html() {
        let content_type = "text/html";

        let expected_result = Some(InvalidSecResponse::new(ErrorReason::InvalidContentType {
            content_type: ContentType::Html,
        }));

        let result = head_from_parts(content_type, StatusCode::Ok).err();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_zip_head_body() {
        implements_auto_traits::<ZipHeadBody>();
    }
}
