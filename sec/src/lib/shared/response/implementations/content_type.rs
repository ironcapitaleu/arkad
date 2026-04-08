use std::collections::HashMap;
use std::fmt;

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
    Other(String), // `Content-Type` header is present but does not match the types we expect.
    Unknown,       // `Content-Type` header is absent or invalid.
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
}
