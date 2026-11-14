//! # Retrying SEC Client
//!
//! Provides the [`RetryingSecClient`], which sends a request again after a transient failure, and
//! the [`RetryPolicy`] that sets its attempts and its waits.
//!
//! `docs/adr/0011-request-retry-policy.md` records the policy.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use async_trait::async_trait;
use serde::Serialize;

use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::http_client::{RespondsWith, SecClient};
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::response::implementations::sec_response::ResponseBody;
use crate::shared::user_agent::UserAgent;

/// The attempts and the waits of a [`RetryingSecClient`].
///
/// The wait before retry `n` is `first_wait * factor^(n - 1)`, at most `max_wait`. Jitter then
/// picks a wait between half of that value and the whole value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct RetryPolicy {
    /// The number of sends of one request, the first included. A value of 0 sends once.
    pub max_attempts: u32,
    /// The wait before the first retry, before jitter.
    pub first_wait: Duration,
    /// The factor by which each wait grows.
    pub factor: u32,
    /// The longest wait before jitter.
    pub max_wait: Duration,
}

impl Default for RetryPolicy {
    /// Returns 3 attempts, with waits of 1 s and 2 s before jitter, and a cap of 4 s.
    fn default() -> Self {
        Self {
            max_attempts: 3,
            first_wait: Duration::from_secs(1),
            factor: 2,
            max_wait: Duration::from_secs(4),
        }
    }
}

impl RetryPolicy {
    /// Returns the wait before retry `retry`, counted from 1, before jitter.
    #[must_use]
    pub fn backoff(&self, retry: u32) -> Duration {
        let growth = self.factor.saturating_pow(retry.saturating_sub(1));
        self.first_wait.saturating_mul(growth).min(self.max_wait)
    }

    /// Returns a wait between half of `backoff` and the whole `backoff`, where `fraction` in
    /// `[0, 1]` picks the point. A `fraction` of NaN counts as 0.
    #[must_use]
    pub fn jitter(backoff: Duration, fraction: f64) -> Duration {
        let fraction = if fraction.is_nan() {
            0.0
        } else {
            fraction.clamp(0.0, 1.0)
        };
        let half = backoff / 2;
        half + half.mul_f64(fraction)
    }
}

/// An SEC client that sends a request again after a transient failure.
///
/// Delegates each attempt to the client `C`, so each attempt waits for its own permit of the rate
/// limiter. Retries only a failure for which [`FailedSecRequest::is_transient`] returns `true`, and
/// returns the last error after [`RetryPolicy::max_attempts`].
///
/// Every request kind runs through [`SecClient::execute_sec_request`], so a request of several
/// HTTP calls, such as a filing document set, retries each call on its own.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct RetryingSecClient<C> {
    client: C,
    policy: RetryPolicy,
    #[serde(skip)]
    on_retry: OnRetry,
}

/// The function that a [`RetryingSecClient`] calls with the error and the number of each retry,
/// before the wait.
#[derive(Clone, Copy)]
struct OnRetry(fn(&FailedSecRequest, u32));

impl<C> RetryingSecClient<C> {
    /// Creates a [`RetryingSecClient`] that sends through `client` under `policy`.
    #[must_use]
    pub const fn new(client: C, policy: RetryPolicy) -> Self {
        Self {
            client,
            policy,
            on_retry: OnRetry(|_, _| {}),
        }
    }

    /// Returns this client, which calls `on_retry` with the error and the number of each retry,
    /// counted from 1, before the wait. A binary passes a function that logs the retry.
    #[must_use]
    pub const fn with_on_retry(mut self, on_retry: fn(&FailedSecRequest, u32)) -> Self {
        self.on_retry = OnRetry(on_retry);
        self
    }

    /// Returns the client that sends each attempt.
    #[must_use]
    pub const fn client(&self) -> &C {
        &self.client
    }

    /// Returns the policy of the retries.
    #[must_use]
    pub const fn policy(&self) -> &RetryPolicy {
        &self.policy
    }
}

