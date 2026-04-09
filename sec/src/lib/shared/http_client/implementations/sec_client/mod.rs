use async_trait::async_trait;

use crate::shared::http_client::InnerClient;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::reqwest_client::ReqwestClient;
use crate::shared::request::implementations::sec::SecRequest;
use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::SecResponse;

use self::error::FailedSecRequest;

pub mod error;

/// An SEC API client that connects `SecRequest` and `SecResponse`.
///
/// `SecClient` orchestrates the full request-response cycle: it takes a
/// validated `SecRequest`, executes it via the underlying HTTP client, and
/// returns a validated `SecResponse`.
#[derive(Debug, Clone, Default)]
pub struct SecClient {
    inner: ReqwestClient,
}

impl SecClient {
    /// Creates a new `SecClient` with the given `ReqwestClient`.
    #[must_use]
    pub const fn new(inner: ReqwestClient) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl SecClientTrait for SecClient {
    type Inner = ReqwestClient;
    type Request = SecRequest;
    type Response = SecResponse;
    type Error = FailedSecRequest;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    async fn execute_sec_request(
        &self,
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        let inner_request = request.into_inner();
        let inner_response = self.inner.execute_request(inner_request).await?;
        let sec_response = SecResponse::from_inner(inner_response).await?;
        Ok(sec_response)
    }
}
