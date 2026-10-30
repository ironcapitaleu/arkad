//! # Response Body Kinds
//!
//! Provides the [`ResponseBody`] trait and the body kinds an
//! [`SecResponse`](super::SecResponse) can hold.
//!
//! ## Modules
//!
//! - [`json`]: The [`JsonBody`], a parsed JSON body with its raw bytes.
//! - [`zip`]: The [`ZipBody`], the bytes of a ZIP file.

use std::fmt::Debug;

use async_trait::async_trait;
use storage::shared::Sha256Digest;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::status_code::StatusCode;

use super::error::InvalidSecResponse;

pub mod json;
pub mod zip;

pub use json::JsonBody;
pub use zip::ZipBody;

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
