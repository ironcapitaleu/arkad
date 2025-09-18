use std::collections::HashMap;

use reqwest::{Response, StatusCode, Url};

#[derive(Debug, Clone)]
pub struct SecResponse {
    pub url: Url,
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: String
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

        let body = response.text().await?;
        
        Ok(Self {
            url,
            status,
            headers,
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
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Returns the response body as a string.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns the content type of the response, if available.
    #[must_use]
    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type").map(|s| s.as_str())
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
