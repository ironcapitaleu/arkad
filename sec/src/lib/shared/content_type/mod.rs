//! # Content Type
//!
//! The [`ContentType`] of a SEC API response, classified from its `Content-Type` header.

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use serde::Serialize;

/// The MIME content type of a SEC API response.
///
/// Models the content types relevant to SEC interactions as explicit variants so callers can
/// match on them directly instead of comparing header strings. Anything recognized-but-unmodeled
/// is captured by `Other`, and a missing or unreadable header by `Unknown`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum ContentType {
    /// `application/json`.
    Json,
    /// `application/xml` or `text/xml`.
    Xml,
    /// `text/html`.
    Html,
    /// `text/plain`.
    Text,
    /// A valid content type that is not one of the modeled variants.
    Other(String),
    /// The `Content-Type` header was absent or could not be read.
    Unknown,
}

impl ContentType {
    /// Classifies the content type from a header map, keyed by lowercase `content-type`.
    ///
    /// Returns [`ContentType::Unknown`] if the header is absent.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use sec::shared::content_type::ContentType;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), "application/json".to_string());
    ///
    /// assert_eq!(ContentType::from_headers(&headers), ContentType::Json);
    /// ```
    #[must_use]
    pub fn from_headers(headers: &HashMap<String, String>) -> Self {
        headers.get("content-type").map_or_else(
            || Self::Unknown,
            |content_type| Self::from_content_type(content_type),
        )
    }

    /// Classifies the content type from a header value string.
    ///
    /// Strips any parameters (e.g. `; charset=utf-8`) before matching the MIME type.
    #[must_use]
    pub fn from_content_type(content_type_str: &str) -> Self {
        let mime_type = content_type_str
            .split(';')
            .next()
            .unwrap_or(content_type_str)
            .trim()
            .to_lowercase();

        match mime_type.as_str() {
            "application/json" => Self::Json,
            "application/xml" | "text/xml" => Self::Xml,
            "text/html" => Self::Html,
            "text/plain" => Self::Text,
            _ => Self::Other(content_type_str.to_string()),
        }
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Json => write!(f, "application/json"),
            Self::Xml => write!(f, "application/xml"),
            Self::Html => write!(f, "text/html"),
            Self::Text => write!(f, "text/plain"),
            Self::Other(content_type) => write!(f, "{content_type}"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_json_when_headers_contain_application_json_content_type() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let expected_result = ContentType::Json;

        let result = ContentType::from_headers(&headers);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_json_when_str_slice_contains_application_json_content_type() {
        let content_type_str = "application/json";

        let expected_result = ContentType::Json;

        let result = ContentType::from_content_type(content_type_str);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_other_when_headers_contain_unexpected_content_type() {
        let mut headers = HashMap::new();
        headers.insert(
            "content-type".to_string(),
            "unexpected/content-type".to_string(),
        );

        let expected_result = ContentType::Other("unexpected/content-type".to_string());

        let result = ContentType::from_headers(&headers);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unknown_when_headers_map_is_empty() {
        let headers = HashMap::new();

        let expected_result = ContentType::Unknown;

        let result = ContentType::from_headers(&headers);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_json_when_content_type_includes_charset_parameter() {
        let content_type_str = "application/json; charset=utf-8";

        let expected_result = ContentType::Json;

        let result = ContentType::from_content_type(content_type_str);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_xml_when_content_type_is_application_xml() {
        let content_type_str = "application/xml";

        let expected_result = ContentType::Xml;

        let result = ContentType::from_content_type(content_type_str);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_xml_when_content_type_is_text_xml() {
        let content_type_str = "text/xml";

        let expected_result = ContentType::Xml;

        let result = ContentType::from_content_type(content_type_str);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_html_when_content_type_is_text_html() {
        let content_type_str = "text/html; charset=ISO-8859-1";

        let expected_result = ContentType::Html;

        let result = ContentType::from_content_type(content_type_str);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_text_when_content_type_is_text_plain() {
        let content_type_str = "text/plain";

        let expected_result = ContentType::Text;

        let result = ContentType::from_content_type(content_type_str);

        assert_eq!(result, expected_result);
    }
}
