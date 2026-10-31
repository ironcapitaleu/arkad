//! # ZIP Body
//!
//! Provides the [`ZipBody`], the body kind of a response that the store keeps as raw bytes. The
//! body is one SEC Financial Statement and Notes file (a ZIP) or one daily index (text).

use async_trait::async_trait;
use bytes::Bytes;
use storage::shared::Sha256Digest;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::response::implementations::sec_response::error::{
    ErrorReason, InvalidSecResponse,
};
use crate::shared::status_code::StatusCode;

use super::{FileBody, ResponseBody, checked_bytes, checked_bytes_off_task};

/// The bytes of a ZIP file or a daily index, exactly as the server sent them.
///
/// Accepts status 200 only, and the [`ContentType::OctetStream`] or [`ContentType::Zip`]
/// content type. If the `Content-Length` header is present, the body has exactly that many bytes.
///
/// Not `Clone`: the pipeline gives each response one owner. The buffer is freed when the owner
/// and every [`Bytes`] from [`FileBody::to_bytes`] drop. So an `SecResponse<ZipBody>` is not
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

impl FileBody for ZipBody {
    fn to_bytes(&self) -> Bytes {
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
        let (bytes, body_digest) = checked_bytes(headers, raw_body)?;
        Ok((Self(bytes), body_digest))
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
        let (bytes, body_digest) = checked_bytes_off_task(headers, raw_body).await?;
        Ok((Self(bytes), body_digest))
    }

    fn content(&self) -> &Self::Content {
        &self.0
    }
}
