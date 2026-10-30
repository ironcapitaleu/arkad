//! # ZIP Body
//!
//! Provides the [`ZipBody`], the body kind of a response to a request for one SEC Financial
//! Statement and Notes file.

use async_trait::async_trait;
use bytes::Bytes;
use storage::shared::Sha256Digest;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::response::implementations::sec_response::error::{
    ErrorReason, InvalidSecResponse,
};
use crate::shared::status_code::StatusCode;

use super::ResponseBody;

/// The name of the header that states the body length in bytes.
const CONTENT_LENGTH_HEADER: &str = "content-length";

/// The bytes of a ZIP file, exactly as the server sent them.
///
/// Accepts status 200 only, and the [`ContentType::OctetStream`] or [`ContentType::Zip`]
/// content type. If the `Content-Length` header is present, the body has exactly that many bytes.
///
/// Not `Clone`: the pipeline gives each response one owner. The buffer is freed when the owner
/// and every [`Bytes`] from [`ZipBody::to_bytes`] drop. So an `SecResponse<ZipBody>` is not
/// `Clone` either:
///
/// ```compile_fail,E0277
/// use sec::shared::response::implementations::sec_response::{SecResponse, ZipBody};
///
/// fn assert_clone<T: Clone>() {}
///
/// assert_clone::<SecResponse<ZipBody>>();
/// ```
#[derive(Debug)]
pub struct ZipBody(Bytes);

impl ZipBody {
    /// Returns the bytes as a [`Bytes`] that shares the buffer, so the call copies a pointer.
    #[must_use]
    pub fn to_bytes(&self) -> Bytes {
        self.0.clone()
    }
}

#[async_trait]
impl ResponseBody for ZipBody {
    type Content = [u8];

    /// Checks that the status code is 200 and the content type is `OctetStream` or `Zip`.
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
        check_content_length(headers, &raw_body)?;
        let body_digest = Sha256Digest::of(&raw_body);
        Ok((Self(Bytes::from(raw_body)), body_digest))
    }

    /// Checks the body length, and hashes the body on the blocking thread pool with
    /// [`Sha256Digest::of_owned_async`], so the task polls other futures during the hash.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidContentLength`] if the body
    /// length does not match the `Content-Length` header.
    async fn from_read(
        headers: &Headers,
        raw_body: Vec<u8>,
    ) -> Result<(Self, Sha256Digest), InvalidSecResponse> {
        check_content_length(headers, &raw_body)?;
        let (body_digest, raw_body) = Sha256Digest::of_owned_async(raw_body).await;
        Ok((Self(Bytes::from(raw_body)), body_digest))
    }

    fn content(&self) -> &Self::Content {
        &self.0
    }
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
