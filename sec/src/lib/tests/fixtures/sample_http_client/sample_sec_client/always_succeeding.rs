use async_trait::async_trait;

use crate::shared::http_client::SecClient;
use crate::tests::fixtures::sample_http_client::sample_inner_client::AlwaysSucceedingHttpClient;

/// Fake SecClient that always returns a fixed SecResponse fixture.
#[derive(Debug)]
pub struct FakeSecClient {
    pub inner: AlwaysSucceedingHttpClient,
}

impl FakeSecClient {
    pub fn new() -> Self {
        Self {
            inner: AlwaysSucceedingHttpClient,
        }
    }
}

#[async_trait]
impl SecClient for FakeSecClient {
    type Inner = AlwaysSucceedingHttpClient;
    type Request = ();
    type Response = String;
    type Error = String;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    async fn execute_sec_request(
        &self,
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        Ok(format!(
            "Simulated success response for sec request: {:?}",
            request
        ))
    }
}
