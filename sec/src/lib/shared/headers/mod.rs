use std::collections::HashMap;

use crate::shared::content_type::ContentType;

use self::constants::{ACCEPT_ENCODING_HEADER, CONTENT_TYPE_HEADER, DATE_HEADER, ETAG_HEADER};

pub mod constants;
pub mod headers_error;

pub use headers_error::{HeadersError, InvalidHeadersReason};

/// Validated HTTP response headers from an SEC API endpoint.
///
/// The `Headers` type provides typed accessors for known headers and stores
/// remaining headers in a catch-all map. `ContentType` is parsed from the
/// `content-type` header during construction.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use sec::shared::headers::Headers;
/// use sec::shared::content_type::ContentType;
///
/// let mut raw = HashMap::new();
/// raw.insert("content-type".to_string(), "application/json".to_string());
/// raw.insert("etag".to_string(), "\"abc123\"".to_string());
/// raw.insert("x-custom".to_string(), "value".to_string());
///
/// let headers = Headers::new(raw);
/// assert_eq!(*headers.content_type(), ContentType::Json);
/// assert_eq!(headers.etag(), Some("\"abc123\""));
/// assert_eq!(headers.get("x-custom"), Some("value"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Headers {
    /// The content type parsed from the `Content-Type` header.
    content_type: ContentType,
    /// The `ETag` header value, if present.
    etag: Option<String>,
    /// The Date header value, if present.
    date: Option<String>,
    /// The Accept-Encoding header value, if present.
    accept_encoding: Option<String>,
    /// Any remaining headers not captured by typed fields.
    other: HashMap<String, String>,
}

impl Headers {
    /// Creates a new `Headers` from a raw header map.
    ///
    /// Known headers are extracted into typed fields. The `content-type`
    /// header is parsed into a `ContentType` value. Remaining headers
    /// are stored in the overflow map.
    #[must_use]
    pub fn new(mut raw_headers: HashMap<String, String>) -> Self {
        let content_type = raw_headers
            .remove(CONTENT_TYPE_HEADER)
            .map_or(ContentType::Unknown, |v| ContentType::from_content_type(&v));

        let etag = raw_headers.remove(ETAG_HEADER);
        let date = raw_headers.remove(DATE_HEADER);
        let accept_encoding = raw_headers.remove(ACCEPT_ENCODING_HEADER);

        Self {
            content_type,
            etag,
            date,
            accept_encoding,
            other: raw_headers,
        }
    }

    /// Returns the content type parsed from the `Content-Type` header.
    #[must_use]
    pub const fn content_type(&self) -> &ContentType {
        &self.content_type
    }

    /// Returns the `ETag` header value, if present.
    #[must_use]
    pub fn etag(&self) -> Option<&str> {
        self.etag.as_deref()
    }

    /// Returns the `Date` header value, if present.
    #[must_use]
    pub fn date(&self) -> Option<&str> {
        self.date.as_deref()
    }

    /// Returns the `Accept-Encoding` header value, if present.
    #[must_use]
    pub fn accept_encoding(&self) -> Option<&str> {
        self.accept_encoding.as_deref()
    }

    /// Returns the value of a header by name from the overflow map.
    ///
    /// This does not search typed fields. Use the typed accessors for
    /// known headers.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.other.get(name).map(String::as_str)
    }

    /// Returns a reference to the overflow map of remaining headers.
    #[must_use]
    pub const fn other(&self) -> &HashMap<String, String> {
        &self.other
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_parse_content_type_as_json_when_content_type_header_is_application_json() {
        let mut raw = HashMap::new();
        raw.insert("content-type".to_string(), "application/json".to_string());

        let expected_result = ContentType::Json;

        let result = Headers::new(raw);

        assert_eq!(*result.content_type(), expected_result);
    }

    #[test]
    fn should_return_unknown_content_type_when_content_type_header_is_absent() {
        let raw = HashMap::new();

        let expected_result = ContentType::Unknown;

        let result = Headers::new(raw);

        assert_eq!(*result.content_type(), expected_result);
    }

    #[test]
    fn should_extract_etag_when_etag_header_is_present() {
        let mut raw = HashMap::new();
        raw.insert("etag".to_string(), "\"abc123\"".to_string());

        let expected_result = Some("\"abc123\"");

        let result = Headers::new(raw);

        assert_eq!(result.etag(), expected_result);
    }

    #[test]
    fn should_return_none_for_etag_when_etag_header_is_absent() {
        let raw = HashMap::new();

        let result = Headers::new(raw);

        assert_eq!(result.etag(), None);
    }

    #[test]
    fn should_extract_date_when_date_header_is_present() {
        let mut raw = HashMap::new();
        raw.insert(
            "date".to_string(),
            "Thu, 01 Jan 2026 00:00:00 GMT".to_string(),
        );

        let expected_result = Some("Thu, 01 Jan 2026 00:00:00 GMT");

        let result = Headers::new(raw);

        assert_eq!(result.date(), expected_result);
    }

    #[test]
    fn should_return_none_for_date_when_date_header_is_absent() {
        let raw = HashMap::new();

        let result = Headers::new(raw);

        assert_eq!(result.date(), None);
    }

    #[test]
    fn should_extract_accept_encoding_when_header_is_present() {
        let mut raw = HashMap::new();
        raw.insert("accept-encoding".to_string(), "gzip, deflate".to_string());

        let expected_result = Some("gzip, deflate");

        let result = Headers::new(raw);

        assert_eq!(result.accept_encoding(), expected_result);
    }

    #[test]
    fn should_return_none_for_accept_encoding_when_header_is_absent() {
        let raw = HashMap::new();

        let result = Headers::new(raw);

        assert_eq!(result.accept_encoding(), None);
    }

    #[test]
    fn should_store_unknown_headers_in_other_when_extra_headers_present() {
        let mut raw = HashMap::new();
        raw.insert("x-custom".to_string(), "value".to_string());

        let expected_result = Some("value");

        let result = Headers::new(raw);

        assert_eq!(result.get("x-custom"), expected_result);
    }

    #[test]
    fn should_return_none_from_get_when_key_is_not_present() {
        let raw = HashMap::new();

        let result = Headers::new(raw);

        assert_eq!(result.get("nonexistent"), None);
    }

    #[test]
    fn should_not_include_etag_in_other_when_etag_is_present() {
        let mut raw = HashMap::new();
        raw.insert("etag".to_string(), "\"abc123\"".to_string());

        let result = Headers::new(raw);

        assert!(result.other().is_empty());
    }

    #[test]
    fn should_not_include_content_type_in_other_when_content_type_is_present() {
        let mut raw = HashMap::new();
        raw.insert("content-type".to_string(), "application/json".to_string());

        let result = Headers::new(raw);

        assert!(result.other().is_empty());
    }

    #[test]
    fn should_have_empty_other_when_only_known_headers_provided() {
        let mut raw = HashMap::new();
        raw.insert("content-type".to_string(), "application/json".to_string());
        raw.insert("etag".to_string(), "\"abc\"".to_string());
        raw.insert("date".to_string(), "Thu, 01 Jan 2026".to_string());
        raw.insert("accept-encoding".to_string(), "gzip".to_string());

        let result = Headers::new(raw);

        assert!(result.other().is_empty());
    }
}
