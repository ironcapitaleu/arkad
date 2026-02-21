use std::fmt::Debug;

use async_trait::async_trait;

/// A trait defining the methods an inner http client is expected to implement. This is used to enforce a consistent interface for any (third party) HTTP client that might be used.
#[async_trait]
pub trait InnerClient: Send + Sync + Debug + Clone {
    type Success;
    type Error;
    type Request;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Success, Self::Error>;
}
