//! # Filing Document Set Execution
//!
//! Implements [`HasResponse`] and [`Executes`] for the [`FilingDocumentSet`] kind. A request of
//! this kind fetches the XBRL instance, and then the label file if the filing has one.

use async_trait::async_trait;

use crate::shared::filing_documents::FilingDocumentResponses;
use crate::shared::filing_ref::FilingRef;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::http_client::{Executes, HasResponse, SecClient};
use crate::shared::request::implementations::sec_request::{FilingDocumentSet, SecRequest};
use crate::shared::response::implementations::sec_response::{SecResponse, XmlBody};

impl HasResponse for FilingDocumentSet {
    type Response = FilingDocumentResponses;
}

/// Fetches the instance first. A failed instance call ends the request before the label call.
/// Each file goes through [`SecClient::execute_sec_request`], so it takes one rate-limiter turn.
#[async_trait]
impl<C> Executes<FilingDocumentSet> for C
where
    C: SecClient<Response<XmlBody> = SecResponse<XmlBody>, Error = FailedSecRequest>,
{
    /// # Errors
    ///
    /// Returns a [`FailedSecRequest`] with
    /// [`ErrorReason::FailedFilingDocument`](crate::shared::http_client::implementations::sec_client::error::ErrorReason::FailedFilingDocument)
    /// that names the file whose request failed.
    async fn execute(
        &self,
        request: SecRequest<FilingDocumentSet>,
    ) -> Result<FilingDocumentResponses, FailedSecRequest> {
        let filing = request.input().filing();
        let documents = request.input().documents();
        let instance = fetch_document(self, filing, documents.instance()).await?;
        let labels = match documents.label_file() {
            Some(label_file) => Some(fetch_document(self, filing, label_file).await?),
            None => None,
        };
        Ok(FilingDocumentResponses::new(instance, labels))
    }
}

/// Fetches the file `file_name` of `filing` as one
/// [`FilingDocument`](crate::shared::request::implementations::sec_request::FilingDocument)
/// request.
///
/// # Errors
///
/// Returns a [`FailedSecRequest`] that names `file_name` if the request fails.
async fn fetch_document<C>(
    client: &C,
    filing: &FilingRef,
    file_name: &str,
) -> Result<SecResponse<XmlBody>, FailedSecRequest>
where
    C: SecClient<Response<XmlBody> = SecResponse<XmlBody>, Error = FailedSecRequest>,
{
    let request = SecRequest::builder()
        .filing_document()
        .filing_ref(filing.clone())
        .file_name(file_name)
        .build();
    client
        .execute_sec_request(request)
        .await
        .map_err(|error| error.for_file(file_name))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest;
    use crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
    use crate::shared::cik::Cik;
    use crate::shared::filing_documents::FilingDocuments;
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::http_client::Executes;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
    use crate::shared::request::implementations::sec_request::{FilingDocumentSet, SecRequest};
    use crate::shared::response::SecResponse as SecResponseTrait;
    use crate::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use crate::shared::status_code::StatusCode;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use crate::tests::fixtures::sample_rate_limiter::counting::CountingRateLimiter;
    use crate::traits::state_machine::state::State;

    const INSTANCE: &str = "alco-20260908_htm.xml";
    const LABEL_FILE: &str = "alco-20260908_lab.xml";
    const INSTANCE_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_htm.xml";
    const LABEL_FILE_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_lab.xml";
    const INSTANCE_BODY: &[u8] = b"<xbrl>instance</xbrl>";
    const LABEL_FILE_BODY: &[u8] = b"<link:linkbase>labels</link:linkbase>";

    fn document_set_request(label_file: Option<&str>) -> SecRequest<FilingDocumentSet> {
        let filing = FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenQ, false),
            NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
        );
        let documents = FilingDocuments::new(INSTANCE, label_file.map(str::to_string));
        SecRequest::builder()
            .filing_documents()
            .filing_ref(filing)
            .documents(documents)
            .build()
    }

    fn client_with_both_files() -> RoutingSecClient {
        RoutingSecClient::new()
            .route(INSTANCE_URL, 200, "text/xml", INSTANCE_BODY)
            .route(LABEL_FILE_URL, 200, "text/xml", LABEL_FILE_BODY)
    }

    #[tokio::test]
    async fn should_return_the_instance_body_when_the_filing_has_both_files() {
        let client = client_with_both_files();
        let request = document_set_request(Some(LABEL_FILE));

        let expected_result = INSTANCE_BODY.to_vec();

        let result = client
            .execute(request)
            .await
            .expect("Given both files served as text/xml, the request should always succeed")
            .instance()
            .body()
            .to_vec();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_label_file_body_when_the_filing_has_both_files() {
        let client = client_with_both_files();
        let request = document_set_request(Some(LABEL_FILE));

        let expected_result = Some(LABEL_FILE_BODY.to_vec());

        let result = client
            .execute(request)
            .await
            .expect("Given both files served as text/xml, the request should always succeed")
            .labels()
            .map(|labels| labels.body().to_vec());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_no_label_response_when_the_filing_has_no_label_file() {
        let client = client_with_both_files();
        let request = document_set_request(None);

        let expected_result = true;

        let result = client
            .execute(request)
            .await
            .expect("Given the instance served as text/xml, the request should always succeed")
            .labels()
            .is_none();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_filing_document_naming_the_instance_when_the_instance_is_not_found()
     {
        let client =
            RoutingSecClient::new().route(LABEL_FILE_URL, 200, "text/xml", LABEL_FILE_BODY);
        let request = document_set_request(Some(LABEL_FILE));

        let expected_result = Some(
            FailedSecRequest::from(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
                status_code: StatusCode::NotFound,
            }))
            .for_file(INSTANCE),
        );

        let result = client.execute(request).await.err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_skip_the_label_call_when_the_instance_call_fails() {
        let client = AlwaysRespondingSecClient::new(404, vec![], Vec::new())
            .with_rate_limiter(CountingRateLimiter::default());
        let request = document_set_request(Some(LABEL_FILE));

        let expected_result = 1;

        let _ = client.execute(request).await;
        let result = client.rate_limiter.turns.load(Ordering::SeqCst);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_wait_on_the_rate_limiter_once_for_each_file_when_the_filing_has_both_files() {
        let client = AlwaysRespondingSecClient::new(
            200,
            vec![("content-type", "text/xml".to_string())],
            INSTANCE_BODY.to_vec(),
        )
        .with_rate_limiter(CountingRateLimiter::default());
        let request = document_set_request(Some(LABEL_FILE));

        let expected_result = 2;

        let _ = client.execute(request).await;
        let result = client.rate_limiter.turns.load(Ordering::SeqCst);

        assert_eq!(result, expected_result);
    }

    const fn implements_state<S: State>() {}

    #[test]
    const fn should_implement_state_for_prepare_sec_request_of_filing_document_set() {
        implements_state::<PrepareSecRequest<FilingDocumentSet>>();
    }

    #[test]
    const fn should_implement_state_for_execute_sec_request_of_filing_document_set() {
        implements_state::<ExecuteSecRequest<FilingDocumentSet, SecClient>>();
    }
}
