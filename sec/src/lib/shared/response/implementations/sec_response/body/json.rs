//! # JSON Body
//!
//! Provides the [`JsonBody`], the body kind of an SEC API response with a JSON body.

use async_trait::async_trait;
use storage::shared::Sha256Digest;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::response::implementations::sec_response::error::{
    ErrorReason, InvalidSecResponse,
};
use crate::shared::status_code::StatusCode;

use super::ResponseBody;

/// A JSON body: the parsed value and the raw bytes exactly as the server sent them.
///
/// Accepts any 2xx status code and the [`ContentType::Json`] content type.
#[derive(Debug, Clone)]
pub struct JsonBody {
    value: serde_json::Value,
    raw: Vec<u8>,
}

impl JsonBody {
    /// Returns the body bytes exactly as the server sent them.
    #[must_use]
    pub fn raw(&self) -> &[u8] {
        &self.raw
    }
}

#[async_trait]
impl ResponseBody for JsonBody {
    type Content = serde_json::Value;

    /// Checks that the status code is in the 2xx range and the content type is JSON.
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

    /// Parses the raw body as JSON and hashes the raw bytes.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidBody`] if the raw body is
    /// not valid JSON.
    fn from_raw(
        _headers: &Headers,
        raw_body: Vec<u8>,
    ) -> Result<(Self, Sha256Digest), InvalidSecResponse> {
        let body_digest = Sha256Digest::of(&raw_body);
        let value = serde_json::from_slice(&raw_body).map_err(|e| {
            InvalidSecResponse::new(ErrorReason::InvalidBody {
                details: e.to_string(),
            })
        })?;
        Ok((
            Self {
                value,
                raw: raw_body,
            },
            body_digest,
        ))
    }

    /// Calls [`from_raw`](ResponseBody::from_raw), because a JSON body is small enough to hash on
    /// the task.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSecResponse`] with [`ErrorReason::InvalidBody`] if the raw body is
    /// not valid JSON.
    async fn from_read(
        headers: &Headers,
        raw_body: Vec<u8>,
    ) -> Result<(Self, Sha256Digest), InvalidSecResponse> {
        Self::from_raw(headers, raw_body)
    }

    fn content(&self) -> &Self::Content {
        &self.value
    }
}
