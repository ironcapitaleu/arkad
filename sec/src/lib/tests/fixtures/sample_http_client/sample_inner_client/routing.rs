use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::shared::http_client::traits::InnerClient;

use super::AlwaysRespondingHttpClient;

/// A fake HTTP client that answers each URL with its own list of responses.
///
/// The first request to a URL gets the first response of its list, the second request the second
/// response, and so on. Once the list ends, the last response repeats. A URL without a route gets
/// an empty 404 response. Clones share the count of requests. Used for testing code that sends
/// more than one request, without a network call.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoutingHttpClient {
    /// The URLs and the responses that answer them, in order.
    pub routes: Vec<(String, Vec<AlwaysRespondingHttpClient>)>,
    /// The number of requests each URL has received.
    pub request_counts: RequestCounts,
}

#[async_trait]
impl InnerClient for RoutingHttpClient {
    type Request = reqwest::Request;
    type Response = reqwest::Response;
    type Error = reqwest::Error;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let url = request.url().as_str();
        let request_count = self.request_counts.increment(url);
        let not_found = AlwaysRespondingHttpClient {
            status: 404,
            headers: Vec::new(),
            body: Vec::new(),
        };
        let responder = self
            .routes
            .iter()
            .find(|(route, _)| route == url)
            .and_then(|(_, responses)| responses.get(request_count).or_else(|| responses.last()))
            .unwrap_or(&not_found);
        responder.execute_request(request).await
    }
}

/// The number of requests each URL has received, shared by all clones of a client.
///
/// Deviation: [`SecClient`](crate::shared::http_client::SecClient) requires `Eq`, `Ord`, and `Hash`
/// of its inner client. A count changes while a test runs, so two counts always compare equal and
/// add nothing to a hash.
#[derive(Debug, Clone, Default)]
pub struct RequestCounts(Arc<Mutex<HashMap<String, usize>>>);

impl RequestCounts {
    /// Returns the number of earlier requests to `url`, and counts this one.
    ///
    /// # Panics
    ///
    /// Panics if another thread panicked while it held the lock.
    #[must_use]
    pub fn increment(&self, url: &str) -> usize {
        let mut counts = self
            .0
            .lock()
            .expect("Given a lock that no thread poisoned, locking should always succeed");
        let count = counts.entry(url.to_string()).or_insert(0);
        let earlier = *count;
        *count += 1;
        drop(counts);
        earlier
    }
}

impl PartialEq for RequestCounts {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for RequestCounts {}

impl PartialOrd for RequestCounts {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RequestCounts {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl Hash for RequestCounts {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const URL: &str = "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json";
    const OTHER_URL: &str = "https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json";

    #[test]
    fn should_return_the_number_of_earlier_requests_when_a_url_is_counted() {
        let counts = RequestCounts::default();
        let _ = counts.increment(URL);
        let _ = counts.increment(URL);

        let expected_result = 2;

        let result = counts.increment(URL);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_start_at_zero_when_another_url_was_counted() {
        let counts = RequestCounts::default();
        let _ = counts.increment(URL);

        let expected_result = 0;

        let result = counts.increment(OTHER_URL);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_compare_equal_when_the_counts_differ() {
        let counts = RequestCounts::default();
        let other_counts = RequestCounts::default();
        let _ = counts.increment(URL);

        let expected_result = true;

        let result = counts == other_counts;

        assert_eq!(result, expected_result);
    }
}
