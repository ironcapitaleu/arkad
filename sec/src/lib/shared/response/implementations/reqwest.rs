//! # Reqwest Inner Response
//!
//! Implements [`InnerResponse`] for [`reqwest::Response`], binding the transport's part types and
//! deriving a [`ContentType`] from the `Content-Type` header.

use async_trait::async_trait;
use reqwest::header::HeaderMap;
use reqwest::{Response, StatusCode, Url};

use crate::shared::content_type::ContentType;

use super::super::traits::InnerResponse;

#[async_trait]
impl InnerResponse for Response {
    /// The [`reqwest::Url`] type.
    type Url = Url;
    /// The [`reqwest::header::HeaderMap`] type.
    type Headers = HeaderMap;
    /// The body as a UTF-8 [`String`].
    type Body = String;
    /// The [`reqwest::StatusCode`] type.
    type StatusCode = StatusCode;
    /// The crate's own [`ContentType`], parsed from the `Content-Type` header.
    type ContentType = ContentType;
    /// The [`reqwest::Error`] type.
    type Error = reqwest::Error;

    fn url(&self) -> &Self::Url {
        self.url()
    }

    fn headers(&self) -> &Self::Headers {
        self.headers()
    }

    fn status_code(&self) -> Self::StatusCode {
        self.status()
    }

    /// Returns [`ContentType::Unknown`] if the `Content-Type` header is absent or not valid UTF-8.
    fn content_type(&self) -> Self::ContentType {
        self.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map_or_else(|| ContentType::Unknown, ContentType::from_content_type)
    }

    async fn body(self) -> Result<Self::Body, Self::Error> {
        self.text().await
    }
}
