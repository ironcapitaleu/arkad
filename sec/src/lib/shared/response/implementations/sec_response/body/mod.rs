//! # Response Body Kinds
//!
//! Provides the [`ResponseBody`] and [`FileBody`] traits and the body kinds an
//! [`SecResponse`](super::SecResponse) can hold.
//!
//! ## Modules
//!
//! - [`json`]: The [`JsonBody`], a parsed JSON body with its raw bytes.
//! - [`xml`]: The [`XmlBody`], the bytes of an XML file.
//! - [`zip`]: The [`ZipBody`], the bytes of a ZIP file.

use std::fmt::Debug;

use async_trait::async_trait;
use bytes::Bytes;
use storage::shared::Sha256Digest;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::status_code::StatusCode;

use super::error::{ErrorReason, InvalidSecResponse};

pub mod json;
pub mod xml;
pub mod zip;

pub use json::JsonBody;
pub use xml::XmlBody;
pub use zip::ZipBody;

/// The name of the header that states the body length in bytes.
const CONTENT_LENGTH_HEADER: &str = "content-length";

/// The body kind of an [`SecResponse`](super::SecResponse): the checks it runs and the body it
/// stores.
///
/// An [`SecResponse`](super::SecResponse) calls
/// [`check_status_and_content_type`](ResponseBody::check_status_and_content_type) before it
/// reads the body, and then builds the body from the raw bytes.
#[async_trait]
pub trait ResponseBody: Debug + Send + Sync + Sized {
    /// The view of the body that [`SecResponse::body`](crate::shared::response::SecResponse::body)
    /// returns.
    type Content: ?Sized;

    /// Checks the status code and the content type, before the body is read.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] for the first check that fails.
    fn check_status_and_content_type(
        status_code: StatusCode,
        content_type: &ContentType,
    ) -> Result<(), InvalidSecResponse>;

    /// Checks the raw body, builds the body, and hashes the raw bytes on the current task.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] if the raw body fails a check of this body kind.
    fn from_raw(
        headers: &Headers,
        raw_body: Vec<u8>,
    ) -> Result<(Self, Sha256Digest), InvalidSecResponse>;

    /// Checks the raw body that was read from the network, builds the body, and hashes the raw
    /// bytes.
    ///
    /// A body kind for large bodies hashes off the task here.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] if the raw body fails a check of this body kind.
    async fn from_read(
        headers: &Headers,
        raw_body: Vec<u8>,
    ) -> Result<(Self, Sha256Digest), InvalidSecResponse>;

    /// Returns the view of the body.
    fn content(&self) -> &Self::Content;
}

/// A [`ResponseBody`] that holds the bytes of one file, which the store keeps unchanged.
pub trait FileBody: ResponseBody<Content = [u8]> {
    /// Returns the bytes as a [`Bytes`] that shares the buffer, so the call copies a pointer.
    ///
    /// The bytes must equal [`content`](ResponseBody::content), because the metadata describes
    /// `content` and the store writes these bytes.
    #[must_use]
    fn to_bytes(&self) -> Bytes;
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

/// Checks the body length against the `Content-Length` header and hashes the body on the current
/// task.
///
/// # Errors
///
/// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidContentLength`] if the body
/// length does not match the `Content-Length` header.
fn checked_bytes(
    headers: &Headers,
    raw_body: Vec<u8>,
) -> Result<(Bytes, Sha256Digest), InvalidSecResponse> {
    check_content_length(headers, &raw_body)?;
    let body_digest = Sha256Digest::of(&raw_body);
    Ok((Bytes::from(raw_body), body_digest))
}

/// Checks the body length against the `Content-Length` header, and hashes the body on the
/// blocking thread pool with [`Sha256Digest::of_owned_async`], so the task polls other futures
/// during the hash.
///
/// # Errors
///
/// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidContentLength`] if the body
/// length does not match the `Content-Length` header.
async fn checked_bytes_off_task(
    headers: &Headers,
    raw_body: Vec<u8>,
) -> Result<(Bytes, Sha256Digest), InvalidSecResponse> {
    check_content_length(headers, &raw_body)?;
    let (body_digest, raw_body) = Sha256Digest::of_owned_async(raw_body).await;
    Ok((Bytes::from(raw_body), body_digest))
}
