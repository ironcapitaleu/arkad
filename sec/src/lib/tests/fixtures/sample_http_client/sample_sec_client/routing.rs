use serde::Serialize;

use crate::shared::http_client::SecClient;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::response::implementations::sec_response::{ResponseBody, SecResponse};
use crate::shared::user_agent::UserAgent;
use crate::tests::fixtures::sample_http_client::sample_inner_client::routing::RequestCounts;
use crate::tests::fixtures::sample_http_client::sample_inner_client::{
    AlwaysRespondingHttpClient, RoutingHttpClient,
};
use crate::tests::fixtures::sample_rate_limiter::always_ready::AlwaysReadyRateLimiter;

/// A fake SEC client that answers each URL with its own fixed response, or with its own list of
/// responses in order.
///
/// A URL without a route gets an empty 404 response. The rate limiter never waits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct RoutingSecClient {
    /// The inner client that routes every request.
    #[serde(skip)]
    pub inner: RoutingHttpClient,
    /// The rate limiter.
    #[serde(skip)]
    pub rate_limiter: AlwaysReadyRateLimiter,
    /// The `User-Agent` the client reports.
    pub user_agent: UserAgent,
}

impl Default for RoutingSecClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutingSecClient {
    /// Creates a [`RoutingSecClient`] with no routes.
    ///
    /// # Panics
    ///
    /// Never panics: the hardcoded `User-Agent` always passes validation.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RoutingHttpClient {
                routes: Vec::new(),
                request_counts: RequestCounts::default(),
            },
            rate_limiter: AlwaysReadyRateLimiter,
            user_agent: UserAgent::new("Fake Routing Client fake@example.com").expect(
                "Given a hardcoded SEC-format user agent, validation should always succeed",
            ),
        }
    }

    /// Adds a route that answers `url` with `status`, `content_type`, and `body`.
    #[must_use]
    pub fn route(mut self, url: &str, status: u16, content_type: &str, body: &[u8]) -> Self {
        let responder = AlwaysRespondingHttpClient {
            status,
            headers: vec![("content-type", content_type.to_string())],
            body: body.to_vec(),
        };
        self.inner.routes.push((url.to_string(), vec![responder]));
        self
    }

    /// Adds a route that answers the requests to `url` with `responses` in order. Each response
    /// is a status, a content type, and a body. Once the list ends, the last response repeats.
    #[must_use]
    pub fn route_sequence(mut self, url: &str, responses: &[(u16, &str, &[u8])]) -> Self {
        let responders = responses
            .iter()
            .map(|(status, content_type, body)| AlwaysRespondingHttpClient {
                status: *status,
                headers: vec![("content-type", (*content_type).to_string())],
                body: body.to_vec(),
            })
            .collect();
        self.inner.routes.push((url.to_string(), responders));
        self
    }
}

impl SecClient for RoutingSecClient {
    type Inner = RoutingHttpClient;
    type Limiter = AlwaysReadyRateLimiter;
    type Response<B: ResponseBody> = SecResponse<B>;
    type Error = FailedSecRequest;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn rate_limiter(&self) -> &Self::Limiter {
        &self.rate_limiter
    }

    fn user_agent(&self) -> &UserAgent {
        &self.user_agent
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::http_client::traits::InnerClient;

    const URL: &str = "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json";

    fn request() -> reqwest::Request {
        let url = reqwest::Url::parse(URL)
            .expect("Given a hardcoded valid URL, parsing should always succeed");
        reqwest::Request::new(reqwest::Method::GET, url)
    }

    fn unavailable_then_ok() -> RoutingSecClient {
        RoutingSecClient::new().route_sequence(
            URL,
            &[(503, "text/html", b""), (200, "application/json", b"{}")],
        )
    }

    async fn statuses(client: &RoutingHttpClient, requests: usize) -> Vec<u16> {
        let mut statuses = Vec::new();
        for _ in 0..requests {
            let response = client.execute_request(request()).await.expect(
                "Given a fake client that builds every response locally, the request should always succeed",
            );
            statuses.push(response.status().as_u16());
        }
        statuses
    }

    #[tokio::test]
    async fn should_answer_the_requests_in_order_when_a_url_has_a_sequence() {
        let client = unavailable_then_ok();
        let requests = 2;

        let expected_result = vec![503, 200];

        let result = statuses(&client.inner, requests).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_repeat_the_last_response_when_the_sequence_ends() {
        let client = unavailable_then_ok();
        let requests = 3;

        let expected_result = vec![503, 200, 200];

        let result = statuses(&client.inner, requests).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_share_the_request_count_when_the_client_is_cloned() {
        let client = unavailable_then_ok();
        let clone = client.clone();
        statuses(&client.inner, 1).await;

        let expected_result = vec![200];

        let result = statuses(&clone.inner, 1).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_answer_404_when_the_url_has_no_route() {
        let client = RoutingSecClient::new();
        let requests = 1;

        let expected_result = vec![404];

        let result = statuses(&client.inner, requests).await;

        assert_eq!(result, expected_result);
    }
}