#[async_trait]
impl<C> SecClient for RetryingSecClient<C>
where
    C: SecClient<Error = FailedSecRequest>,
{
    type Inner = C::Inner;
    type Limiter = C::Limiter;
    type Response<B: ResponseBody> = C::Response<B>;
    type Error = FailedSecRequest;

    fn inner(&self) -> &Self::Inner {
        self.client.inner()
    }

    fn rate_limiter(&self) -> &Self::Limiter {
        self.client.rate_limiter()
    }

    fn user_agent(&self) -> &UserAgent {
        self.client.user_agent()
    }

    /// # Errors
    ///
    /// Returns the first permanent [`FailedSecRequest`], or the last transient one after
    /// [`RetryPolicy::max_attempts`].
    async fn execute_sec_request<E>(
        &self,
        request: SecRequest<E>,
    ) -> Result<Self::Response<E::Body>, Self::Error>
    where
        E: RespondsWith,
    {
        let mut retry = 0;
        loop {
            match self.client.execute_sec_request(request.clone()).await {
                Err(error) if error.is_transient() && retry + 1 < self.policy.max_attempts => {
                    retry += 1;
                    (self.on_retry.0)(&error, retry);
                    let wait = RetryPolicy::jitter(self.policy.backoff(retry), fastrand::f64());
                    tokio::time::sleep(wait).await;
                }
                result => return result,
            }
        }
    }
}

// Deviation: a client needs `Eq`, `Ord`, and `Hash` to implement `SecClient`, and a function
// pointer gives no stable order. Every `OnRetry` compares equal and adds nothing to a hash. Same
// rule as `CountingRateLimiter`.

impl fmt::Debug for OnRetry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("OnRetry")
    }
}

impl PartialEq for OnRetry {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for OnRetry {}

impl PartialOrd for OnRetry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OnRetry {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl Hash for OnRetry {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use pretty_assertions::assert_eq;

    use super::{RetryPolicy, RetryingSecClient};
    use crate::shared::cik::Cik;
    use crate::shared::http_client::SecClient;
    use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
    use crate::shared::request::implementations::sec_request::{CompanyFacts, SecRequest};
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_rate_limiter::counting::CountingRateLimiter;

    /// A policy of 3 attempts without waits.
    const NO_WAIT_POLICY: RetryPolicy = RetryPolicy {
        max_attempts: 3,
        first_wait: Duration::ZERO,
        factor: 2,
        max_wait: Duration::ZERO,
    };

    fn company_facts_request() -> SecRequest<CompanyFacts> {
        let cik = Cik::new("1067983").expect(
            "Given a hardcoded valid CIK, the creation of a CIK object should always succeed",
        );
        SecRequest::builder().all_company_facts().cik(cik).build()
    }

    fn responding_client(
        status: u16,
        content_type: &str,
        body: &[u8],
    ) -> AlwaysRespondingSecClient<CountingRateLimiter> {
        AlwaysRespondingSecClient::new(
            status,
            vec![("content-type", content_type.to_string())],
            body.to_vec(),
        )
        .with_rate_limiter(CountingRateLimiter::default())
    }

    fn retrying_client(
        status: u16,
        content_type: &str,
        body: &[u8],
    ) -> RetryingSecClient<AlwaysRespondingSecClient<CountingRateLimiter>> {
        RetryingSecClient::new(
            responding_client(status, content_type, body),
            NO_WAIT_POLICY,
        )
    }

    /// Sends one request through `client`. Returns whether it failed, and the number of permits it
    /// took, so a test of the permits also shows that the request failed.
    async fn permits_of_one_request(
        client: &RetryingSecClient<AlwaysRespondingSecClient<CountingRateLimiter>>,
    ) -> (bool, usize) {
        let failed = client
            .execute_sec_request(company_facts_request())
            .await
            .is_err();
        (failed, client.rate_limiter().turns.load(Ordering::SeqCst))
    }

