use std::fmt::Debug;
use std::hash::Hash;

use async_trait::async_trait;
use serde::Serialize;

use crate::shared::http_client::InnerClient;
use crate::shared::rate_limiter::RateLimiter;
use crate::shared::request::SecRequest as SecRequestTrait;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::ResponseBody;
use crate::shared::user_agent::UserAgent;

use super::RespondsWith;

/// The domain-level SEC HTTP client: executes an SEC request and returns an SEC response.
///
/// Sits above [`InnerClient`], adding the SEC-specific knowledge (endpoints, request/response
/// shaping) that the raw HTTP client lacks. A client is a value, so it fits into a state context.
///
/// # Associated Types
///
/// Each implementor chooses the concrete types filling these slots, which is what keeps the trait
/// decoupled from any specific HTTP crate:
///
/// - `Inner`: The raw HTTP client this trait delegates to. It sends the inner request of an
///   [`SecRequest`].
/// - `Limiter`: The rate limiter pacing outgoing requests. Must implement [`RateLimiter`].
/// - `Response<B>`: The SEC response with a body of kind `B`. It builds from the response of
///   `Inner`.
/// - `Error`: The error type returned when request execution or response validation fails.
#[async_trait]
pub trait SecClient: Debug + Clone + Eq + Ord + Hash + Send + Sync + Unpin + Serialize {
    /// The raw HTTP client this trait delegates to.
    ///
    /// The bound names the inner request of the default endpoint. Every endpoint builds the same
    /// inner request type, so the default body can send any `SecRequest<E>`.
    type Inner: InnerClient<Request = <SecRequest as SecRequestTrait>::Inner, Error: Into<Self::Error>>;
    /// The rate limiter pacing outgoing requests. Must implement [`RateLimiter`].
    type Limiter: RateLimiter;
    /// The SEC response with a body of kind `B`.
    type Response<B: ResponseBody>: SecResponseTrait<
            Inner = <Self::Inner as InnerClient>::Response,
            Body = B::Content,
            Error: Into<Self::Error>,
        >;
    /// The error type returned when request execution or response validation fails.
    type Error: Send;

    /// Returns a reference to the underlying raw HTTP client.
    fn inner(&self) -> &Self::Inner;

    /// Returns a reference to the rate limiter pacing outgoing requests.
    fn rate_limiter(&self) -> &Self::Limiter;

    /// Returns the [`UserAgent`] that every request carries.
    fn user_agent(&self) -> &UserAgent;

