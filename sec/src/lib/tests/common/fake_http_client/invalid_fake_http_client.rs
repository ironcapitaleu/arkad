use async_trait::async_trait;

use crate::shared::sec_client::traits::http_client::HttpClient;
use crate::shared::sec_request::SecRequest;
use crate::shared::sec_request::sec_request_error::{SecRequestError, SecRequestErrorReason};
use crate::shared::sec_response::SecResponse;

/// A fake HTTP client implementation that always returns error responses.
/// This client is used for testing error handling in the [`HttpClient`] trait implementation.
#[derive(Debug, Clone)]
pub struct InvalidFakeHttpClient;

#[async_trait]
impl HttpClient for InvalidFakeHttpClient {
    async fn execute_request(&self, request: SecRequest) -> Result<SecResponse, SecRequestError> {
        Err(SecRequestError::new(
            SecRequestErrorReason::NetworkError(format!(
                "Failed to execute request to: {}",
                request.inner.url()
            )),
        ))
    }
}
