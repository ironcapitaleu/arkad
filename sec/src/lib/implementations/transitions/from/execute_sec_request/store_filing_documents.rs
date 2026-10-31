//! # Transition: `ExecuteSecRequest` → `StoreFilingDocuments`
//!
//! Implements the [`TryFrom`] conversion producing a [`StoreFilingDocuments`] from an
//! [`ExecuteSecRequest`] of the [`FilingDocumentSet`] kind, the `execution_id`, and the two
//! storage repositories.
//!
//! It moves the [`Arc`](std::sync::Arc) of the responses from the output of the source state, the
//! filing from its context, and the `User-Agent` from its client. It builds the URL of each file
//! as its [`FilingDocument`] request does. It keeps no clone of the [`Arc`](std::sync::Arc). The
//! `execution_id` and the repositories belong to the state machine run, so the caller passes the
//! ones from the state machine context.
//!
//! The source state has the [`FilingDocumentSet`] kind. An executed Notes request cannot become a
//! filing documents store, so this does not compile:
//!
//! ```compile_fail,E0277
//! use sec::implementations::states::extract::execute_sec_request::ExecuteNotesRequest;
//! use sec::implementations::states::extract::store_filing_documents::StoreFilingDocuments;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use uuid::Uuid;
//!
//! fn store_as_filing_documents<I, L>(state: ExecuteNotesRequest<SecClient>, instances: I, labels: L) {
//!     let _next = StoreFilingDocuments::<I, L>::try_from((state, Uuid::nil(), instances, labels));
//! }
//! ```
//!
//! # Errors
//!
//! Returns [`TransitionError`] when the source state has no computed output
//! ([`MissingOutput`](crate::error::state_machine::transition::MissingOutput)), or when its
//! request names other documents than its context
//! ([`FailedContextConversion`](crate::error::state_machine::transition::FailedContextConversion)).

use uuid::Uuid;

use super::request_check::ensure_request_matches_input;
use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest;
use crate::implementations::states::extract::execute_sec_request::constants::STATE_NAME as EXECUTE_SEC_REQUEST;
use crate::implementations::states::extract::store_filing_documents::constants::STATE_NAME as STORE_FILING_DOCUMENTS;
use crate::implementations::states::extract::store_filing_documents::{
    StoreFilingDocuments, StoreFilingDocumentsContext, StoreFilingDocumentsInput,
};
use crate::shared::filing_documents::FilingDocumentRef;
use crate::shared::filing_ref::FilingRef;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::request::SecRequest as SecRequestTrait;
use crate::shared::request::implementations::sec_request::{
    FilingDocument, FilingDocumentSet, SecRequest,
};
use crate::shared::url::Url;

impl<I, L, C: SecClientTrait> TryFrom<(ExecuteSecRequest<FilingDocumentSet, C>, Uuid, I, L)>
    for StoreFilingDocuments<I, L>
{
    type Error = TransitionError;

    /// Builds a [`StoreFilingDocuments`] from a computed [`ExecuteSecRequest`] of the
    /// [`FilingDocumentSet`] kind, the `execution_id`, the repository of the instances, and the
    /// repository of the label files.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    ///
    /// Returns [`TransitionError::FailedContextConversion`] if the request of `state` names other
    /// documents than its context. The keys and the URLs follow the context, so a mismatch
    /// stores a file under the wrong filing.
    fn try_from(
        (state, execution_id, instance_repository, labels_repository): (
            ExecuteSecRequest<FilingDocumentSet, C>,
            Uuid,
            I,
            L,
        ),
    ) -> Result<Self, TransitionError> {
        let (input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(EXECUTE_SEC_REQUEST, STORE_FILING_DOCUMENTS)
        })?;

        ensure_request_matches_input(
            &input.sec_request,
            &context.request_input,
            "documents",
            STORE_FILING_DOCUMENTS,
        )?;

        let filing = context.request_input.filing();
        let documents = context.request_input.documents();
        let instance_url = filing_document_url(filing, documents.instance());
        let labels_url = documents
            .label_file()
            .map(|label_file| filing_document_url(filing, label_file));
        let new_input = StoreFilingDocumentsInput::new(
            output_data.response,
            filing.clone(),
            instance_url,
            labels_url,
            input.sec_client.user_agent().clone(),
        );
        let new_context =
            StoreFilingDocumentsContext::new(execution_id, instance_repository, labels_repository);

        Ok(Self::new(new_input, new_context))
    }
}

