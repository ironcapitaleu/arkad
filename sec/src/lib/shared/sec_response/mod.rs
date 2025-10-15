//! # SEC Response Module
//!
//! This module provides the [`SecResponse`] type and related utilities for handling
//! HTTP responses from SEC (Securities and Exchange Commission) endpoints. It wraps
//! response data with convenient accessors and content type detection to simplify
//! working with SEC API responses.
//!
//! ## Overview
//! The [`SecResponse`] is a wrapper around HTTP response data that preserves key
//! response information including URL, status code, headers, and body content.
//! It automatically detects and categorizes content types for easier response processing.
//!
//! ## Types
//! - [`SecResponse`]: Main response wrapper containing URL, status, headers, and body.
//! - [`ContentType`]: Enumeration of supported content types with automatic detection.
//!
//! ## See Also
//! - [`reqwest::Response`]: Underlying HTTP response implementation.
//! - [`super::sec_client`]: HTTP client for making SEC-compliant requests.

use std::collections::HashMap;
use std::fmt;

use reqwest::{Response, StatusCode, Url};

/// A wrapper around HTTP response data from SEC endpoints.
///
/// `SecResponse` preserves key information from HTTP responses including the request URL,
/// status code, headers, and response body. It automatically detects content types and
/// provides convenient accessor methods for working with SEC API responses.
///
/// # Examples
///
/// ```rust
/// use reqwest::{Response, StatusCode};
/// use sec::shared::sec_response::{SecResponse, ContentType};
///
/// // Create a SecResponse from a reqwest Response (async)
/// // let response: Response = /* ... make HTTP request ... */;
/// // let sec_response = SecResponse::from_response(response).await?;
///
/// // Access response data
/// // println!("Status: {}", sec_response.status());
/// // println!("URL: {}", sec_response.url());
/// // println!("Body length: {}", sec_response.body().len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Content Type Detection
/// The response automatically detects content types from response headers:
/// - JSON: `application/json` or any header containing "json"
/// - XML: `application/xml`, `text/xml`, or any header containing "xml"
/// - HTML: `text/html` or any header containing "html"
/// - Text: `text/plain` or any header starting with "text/"
/// - Other: Any unrecognized content type is preserved as-is
#[derive(Debug, Clone)]
pub struct SecResponse {
    pub url: Url,
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub content_type: ContentType,
    pub body: String,
}

/// Content type enumeration with automatic detection from HTTP headers.
///
/// `ContentType` represents the main content types expected from SEC API responses.
/// It provides methods to detect content type from response headers and format
/// content type strings for use in HTTP headers.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use sec::shared::sec_response::ContentType;
///
/// let mut headers = HashMap::new();
/// headers.insert("content-type".to_string(), "application/json".to_string());
///
/// let content_type = ContentType::from_headers(&headers);
/// assert_eq!(content_type, ContentType::Json);
/// assert_eq!(content_type.to_string(), "application/json");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContentType {
    Json,
    Xml,
    Html,
    Text,
    Other(String),
}

impl ContentType {
    /// Determines the content type from response headers.
    #[must_use]
    pub fn from_headers(headers: &HashMap<String, String>) -> Self {
        headers.get("content-type").map_or_else(
            || Self::Other("unknown".to_string()),
            |content_type| Self::from_content_type(content_type),
        )
    }
    /// Determines the content type from a string.
    #[must_use]
    pub fn from_content_type(content_type_str: &str) -> Self {
        let content_type_lower = content_type_str.to_lowercase();

        if content_type_lower.contains("application/json") || content_type_lower.contains("json") {
            Self::Json
        } else if content_type_lower.contains("application/xml")
            || content_type_lower.contains("text/xml")
            || content_type_lower.contains("xml")
        {
            Self::Xml
        } else if content_type_lower.contains("text/html") || content_type_lower.contains("html") {
            Self::Html
        } else if content_type_lower.contains("text/plain") || content_type_lower.contains("text/")
        {
            Self::Text
        } else {
            Self::Other(content_type_str.to_string())
        }
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Json => write!(f, "application/json"),
            Self::Xml => write!(f, "application/xml"),
            Self::Html => write!(f, "text/html"),
            Self::Text => write!(f, "text/plain"),
            Self::Other(content_type) => write!(f, "{content_type}"),
        }
    }
}

impl SecResponse {
    /// Creates a new [`SecResponse`] from a [`Response`] by consuming its body.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the response body fails.
    pub async fn from_response(response: Response) -> Result<Self, reqwest::Error> {
        let url = response.url().clone();
        let status = response.status();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                let value_str = value.to_str().map_or_else(
                    |_| {
                        eprintln!("WARNING: Header value for '{name}' is not valid UTF-8");
                        "invalid-utf8".to_string()
                    },
                    std::string::ToString::to_string,
                );

                (name.to_string(), value_str)
            })
            .collect();

        let content_type = ContentType::from_headers(&headers);
        let body = response.text().await?;

        Ok(Self {
            url,
            status,
            headers,
            content_type,
            body,
        })
    }

    /// Returns the URL of the response.
    #[must_use]
    pub const fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the status code of the response.
    #[must_use]
    pub const fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns the headers of the response as a map.
    #[must_use]
    pub const fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Returns the response body as a string.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns the content type of the response as a `ContentType` enum.
    #[must_use]
    pub const fn content_type(&self) -> &ContentType {
        &self.content_type
    }
}

impl PartialEq for SecResponse {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl PartialOrd for SecResponse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SecResponse {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.url.cmp(&other.url)
    }
}

impl Eq for SecResponse {}

impl std::hash::Hash for SecResponse {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url.hash(state);
    }
}

impl Default for SecResponse {
    fn default() -> Self {
        Self {
            url: Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json")
                .expect("Default SEC URL should always be valid"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("Default response body"),
        }
    }
}

impl fmt::Display for SecResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SEC Response:\n\
             \t\tStatus: {}\n\
             \t\tURL: {}\n\
             \t\tContent-Type: {}\n\
             \t\tBody Length: {} bytes",
            self.status,
            self.url,
            self.content_type,
            self.body.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_detect_json_content_type_when_application_json_header_is_present() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let expected_result = ContentType::Json;

        let result = ContentType::from_headers(&headers);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unknown_content_type_when_no_content_type_header_is_present() {
        let headers = HashMap::new();
        let expected_result = ContentType::Other("unknown".to_string());

        let result = ContentType::from_headers(&headers);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_detect_content_type_case_insensitively_when_header_has_mixed_case() {
        let content_type_str = "APPLICATION/JSON; charset=UTF-8";

        let expected_result = ContentType::Json;
        let result = ContentType::from_content_type(content_type_str);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_default_response_when_default_is_called() {
        let result = SecResponse::default();
        let expected_result = SecResponse {
            url: Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json")
                .expect("Default SEC URL should always be valid"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("Default response body"),
        };

        assert_eq!(result, expected_result);
    }
}
