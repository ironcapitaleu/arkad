//! # Transition: `StoreFilingIndex` → `SelectFilingDocuments`
//!
//! Implements the [`TryFrom`] conversion producing a [`SelectFilingDocuments`] from a
//! [`StoreFilingIndex`] and the `execution_id`.
//!
//! It parses the stored response into a [`FolderIndex`] and takes the accession number from the
//! filing in the context. The transition into [`StoreFilingIndex`] already checked that the
//! request names the filing of the context. The caller passes the `execution_id` of the state
//! machine run.
//!
//! # Errors
//!
//! Returns [`TransitionError`] when the source state has no computed output
//! ([`MissingOutput`](crate::error::state_machine::transition::MissingOutput)), or when the
//! response does not parse into a [`FolderIndex`]
//! ([`FailedOutputConversion`](crate::error::state_machine::transition::FailedOutputConversion)).

use uuid::Uuid;

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::select_filing_documents::constants::STATE_NAME as SELECT_FILING_DOCUMENTS;
use crate::implementations::states::extract::select_filing_documents::{
    SelectFilingDocuments, SelectFilingDocumentsContext, SelectFilingDocumentsInput,
};
use crate::implementations::states::extract::store_filing_index::STATE_NAME as STORE_FILING_INDEX;
use crate::implementations::states::extract::store_filing_index::StoreFilingIndex;
use crate::shared::filing_documents::FolderIndex;

impl<R> TryFrom<(StoreFilingIndex<R>, Uuid)> for SelectFilingDocuments {
    type Error = TransitionError;

    /// Builds a [`SelectFilingDocuments`] from a computed [`StoreFilingIndex`] and the
    /// `execution_id`.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output, so the store
    /// did not write the folder index.
    ///
    /// Returns [`TransitionError::FailedOutputConversion`] if the response does not parse into a
    /// [`FolderIndex`]. The reason is the text of the
    /// [`InvalidFolderIndex`](crate::shared::filing_documents::InvalidFolderIndex), which names
    /// the URL of the folder index.
    fn try_from(
        (state, execution_id): (StoreFilingIndex<R>, Uuid),
    ) -> Result<Self, TransitionError> {
        let (input, output, context) = state.into_parts();
        if output.is_none() {
            return Err(transition::MissingOutput::new(
                STORE_FILING_INDEX,
                SELECT_FILING_DOCUMENTS,
            )
            .into());
        }

        // Deviation: the reason holds the text of the inner error, so the chain nests one
        // `Reason:` in another, not `Caused by:`. The lead decided this form in STA-371, and
        // `FailedOutputConversion` carries no source.
        let folder_index = FolderIndex::try_from(input.response()).map_err(|e| {
            transition::FailedOutputConversion::new(
                STORE_FILING_INDEX,
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
    use std::collections::HashMap;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use storage::implementations::repository::filesystem::IngestionMode;

    use super::*;
    use crate::implementations::states::extract::store_filing_index::{
        StoreFilingIndexContext, StoreFilingIndexInput,
    };
    use crate::prelude::*;
    use crate::shared::cik::Cik;
    use crate::shared::content_type::ContentType;
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::headers::Headers;
    use crate::shared::response::implementations::sec_response::SecResponse;
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::data::filing_index::INLINE_XBRL_FOLDER_INDEX_URL;
    use crate::tests::fixtures::sample_write_repository::FakeFilingIndexRepository;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);

    const ACCESSION_NUMBER: &str = "0000003545-26-000047";
    const FOLDER_INDEX: &[u8] = br#"{"directory":{"item":[{"name":"alco-20260908_htm.xml"},{"name":"alco-20260908_lab.xml"}]}}"#;

    type TestStore = StoreFilingIndex<FakeFilingIndexRepository>;

    fn accession_number() -> AccessionNumber {
        AccessionNumber::new(ACCESSION_NUMBER).expect(
            "Given a hardcoded accession number of the right shape, creation should always succeed",
        )
    }

    fn folder_index_url() -> Url {
        INLINE_XBRL_FOLDER_INDEX_URL
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn store_without_output(folder_index_body: &[u8]) -> TestStore {
        let response = SecResponse::from_parts(
            folder_index_url(),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            folder_index_body.to_vec(),
        )
        .expect("Given a hardcoded JSON body, building the response should always succeed");
        let user_agent = UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        let filing = FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            accession_number(),
            ParsedForm::new(Form::TenQ, false),
            NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
        );
        StoreFilingIndex::new(
            StoreFilingIndexInput::new(response, folder_index_url(), user_agent),
            StoreFilingIndexContext::new(
                filing,
                SAMPLE_EXECUTION_ID,
                IngestionMode::Incremental,
                FakeFilingIndexRepository::new(),
            ),
        )
    }

    async fn computed_source(folder_index_body: &[u8]) -> TestStore {
        let mut state = store_without_output(folder_index_body);
        state.compute_output_data_async().await.expect(
            "Given a fake repository that accepts every write, the store should always succeed",
        );
        state
    }

    async fn selected(folder_index_body: &[u8]) -> SelectFilingDocuments {
        SelectFilingDocuments::try_from((
            computed_source(folder_index_body).await,
            SAMPLE_EXECUTION_ID,
        ))
        .expect("Given a source state with a folder index, the transition should always succeed")
    }

    #[tokio::test]
    async fn should_parse_the_response_into_the_folder_index_when_source_has_output() {
        let body = FOLDER_INDEX;

        let expected_result = FolderIndex::new(vec![
            "alco-20260908_htm.xml".to_string(),
            "alco-20260908_lab.xml".to_string(),
        ]);

        let result = selected(body).await.input_data().folder_index.clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_the_accession_number_from_the_context_when_source_has_output() {
        let body = FOLDER_INDEX;

        let expected_result = accession_number();

        let result = selected(body).await.input_data().accession_number.clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_the_execution_id_from_the_run_when_source_has_output() {
        let body = FOLDER_INDEX;

        let expected_result = SelectFilingDocumentsContext::new(SAMPLE_EXECUTION_ID);

        let result = selected(body).await.context_data().clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_output_conversion_naming_the_url_when_the_body_has_no_directory()
    {
        let state = computed_source(b"{}").await;

        let expected_result = Err(TransitionError::FailedOutputConversion(
            transition::FailedOutputConversion::new(
                STORE_FILING_INDEX,
                SELECT_FILING_DOCUMENTS,
                "[InvalidFolderIndex] The folder index at 'https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json' does not parse, Reason: 'missing field `directory` at line 1 column 2'",
            ),
        ));

        let result = SelectFilingDocuments::try_from((state, SAMPLE_EXECUTION_ID));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = store_without_output(FOLDER_INDEX);

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(STORE_FILING_INDEX, SELECT_FILING_DOCUMENTS),
        ));

        let result = SelectFilingDocuments::try_from((state, SAMPLE_EXECUTION_ID));

        assert_eq!(result, expected_result);
    }
}