/// Returns the URL that the [`FilingDocument`] request for `file_name` of `filing` sends.
fn filing_document_url(filing: &FilingRef, file_name: &str) -> Url {
    let request =
        SecRequest::<FilingDocument>::new(FilingDocumentRef::new(filing.clone(), file_name));
    Url::from(request.inner.url().clone())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::implementations::states::extract::execute_sec_request::{
        ExecuteSecRequestContext, ExecuteSecRequestInput,
    };
    use crate::prelude::*;
    use crate::shared::cik::Cik;
    use crate::shared::filing_documents::{FilingDocumentSetRef, FilingDocuments};
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use crate::tests::fixtures::sample_write_repository::{
        FakeXbrlInstanceRepository, FakeXbrlLabelsRepository,
    };

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);

    const INSTANCE: &str = "alco-20260908_htm.xml";
    const LABEL_FILE: &str = "alco-20260908_lab.xml";
    const INSTANCE_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_htm.xml";
    const LABEL_FILE_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_lab.xml";

    type TestExecute = ExecuteSecRequest<FilingDocumentSet, RoutingSecClient>;
    type TestStore = StoreFilingDocuments<FakeXbrlInstanceRepository, FakeXbrlLabelsRepository>;

    fn filing() -> FilingRef {
        FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenQ, false),
            NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    fn document_set(instance: &str, label_file: Option<&str>) -> FilingDocumentSetRef {
        FilingDocumentSetRef::new(
            filing(),
            FilingDocuments::new(instance, label_file.map(str::to_string)),
        )
    }

    fn sec_client() -> RoutingSecClient {
        RoutingSecClient::new()
            .route(INSTANCE_URL, 200, "text/xml", b"<xbrl/>")
            .route(LABEL_FILE_URL, 200, "text/xml", b"<labels/>")
    }

    fn execute_without_output(
        request_input: FilingDocumentSetRef,
        context_input: FilingDocumentSetRef,
    ) -> TestExecute {
        ExecuteSecRequest::new(
            ExecuteSecRequestInput::new(sec_client(), SecRequest::new(request_input)),
            ExecuteSecRequestContext::new(context_input),
        )
    }

    async fn execute_with_output(
        request_input: FilingDocumentSetRef,
        context_input: FilingDocumentSetRef,
    ) -> TestExecute {
        let mut state = execute_without_output(request_input, context_input);
        state.compute_output_data_async().await.expect(
            "Given a client that serves both files as text/xml, the request should always succeed",
        );
        state
    }

    async fn store(label_file: Option<&str>) -> TestStore {
        let documents = document_set(INSTANCE, label_file);
        StoreFilingDocuments::try_from((
            execute_with_output(documents.clone(), documents).await,
            SAMPLE_EXECUTION_ID,
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        ))
        .expect("Given a source state with output, the transition should always succeed")
    }

    #[tokio::test]
    async fn should_hand_over_the_only_reference_to_the_responses_when_source_has_output() {
        let state = store(Some(LABEL_FILE)).await;

        let expected_result = Some(1);

        let result = state.input_data().responses.as_ref().map(Arc::strong_count);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_filing_from_context_when_source_has_output() {
        let state = store(Some(LABEL_FILE)).await;

        let expected_result = filing();

        let result = state.input_data().filing.clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_build_the_instance_url_from_the_instance_name_when_source_has_output() {
        let state = store(Some(LABEL_FILE)).await;

        let expected_result = INSTANCE_URL;

        let result = state.input_data().instance_url.as_str();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_build_the_labels_url_from_the_label_file_name_when_the_filing_has_one() {
        let state = store(Some(LABEL_FILE)).await;

        let expected_result = Some(LABEL_FILE_URL);

        let result = state.input_data().labels_url.as_ref().map(Url::as_str);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_set_no_labels_url_when_the_filing_has_no_label_file() {
        let state = store(None).await;

        let expected_result = None;

        let result = state.input_data().labels_url.clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_user_agent_from_sec_client_when_source_has_output() {
        let state = store(Some(LABEL_FILE)).await;

        let expected_result = UserAgent::new("Fake Routing Client fake@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");

        let result = state.input_data().user_agent.clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_execution_id_and_repositories_from_run_when_source_has_output() {
        let state = store(Some(LABEL_FILE)).await;

        let expected_result = &StoreFilingDocumentsContext::new(
            SAMPLE_EXECUTION_ID,
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        );

        let result = state.context_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_context_conversion_when_request_names_other_documents() {
        let state = execute_with_output(
            document_set(INSTANCE, Some(LABEL_FILE)),
            document_set(INSTANCE, None),
        )
        .await;

        let expected_result = Err(TransitionError::FailedContextConversion(
            transition::FailedContextConversion::new(
                EXECUTE_SEC_REQUEST,
                STORE_FILING_DOCUMENTS,
                "The request targets 'https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/', but the context names documents '0000003545-26-000047 (Instance: alco-20260908_htm.xml, Label File: None)'",
            ),
        ));

        let result = TestStore::try_from((
            state,
            SAMPLE_EXECUTION_ID,
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        ));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let documents = document_set(INSTANCE, Some(LABEL_FILE));
        let state = execute_without_output(documents.clone(), documents);

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(EXECUTE_SEC_REQUEST, STORE_FILING_DOCUMENTS),
        ));

        let result = TestStore::try_from((
            state,
            SAMPLE_EXECUTION_ID,
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        ));

        assert_eq!(result, expected_result);
    }
}
