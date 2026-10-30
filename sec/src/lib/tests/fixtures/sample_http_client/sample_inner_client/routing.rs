use async_trait::async_trait;

use crate::shared::http_client::traits::InnerClient;

use super::AlwaysRespondingHttpClient;

/// A fake HTTP client that answers each URL with its own fixed response.
///
/// A URL without a route gets an empty 404 response. Used for testing code that sends more than
/// one request, without a network call.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoutingHttpClient {
    /// The URLs and the responders that answer them.
    pub routes: Vec<(String, AlwaysRespondingHttpClient)>,
}

#[async_trait]
impl InnerClient for RoutingHttpClient {
    type Request = reqwest::Request;
    type Response = reqwest::Response;
    type Error = reqwest::Error;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let not_found = AlwaysRespondingHttpClient {
            status: 404,
            headers: Vec::new(),
            body: Vec::new(),
        };
        let responder = self
            .routes
            .iter()
            .find(|(url, _)| url == request.url().as_str())
            .map_or(&not_found, |(_, responder)| responder);
        responder.execute_request(request).await
    }
}
