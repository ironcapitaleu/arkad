//! # Transition: `ExecuteSecRequest` → `SelectFilingDocuments`
//!
//! Implements the [`TryFrom`] conversion producing a [`SelectFilingDocuments`] from an
//! [`ExecuteSecRequest`] of the [`FilingIndex`] kind and the `execution_id`.
//!
//! It parses the response into a [`FolderIndex`] and takes the accession number from the filing
//! in the context. The caller passes the `execution_id` of the state machine run.
//!
//! # Errors
//!
//! Returns [`TransitionError`] when the source state has no computed output
//! ([`MissingOutput`](crate::error::state_machine::transition::MissingOutput)), when its request
//! names another filing than its context
//! ([`FailedContextConversion`](crate::error::state_machine::transition::FailedContextConversion)),
//! or when the response does not parse into a [`FolderIndex`]
//! ([`FailedOutputConversion`](crate::error::state_machine::transition::FailedOutputConversion)).

use uuid::Uuid;

use super::request_check::ensure_request_matches_input;
use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest;
use crate::implementations::states::extract::execute_sec_request::constants::STATE_NAME as EXECUTE_SEC_REQUEST;
use crate::implementations::states::extract::select_filing_documents::constants::STATE_NAME as SELECT_FILING_DOCUMENTS;
use crate::implementations::states::extract::select_filing_documents::{
    SelectFilingDocuments, SelectFilingDocumentsContext, SelectFilingDocumentsInput,
};
use crate::shared::filing_documents::FolderIndex;
use crate::shared::request::implementations::sec_request::FilingIndex;

impl<C> TryFrom<(ExecuteSecRequest<FilingIndex, C>, Uuid)> for SelectFilingDocuments {
    type Error = TransitionError;

    /// Builds a [`SelectFilingDocuments`] from a computed [`ExecuteSecRequest`] of the
    /// [`FilingIndex`] kind and the `execution_id`.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    ///
    /// Returns [`TransitionError::FailedContextConversion`] if the request of `state` names another
    /// filing than its context.
    ///
    /// Returns [`TransitionError::FailedOutputConversion`] if the response does not parse into a
    /// [`FolderIndex`]. The reason is the text of the
    /// [`InvalidFolderIndex`](crate::shared::filing_documents::InvalidFolderIndex), which names
    /// the URL of the folder index.
    fn try_from(
        (state, execution_id): (ExecuteSecRequest<FilingIndex, C>, Uuid),
    ) -> Result<Self, TransitionError> {
        let (input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(EXECUTE_SEC_REQUEST, SELECT_FILING_DOCUMENTS)
        })?;

        ensure_request_matches_input(
            &input.sec_request,
            &context.request_input,
            SELECT_FILING_DOCUMENTS,
        )?;

        // Deviation: the reason holds the text of the inner error, so the chain nests one
        // `Reason:` in another, not `Caused by:`. The lead decided this form in STA-371, and
        // `FailedOutputConversion` carries no source.
        let folder_index = FolderIndex::try_from(output_data.response()).map_err(|e| {
            transition::FailedOutputConversion::new(
                EXECUTE_SEC_REQUEST,
                SELECT_FILING_DOCUMENTS,
                e.to_string(),
            )
        })?;
        let accession_number = context.request_input.accession_number().clone();

