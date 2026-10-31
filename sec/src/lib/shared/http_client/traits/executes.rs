//! # Execution of a Request Kind
//!
//! Provides the [`Executes`] trait, which a client implements for each request kind it runs.

use async_trait::async_trait;

use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::response::implementations::sec_response::SecResponse;

use super::{HasResponse, RespondsWith, SecClient};

/// A client that runs a request of kind `K` and returns its [`HasResponse::Response`].
///
/// The impl for a kind decides how many HTTP calls the request takes. A kind with its own impl
/// must not implement [`RespondsWith`], or its impl overlaps the blanket impl for `RespondsWith`
/// kinds.
#[async_trait]
pub trait Executes<K: HasResponse>: Send + Sync {
    /// Runs `request` and returns the response of its kind.
    ///
    /// # Errors
    ///
    /// Returns a [`FailedSecRequest`] if an HTTP call fails or a response fails SEC validation.
    async fn execute(&self, request: SecRequest<K>) -> Result<K::Response, FailedSecRequest>;
}

/// A kind that makes one HTTP call runs through [`SecClient::execute_sec_request`], so it takes
/// one rate-limiter turn.
#[async_trait]
impl<C, K> Executes<K> for C
where
    K: RespondsWith + 'static,
    C: SecClient<Response<K::Body> = SecResponse<K::Body>, Error = FailedSecRequest>,
{
    async fn execute(
        &self,
        request: SecRequest<K>,
    ) -> Result<SecResponse<K::Body>, FailedSecRequest> {
        self.execute_sec_request(request).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::shared::cik::Cik;
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::request::implementations::sec_request::{FilingDocument, SecRequest};
    use crate::shared::response::SecResponse as SecResponseTrait;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_rate_limiter::counting::CountingRateLimiter;

    use super::Executes;

    const INSTANCE_BODY: &[u8] = b"<xbrl></xbrl>";

    fn instance_request() -> SecRequest<FilingDocument> {
        let filing = FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenQ, false),
            NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
        );
        SecRequest::builder()
            .filing_document()
            .filing_ref(filing)
            .file_name("alco-20260908_htm.xml")
            .build()
    }

    fn xml_client() -> AlwaysRespondingSecClient<CountingRateLimiter> {
        AlwaysRespondingSecClient::new(
            200,
            vec![("content-type", "text/xml".to_string())],
            INSTANCE_BODY.to_vec(),
        )
        .with_rate_limiter(CountingRateLimiter::default())
    }

    #[tokio::test]
    async fn should_return_the_body_when_a_client_executes_a_filing_document_request() {
        let client = xml_client();
        let request = instance_request();

        let expected_result = INSTANCE_BODY.to_vec();

        let result = client
            .execute(request)
            .await
            .expect("Given status 200 and text/xml, the request should always succeed")
            .body()
            .to_vec();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_wait_on_the_rate_limiter_once_when_a_client_executes_a_one_call_request() {
        let client = xml_client();
        let request = instance_request();

        let expected_result = 1;

        let _ = client.execute(request).await;
        let result = client.rate_limiter.turns.load(Ordering::SeqCst);

        assert_eq!(result, expected_result);
    }
}
