use std::fmt::Debug;

use async_trait::async_trait;

use crate::shared::http_client::InnerClient;

/// A trait defining the behavior a high-level SecClient is expected to implement. This is used to define operations on a level where domain-specific knowledge can be used to abstract away the inner handling of HTTP requests, endpoints, etc.
#[async_trait]
pub trait SecClient: Send + Sync + Debug {
    type Inner: InnerClient;
    type Success;
    type Error;
    type Request;

    fn inner(&self) -> &Self::Inner;
    async fn execute_sec_request(
        &self,
        request: Self::Request,
    ) -> Result<Self::Success, Self::Error>;
}