    /// Executes an SEC request to the endpoint `E`, for example `CompanyFacts`, and returns a
    /// response with the body kind of `E`. See [`RespondsWith`] for how `E` sets the response type.
    ///
    /// Waits on the rate limiter, sends the request through [`Self::inner`], and builds the
    /// response with [`SecResponseTrait::from_inner`].
    ///
    /// A Notes request does not give a JSON response:
    ///
    /// ```compile_fail,E0308
    /// use sec::shared::http_client::SecClient as SecClientTrait;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::http_client::implementations::sec_client::error::FailedSecRequest;
    /// use sec::shared::request::implementations::sec_request::{NotesFile, SecRequest};
    /// use sec::shared::response::implementations::sec_response::{JsonBody, SecResponse};
    ///
    /// async fn execute_notes_request(
    ///     client: &SecClient,
    ///     request: SecRequest<NotesFile>,
    /// ) -> Result<SecResponse<JsonBody>, FailedSecRequest> {
    ///     client.execute_sec_request(request).await
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the request fails at the network level or the response fails
    /// SEC validation.
    async fn execute_sec_request<E>(
        &self,
        request: SecRequest<E>,
    ) -> Result<Self::Response<E::Body>, Self::Error>
    where
        E: RespondsWith,
    {
        self.rate_limiter().await_turn().await;

        let inner_response = self
            .inner()
            .execute_request(request.into_inner())
            .await
            .map_err(Into::into)?;
        <Self::Response<E::Body> as SecResponseTrait>::from_inner(inner_response)
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use pretty_assertions::assert_eq;

    use crate::shared::cik::Cik;
    use crate::shared::content_type::ContentType;
    use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
    use crate::shared::http_client::traits::SecClient;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::implementations::sec_request::{
        CompanyFacts, NotesFile, SecRequest,
    };
    use crate::shared::response::SecResponse as SecResponseTrait;
    use crate::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use crate::shared::status_code::StatusCode;
    use crate::tests::fixtures::sample_http_client::sample_inner_client::AlwaysReturningHttpClient;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_succeeding::{
        FakeSecClient, fake_json_inner_response,
    };
    use crate::tests::fixtures::sample_rate_limiter::counting::CountingRateLimiter;

    #[tokio::test]
    async fn should_return_the_fake_json_body_when_the_fake_sec_client_executes_a_company_facts_request()
     {
        let client = FakeSecClient::new();
        let request = company_facts_request();

        let expected_result = serde_json::json!({ "key": "value" });

        let result = client
            .execute_sec_request(request)
            .await
            .expect("Given a fake inner response with a JSON body, the fake response should always build")
            .body()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_inner_error_when_the_inner_client_fails() {
        let client = FakeSecClient::failing("Simulated network error");
        let request = company_facts_request();

        let expected_result = Some(String::from("Simulated network error"));

        let result = client.execute_sec_request(request).await.err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_expected_inner_client_for_fake_sec_client() {
        let fake_client = FakeSecClient::new();
        let expected_inner_client = AlwaysReturningHttpClient {
            result: Ok(fake_json_inner_response()),
        };

        let expected_result = &expected_inner_client;

        let result = fake_client.inner();

        assert_eq!(result, expected_result);
    }

    const NOTES_FILE_BODY: &[u8] = b"PK\x03\x04 notes";

    fn company_facts_request() -> SecRequest<CompanyFacts> {
        let cik = Cik::new("1067983").expect(
            "Given a hardcoded valid CIK, the creation of a CIK object should always succeed",
        );
        SecRequest::builder().all_company_facts().cik(cik).build()
    }

    fn notes_file_request() -> SecRequest<NotesFile> {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed");
        SecRequest::builder().notes_file().period(period).build()
    }

    fn responding_client(
        status: u16,
        content_type: &str,
        body: &[u8],
    ) -> AlwaysRespondingSecClient<CountingRateLimiter> {
        AlwaysRespondingSecClient::new(
            status,
            vec![
                ("content-type", content_type.to_string()),
                ("content-length", NOTES_FILE_BODY.len().to_string()),
            ],
            body.to_vec(),
        )
        .with_rate_limiter(CountingRateLimiter::default())
    }

    fn invalid_response(reason: ErrorReason) -> FailedSecRequest {
        FailedSecRequest::from(InvalidSecResponse::new(reason))
    }

    #[tokio::test]
    async fn should_return_the_body_when_the_notes_file_download_succeeds() {
        let client = responding_client(200, "application/octet-stream", NOTES_FILE_BODY);

        let expected_result = NOTES_FILE_BODY.to_vec();

        let result = client.execute_sec_request(notes_file_request())
            .await
            .expect("Given status 200, an octet stream and a matching length, the download should always succeed")
            .body()
            .to_vec();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_wait_on_the_rate_limiter_once_when_downloading_a_notes_file() {
        let client = responding_client(200, "application/zip", NOTES_FILE_BODY);

        let expected_result = 1;

        let _ = client.execute_sec_request(notes_file_request()).await;
        let result = client.rate_limiter.turns.load(Ordering::SeqCst);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_status_code_when_the_notes_file_is_not_found() {
        let client = responding_client(404, "application/octet-stream", NOTES_FILE_BODY);

        let expected_result = Some(invalid_response(ErrorReason::InvalidStatusCode {
            status_code: StatusCode::NotFound,
        }));

        let result = client.execute_sec_request(notes_file_request()).await.err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_content_type_when_the_notes_file_is_html() {
        let client = responding_client(200, "text/html", NOTES_FILE_BODY);

        let expected_result = Some(invalid_response(ErrorReason::InvalidContentType {
            content_type: ContentType::Html,
        }));

        let result = client.execute_sec_request(notes_file_request()).await.err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_content_length_when_the_notes_file_body_is_short() {
        let client = responding_client(200, "application/zip", &NOTES_FILE_BODY[..4]);

        let expected_result = Some(invalid_response(ErrorReason::InvalidContentLength {
            content_length: NOTES_FILE_BODY.len().to_string(),
            body_length: 4,
        }));

        let result = client.execute_sec_request(notes_file_request()).await.err();

        assert_eq!(result, expected_result);
    }
}