    #[tokio::test]
    async fn should_take_one_permit_per_attempt_when_every_attempt_gets_a_503() {
        let client = retrying_client(503, "text/html", b"");

        let expected_result = (true, 3);

        let result = permits_of_one_request(&client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_send_once_when_the_first_attempt_succeeds() {
        let client = retrying_client(200, "application/json", b"{}");

        let expected_result = (false, 1);

        let result = permits_of_one_request(&client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_send_once_when_the_sec_answers_403() {
        let client = retrying_client(403, "text/html", b"");

        let expected_result = (true, 1);

        let result = permits_of_one_request(&client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_send_once_when_the_sec_answers_404() {
        let client = retrying_client(404, "text/html", b"");

        let expected_result = (true, 1);

        let result = permits_of_one_request(&client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_send_once_when_the_body_fails_to_parse() {
        let client = retrying_client(200, "application/json", b"{ not json");

        let expected_result = (true, 1);

        let result = permits_of_one_request(&client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_send_once_when_the_content_type_is_invalid() {
        let client = retrying_client(200, "text/html", b"{}");

        let expected_result = (true, 1);

        let result = permits_of_one_request(&client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_send_once_when_the_policy_allows_zero_attempts() {
        let client = RetryingSecClient::new(
            responding_client(503, "text/html", b""),
            RetryPolicy {
                max_attempts: 0,
                ..NO_WAIT_POLICY
            },
        );

        let expected_result = (true, 1);

        let result = permits_of_one_request(&client).await;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_double_the_wait_for_each_retry_of_the_default_policy() {
        let policy = RetryPolicy::default();

        let expected_result = vec![
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(4),
        ];

        let result: Vec<Duration> = (1..=3).map(|retry| policy.backoff(retry)).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cap_the_wait_when_the_backoff_passes_the_max_wait() {
        let policy = RetryPolicy::default();

        let expected_result = Duration::from_secs(4);

        let result = policy.backoff(10);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cap_the_wait_when_the_growth_overflows() {
        let policy = RetryPolicy::default();

        let expected_result = Duration::from_secs(4);

        let result = policy.backoff(u32::MAX);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_wait_half_the_backoff_when_the_jitter_fraction_is_zero() {
        let backoff = Duration::from_secs(2);

        let expected_result = Duration::from_secs(1);

        let result = RetryPolicy::jitter(backoff, 0.0);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_wait_the_whole_backoff_when_the_jitter_fraction_is_one() {
        let backoff = Duration::from_secs(2);

        let expected_result = Duration::from_secs(2);

        let result = RetryPolicy::jitter(backoff, 1.0);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_wait_within_the_backoff_when_the_jitter_fraction_is_out_of_range() {
        let backoff = Duration::from_secs(2);

        let expected_result = Duration::from_secs(2);

        let result = RetryPolicy::jitter(backoff, 7.5);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_wait_half_the_backoff_when_the_jitter_fraction_is_nan() {
        let backoff = Duration::from_secs(2);

        let expected_result = Duration::from_secs(1);

        let result = RetryPolicy::jitter(backoff, f64::NAN);

        assert_eq!(result, expected_result);
    }

    #[tokio::test(start_paused = true)]
    async fn should_wait_between_half_and_all_of_each_backoff_when_every_attempt_gets_a_503() {
        let client = RetryingSecClient::new(
            responding_client(503, "text/html", b""),
            RetryPolicy::default(),
        );
        let start = tokio::time::Instant::now();

        let expected_result = true;

        let _ = client.execute_sec_request(company_facts_request()).await;
        let elapsed = start.elapsed();
        let result = (Duration::from_millis(1500)..=Duration::from_secs(3)).contains(&elapsed);

        assert_eq!(result, expected_result);
    }

    /// The retry numbers that [`record_retry`] received. Only one test uses it.
    static RECORDED_RETRIES: Mutex<Vec<u32>> = Mutex::new(Vec::new());

    fn record_retry(_error: &FailedSecRequest, retry: u32) {
        RECORDED_RETRIES
            .lock()
            .expect("Given a lock that no thread poisoned, locking should always succeed")
            .push(retry);
    }

    #[tokio::test]
    async fn should_call_on_retry_with_each_retry_number_when_every_attempt_gets_a_503() {
        let client = retrying_client(503, "text/html", b"").with_on_retry(record_retry);

        let expected_result = vec![1, 2];

        let _ = client.execute_sec_request(company_facts_request()).await;
        let result = RECORDED_RETRIES
            .lock()
            .expect("Given a lock that no thread poisoned, locking should always succeed")
            .clone();

        assert_eq!(result, expected_result);
    }
}
