use std::fmt::Debug;

use async_trait::async_trait;

pub mod inner;

pub use inner::InnerClient;

/// A trait defining the behavior an HTTP client is expected to implement. This is used to abstract away the specifics of any (third party) HTTP client that might be used.
#[async_trait]
pub trait HttpClient: Send + Sync + Debug {
    type Inner: InnerClient;
    type Success;
    type Error;

    fn inner(&self) -> &Self::Inner;
    async fn execute_request(&self) -> Result<Self::Success, Self::Error>;
}
