use std::collections::HashMap;
use std::fmt;

use reqwest::{Response, StatusCode, Url};

#[derive(Debug, Clone)]
pub struct SecResponse {
    pub url: Url,
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub content_type: ContentType,
    pub body: String,
}

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
        if let Some(content_type) = headers.get("content-type") {
            Self::from_str(content_type)
        } else {
            Self::Other("unknown".to_string())
        }
    }
    /// Determines the content type from a string.
    #[must_use]
    pub fn from_str(content_type_str: &str) -> Self {
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
                (
                    name.to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                )
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
                .unwrap(),
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
