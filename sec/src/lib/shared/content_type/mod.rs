use std::collections::HashMap;
use std::fmt;

/// Content type enum with automatic detection from HTTP headers.
///
/// `ContentType` represents the main content types expected from SEC API responses.
/// It provides methods to detect content type from response headers and format
/// content type strings for use in HTTP headers.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use sec::shared::content_type::ContentType;
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
    /// Content type is JSON (e.g., `application/json`).
    Json,
    /// Content type is XML (e.g., `application/xml` or `text/xml`).
    Xml,
    /// Content type is HTML (e.g., `text/html`).
    Html,
    /// Content type is plain text (e.g., `text/plain`).
    Text,
    /// Content type is present but does not match the types we expect.
    Other(String),
    /// Content type is unknown (e.g., `Content-Type` header is absent or invalid).
    Unknown,
}

impl ContentType {
    /// Determines the content type from response headers.
    #[must_use]
    pub fn from_headers(headers: &HashMap<String, String>) -> Self {
        headers.get("content-type").map_or_else(
            || Self::Unknown,
            |content_type| Self::from_content_type(content_type),
        )
    }

    /// Determines the content type from a string.
    ///
    /// Strips any parameters (e.g., `; charset=utf-8`) before matching the MIME type.
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

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
