//! # Transition: `SelectFilingDocuments` → `PrepareSecRequest`
//!
//! Implements the [`TryFrom`] conversion producing a [`PrepareSecRequest`] of the
//! [`FilingDocumentSet`] kind from a [`SelectFilingDocuments`] and the [`FilingRef`] of the
//! filing.
//!
//! [`SelectFilingDocuments`] holds only the accession number, so the caller passes the
//! [`FilingRef`] that the state machine run holds. The request input joins it with the selected
//! [`FilingDocuments`](crate::shared::filing_documents::FilingDocuments).
//!
//! # Errors
//!
//! Returns [`TransitionError`] when the source state has no computed output
//! ([`MissingOutput`](crate::error::state_machine::transition::MissingOutput)), or when the
//! [`FilingRef`] names another accession number than the source state
//! ([`FailedContextConversion`](crate::error::state_machine::transition::FailedContextConversion)).

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use crate::implementations::states::extract::select_filing_documents::SelectFilingDocuments;
use crate::implementations::states::extract::select_filing_documents::constants::STATE_NAME as SELECT_FILING_DOCUMENTS;
use crate::shared::filing_documents::FilingDocumentSetRef;
use crate::shared::filing_ref::FilingRef;
use crate::shared::request::implementations::sec_request::FilingDocumentSet;

impl TryFrom<(SelectFilingDocuments, FilingRef)> for PrepareSecRequest<FilingDocumentSet> {
    type Error = TransitionError;

    /// Builds a [`PrepareSecRequest`] of the [`FilingDocumentSet`] kind from a computed
    /// [`SelectFilingDocuments`] and the [`FilingRef`] of the filing.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    ///
    /// Returns [`TransitionError::FailedContextConversion`] if `filing` names another accession
    /// number than `state`. The request URL follows `filing`, so a mismatch downloads the files
    /// of another filing.
    fn try_from(
        (state, filing): (SelectFilingDocuments, FilingRef),
    ) -> Result<Self, TransitionError> {
        let (input, output, _context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(SELECT_FILING_DOCUMENTS, PREPARE_SEC_REQUEST)
        })?;

        if *filing.accession_number() != input.accession_number {
            let reason = format!(
                "The filing names accession number '{}', but the selected documents belong to accession number '{}'",
                filing.accession_number(),
                input.accession_number,
            );
            return Err(transition::FailedContextConversion::new(
                SELECT_FILING_DOCUMENTS,
                PREPARE_SEC_REQUEST,
                reason,
            )
            .into());
        }

        let request_input = FilingDocumentSetRef::new(filing, output_data.documents);
        Ok(Self::new(
            PrepareSecRequestInput::new(request_input.clone()),
            PrepareSecRequestContext::new(request_input),
        ))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::implementations::states::extract::select_filing_documents::{
        SelectFilingDocumentsContext, SelectFilingDocumentsInput,
    };
    use crate::prelude::*;
    use crate::shared::cik::Cik;
    use crate::shared::filing_documents::{FilingDocuments, FolderIndex};
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};

    const ACCESSION_NUMBER: &str = "0000003545-26-000047";
    const OTHER_ACCESSION_NUMBER: &str = "0000003545-26-000048";
    const INSTANCE: &str = "alco-20260908_htm.xml";
    const LABEL_FILE: &str = "alco-20260908_lab.xml";

    type PrepareFilingDocumentsRequest = PrepareSecRequest<FilingDocumentSet>;

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

    fn select_without_output() -> SelectFilingDocuments {
        SelectFilingDocuments::new(
            SelectFilingDocumentsInput::new(
                accession_number(ACCESSION_NUMBER),
                FolderIndex::new(vec![INSTANCE.to_string(), LABEL_FILE.to_string()]),
            ),
            SelectFilingDocumentsContext::new(Uuid::nil()),
        )
    }

    async fn select_with_output() -> SelectFilingDocuments {
        let mut state = select_without_output();
        state.compute_output_data_async().await.expect(
            "Given a folder with one instance and one label file, the selection should always succeed",
        );
        state
    }

    #[tokio::test]
    async fn should_join_the_filing_and_the_selected_documents_when_source_has_output() {
        let state = select_with_output().await;
        let filing = filing(ACCESSION_NUMBER);

        let expected_result = &FilingDocumentSetRef::new(
            filing.clone(),
            FilingDocuments::new(INSTANCE, Some(LABEL_FILE.to_string())),
        );

        let prepare = PrepareFilingDocumentsRequest::try_from((state, filing))
            .expect("Given a source state with output for the same filing, the transition should always succeed");
        let result = prepare.input_data().request_input();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_put_the_same_request_input_into_the_context_when_source_has_output() {
        let state = select_with_output().await;
        let filing = filing(ACCESSION_NUMBER);

        let expected_result = &FilingDocumentSetRef::new(
            filing.clone(),
            FilingDocuments::new(INSTANCE, Some(LABEL_FILE.to_string())),
        );

        let prepare = PrepareFilingDocumentsRequest::try_from((state, filing))
            .expect("Given a source state with output for the same filing, the transition should always succeed");
        let result = prepare.context_data().request_input();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_context_conversion_when_the_filing_names_another_accession_number()
     {
        let state = select_with_output().await;
        let filing = filing(OTHER_ACCESSION_NUMBER);

        let expected_result = Err(TransitionError::FailedContextConversion(
            transition::FailedContextConversion::new(
                SELECT_FILING_DOCUMENTS,
                PREPARE_SEC_REQUEST,
                "The filing names accession number '0000003545-26-000048', but the selected documents belong to accession number '0000003545-26-000047'",
            ),
        ));

        let result = PrepareFilingDocumentsRequest::try_from((state, filing));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = select_without_output();
        let filing = filing(ACCESSION_NUMBER);

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(SELECT_FILING_DOCUMENTS, PREPARE_SEC_REQUEST),
        ));

        let result = PrepareFilingDocumentsRequest::try_from((state, filing));

        assert_eq!(result, expected_result);
    }
}
