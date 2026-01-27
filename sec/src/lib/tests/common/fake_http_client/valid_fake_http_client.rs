use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::StatusCode;

use crate::shared::sec_client::traits::http_client::HttpClient;
use crate::shared::sec_request::SecRequest;
use crate::shared::sec_request::sec_request_error::SecRequestError;
use crate::shared::sec_response::{ContentType, SecResponse};

/// A fake HTTP client implementation that always returns successful responses.
///
/// This client is used for testing the [`HttpClient`] trait implementation.
#[derive(Debug, Clone)]
pub struct ValidFakeHttpClient;

#[async_trait]
impl HttpClient for ValidFakeHttpClient {
    async fn execute_request(&self, request: SecRequest) -> Result<SecResponse, SecRequestError> {
        let response_body = r#"{"filings": []}"#;
        let url = request.inner.url().clone();
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        Ok(SecResponse {
            url,
            status: StatusCode::OK,
            headers,
            content_type: ContentType::Json,
            body: response_body.to_string(),
        })
    }
}