        Ok(Self::new(
            SelectFilingDocumentsInput::new(accession_number, folder_index),
            SelectFilingDocumentsContext::new(execution_id),
        ))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::implementations::states::extract::execute_sec_request::{
        ExecuteSecRequestContext, ExecuteSecRequestInput,
    };
    use crate::prelude::*;
    use crate::shared::cik::Cik;
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::request::SecRequest as SecRequestTrait;
    use crate::shared::request::implementations::sec_request::SecRequest;
    use crate::tests::fixtures::data::filing_index::INLINE_XBRL_FOLDER_INDEX_URL;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);

    const ACCESSION_NUMBER: &str = "0000003545-26-000047";
    const OTHER_ACCESSION_NUMBER: &str = "0000003545-26-000048";
    const FOLDER_INDEX: &[u8] = br#"{"directory":{"item":[{"name":"alco-20260908_htm.xml"},{"name":"alco-20260908_lab.xml"}]}}"#;

    type TestExecute = ExecuteSecRequest<FilingIndex, RoutingSecClient>;

    fn accession_number(raw_accession_number: &str) -> AccessionNumber {
        AccessionNumber::new(raw_accession_number).expect(
            "Given a hardcoded accession number of the right shape, creation should always succeed",
        )
    }

    fn filing(raw_accession_number: &str) -> FilingRef {
        FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            accession_number(raw_accession_number),
            ParsedForm::new(Form::TenQ, false),
            NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    fn sec_client(folder_index_body: &[u8]) -> RoutingSecClient {
        RoutingSecClient::new().route(
            INLINE_XBRL_FOLDER_INDEX_URL,
            200,
            "application/json",
            folder_index_body,
        )
    }

    fn execute_without_output(
        folder_index_body: &[u8],
        request_filing: FilingRef,
        context_filing: FilingRef,
    ) -> TestExecute {
        ExecuteSecRequest::new(
            ExecuteSecRequestInput::new(
                sec_client(folder_index_body),
                SecRequest::new(request_filing),
            ),
            ExecuteSecRequestContext::new(context_filing),
        )
    }

    async fn execute_with_output(
        folder_index_body: &[u8],
        request_filing: FilingRef,
        context_filing: FilingRef,
    ) -> TestExecute {
        let mut state = execute_without_output(folder_index_body, request_filing, context_filing);
        state.compute_output_data_async().await.expect(
            "Given a client that serves the folder index as JSON, the request should always succeed",
        );
        state
    }

    async fn computed_source(folder_index_body: &[u8]) -> TestExecute {
        execute_with_output(
            folder_index_body,
            filing(ACCESSION_NUMBER),
            filing(ACCESSION_NUMBER),
        )
        .await
    }

    #[tokio::test]
    async fn should_parse_the_response_into_the_folder_index_when_source_has_output() {
        let state = computed_source(FOLDER_INDEX).await;

        let expected_result = FolderIndex::new(vec![
            "alco-20260908_htm.xml".to_string(),
            "alco-20260908_lab.xml".to_string(),
        ]);

        let result = SelectFilingDocuments::try_from((state, SAMPLE_EXECUTION_ID))
            .expect(
                "Given a source state with a folder index, the transition should always succeed",
            )
            .input_data()
            .folder_index
            .clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_the_accession_number_from_the_context_when_source_has_output() {
        let state = computed_source(FOLDER_INDEX).await;

        let expected_result = accession_number(ACCESSION_NUMBER);

        let result = SelectFilingDocuments::try_from((state, SAMPLE_EXECUTION_ID))
            .expect(
                "Given a source state with a folder index, the transition should always succeed",
            )
            .input_data()
            .accession_number
            .clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_the_execution_id_from_the_run_when_source_has_output() {
        let state = computed_source(FOLDER_INDEX).await;

        let expected_result = SelectFilingDocumentsContext::new(SAMPLE_EXECUTION_ID);

        let result = SelectFilingDocuments::try_from((state, SAMPLE_EXECUTION_ID))
            .expect(
                "Given a source state with a folder index, the transition should always succeed",
            )
            .context_data()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_output_conversion_naming_the_url_when_the_body_has_no_directory()
    {
        let state = computed_source(b"{}").await;

        let expected_result = Err(TransitionError::FailedOutputConversion(
            transition::FailedOutputConversion::new(
                EXECUTE_SEC_REQUEST,
                SELECT_FILING_DOCUMENTS,
                "[InvalidFolderIndex] The folder index at 'https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json' does not parse, Reason: 'missing field `directory` at line 1 column 2'",
            ),
        ));

        let result = SelectFilingDocuments::try_from((state, SAMPLE_EXECUTION_ID));

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_context_conversion_when_request_names_another_filing() {
        let state = execute_with_output(
            FOLDER_INDEX,
            filing(ACCESSION_NUMBER),
            filing(OTHER_ACCESSION_NUMBER),
        )
        .await;

        let expected_result = Err(TransitionError::FailedContextConversion(
            transition::FailedContextConversion::new(
                EXECUTE_SEC_REQUEST,
                SELECT_FILING_DOCUMENTS,
                "The request targets 'https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json' for filing '0000003545-26-000047', but the context names filing '0000003545-26-000048'",
            ),
        ));

        let result = SelectFilingDocuments::try_from((state, SAMPLE_EXECUTION_ID));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = execute_without_output(
            FOLDER_INDEX,
            filing(ACCESSION_NUMBER),
            filing(ACCESSION_NUMBER),
        );

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(EXECUTE_SEC_REQUEST, SELECT_FILING_DOCUMENTS),
        ));

        let result = SelectFilingDocuments::try_from((state, SAMPLE_EXECUTION_ID));

        assert_eq!(result, expected_result);
    }
}
