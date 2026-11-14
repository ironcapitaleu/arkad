//! # Retrying SEC Client
//!
//! Provides the [`RetryingSecClient`], which sends a request again after a transient failure, and
//! the [`RetryPolicy`] that sets its attempts and its waits.
//!
//! `docs/adr/0011-request-retry-policy.md` records the policy.

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
    /// `[0, 1]` picks the point.
    #[must_use]
    pub fn jitter(backoff: Duration, fraction: f64) -> Duration {
        let half = backoff / 2;
        half + half.mul_f64(fraction.clamp(0.0, 1.0))
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
}

impl<C> RetryingSecClient<C> {
    /// Creates a [`RetryingSecClient`] that sends through `client` under `policy`.
    #[must_use]
    pub const fn new(client: C, policy: RetryPolicy) -> Self {
        Self { client, policy }
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
                    let wait = RetryPolicy::jitter(self.policy.backoff(retry), fastrand::f64());
                    tokio::time::sleep(wait).await;
                }
                result => return result,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use pretty_assertions::assert_eq;

    use super::{RetryPolicy, RetryingSecClient};
    use crate::shared::cik::Cik;
    use crate::shared::http_client::SecClient;
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

    fn retrying_client(
        status: u16,
        content_type: &str,
        body: &[u8],
    ) -> RetryingSecClient<AlwaysRespondingSecClient<CountingRateLimiter>> {
        let client = AlwaysRespondingSecClient::new(
            status,
            vec![("content-type", content_type.to_string())],
            body.to_vec(),
        )
        .with_rate_limiter(CountingRateLimiter::default());
        RetryingSecClient::new(client, NO_WAIT_POLICY)
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
            retrying_client(503, "text/html", b"").client().clone(),
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
}
