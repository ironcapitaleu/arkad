//! # Select Filing Documents State
//!
//! Provides the [`SelectFilingDocuments`] state, which reads the folder index of one filing and
//! picks the file names of its XBRL instance and its label file. The rules are those of
//! [`FilingDocuments::select`].
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`SelectFilingDocumentsContext`] carried alongside the state.
//! - [`data`]: The [`SelectFilingDocumentsInput`] and [`SelectFilingDocumentsOutput`] data types.
//!
//! ## Usage
//!
//! ```rust
//! use std::collections::HashMap;
//!
//! use sec::implementations::states::extract::select_filing_documents::*;
//! use sec::prelude::*;
//! use sec::shared::content_type::ContentType;
//! use sec::shared::financial::accession_number::AccessionNumber;
//! use sec::shared::headers::Headers;
//! use sec::shared::response::implementations::sec_response::SecResponse;
//! use sec::shared::status_code::StatusCode;
//! use uuid::Uuid;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let folder_index = SecResponse::from_parts(
//!     "https://www.sec.gov/Archives/edgar/data/320193/000032019317000070/index.json"
//!         .parse()
//!         .expect("Given a hardcoded valid URL, parsing should always succeed"),
//!     Headers::new(HashMap::new()),
//!     ContentType::Json,
//!     StatusCode::Ok,
//!     br#"{"directory": {"item": [{"name": "aapl-20170930.xml"}]}}"#.to_vec(),
//! )
//! .expect("Given a valid JSON body, building the response should always succeed");
//! let input =
//!     SelectFilingDocumentsInput::new(AccessionNumber::new("0000320193-17-000070"), folder_index);
//! let mut state = SelectFilingDocuments::new(input, SelectFilingDocumentsContext::new(Uuid::nil()));
//! state
//!     .compute_output_data_async()
//!     .await
//!     .expect("Given a folder with one instance, the selection should always succeed");
//!
//! let expected_result = "aapl-20170930.xml";
//!
//! let result = state
//!     .output_data()
//!     .expect("Given a successful computation, the output should always be present")
//!     .documents
//!     .instance();
//!
//! assert_eq!(result, expected_result);
//! # }
//! ```

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::error::state_machine::state::FailedDocumentSelection;
use crate::shared::filing_documents::FilingDocuments;
use crate::traits::error::FromDomainError;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::SelectFilingDocumentsContext;
pub use data::SelectFilingDocumentsInput;
pub use data::SelectFilingDocumentsOutput;

/// Picks the XBRL instance and the label file of one filing from its folder index.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct SelectFilingDocuments {
    input: SelectFilingDocumentsInput,
    context: SelectFilingDocumentsContext,
    output: Option<SelectFilingDocumentsOutput>,
}

impl SelectFilingDocuments {
    /// Creates a new [`SelectFilingDocuments`] state with no output computed yet.
    #[must_use]
    pub const fn new(
        input: SelectFilingDocumentsInput,
        context: SelectFilingDocumentsContext,
    ) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        SelectFilingDocumentsInput,
        Option<SelectFilingDocumentsOutput>,
        SelectFilingDocumentsContext,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl State for SelectFilingDocuments {
    /// Picks the documents from the folder index and stores them as output.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::FailedDocumentSelection`] if [`FilingDocuments::select`] fails. The
    /// error names the accession number of the filing.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let documents = FilingDocuments::select(
            &self.input.accession_number,
            self.input.folder_index.raw_body(),
        )
        .map_err(|e| FailedDocumentSelection::from_domain_error(STATE_NAME, e))?;
        self.output = Some(SelectFilingDocumentsOutput::new(documents));
        Ok(())
    }
}

impl SMState for SelectFilingDocuments {
    type InputData = SelectFilingDocumentsInput;
    type OutputData = SelectFilingDocumentsOutput;
    type Context = SelectFilingDocumentsContext;

    fn state_name(&self) -> impl ToString {
        STATE_NAME
    }

    /// Blocking wrapper around [`compute_output_data_async`](crate::traits::state_machine::state::State::compute_output_data_async).
    ///
    /// # Panics
    ///
    /// - Panics if the async computation returns an error.
    /// - Panics if called inside a `current_thread` runtime, because `block_in_place` requires a
    ///   multi-thread runtime.
    fn compute_output_data(&mut self) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(self.compute_output_data_async()))
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.compute_output_data_async())
        };

        if let Err(e) = result {
            panic!("compute_output_data failed: {e}")
        }
    }

    fn context_data(&self) -> &Self::Context {
        &self.context
    }

    fn input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl fmt::Display for SelectFilingDocuments {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::HashMap;
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::shared::content_type::ContentType;
    use crate::shared::filing_documents::FilingDocumentsError;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::headers::Headers;
    use crate::shared::response::implementations::sec_response::SecResponse;
    use crate::shared::status_code::StatusCode;
    use crate::tests::fixtures::data::filing_index::INLINE_XBRL_FOLDER_INDEX;

    /// Returns a `200 OK` JSON response with `body`, as the SEC sends a folder index.
    pub fn folder_index_response(body: &[u8]) -> SecResponse {
        SecResponse::from_parts(
            "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json"
                .parse()
                .expect("Given a hardcoded valid URL, parsing should always succeed"),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            body.to_vec(),
        )
        .expect("Given a valid JSON body, building the response should always succeed")
    }

    fn test_state(accession_number: &str, body: &[u8]) -> SelectFilingDocuments {
        SelectFilingDocuments::new(
            SelectFilingDocumentsInput::new(
                AccessionNumber::new(accession_number),
                folder_index_response(body),
            ),
            SelectFilingDocumentsContext::new(Uuid::nil()),
        )
    }

    async fn computed_documents(state: &mut SelectFilingDocuments) -> FilingDocuments {
        state
            .compute_output_data_async()
            .await
            .expect("Given a folder with one instance, the selection should always succeed");
        state
            .output_data()
            .expect("Given a successful computation, the output should always be present")
            .documents
            .clone()
    }

    #[test]
    fn should_return_state_name_for_select_filing_documents() {
        let state = test_state("0000003545-26-000047", INLINE_XBRL_FOLDER_INDEX);

        let expected_result = "Select Filing Documents";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let state = test_state("0000003545-26-000047", INLINE_XBRL_FOLDER_INDEX);

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_htm_xml_instance_and_the_label_file_for_inline_xbrl_filing() {
        let mut state = test_state("0000003545-26-000047", INLINE_XBRL_FOLDER_INDEX);

        let expected_result = FilingDocuments::new(
            "alco-20260908_htm.xml",
            Some("alco-20260908_lab.xml".to_string()),
        );

        let result = computed_documents(&mut state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_document_selection_naming_the_filing_when_the_folder_has_no_instance()
     {
        let folder_index = br#"{"directory": {"item": [{"name": "FilingSummary.xml"}]}}"#;
        let mut state = test_state("0000003545-26-000047", folder_index);

        let expected_result = StateError::FailedDocumentSelection(FailedDocumentSelection::new(
            STATE_NAME,
            FilingDocumentsError::MissingXbrlInstance {
                accession_number: AccessionNumber::new("0000003545-26-000047"),
            },
        ));

        let result = state
            .compute_output_data_async()
            .await
            .expect_err("Given a folder without an instance, the selection should always fail");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_output_empty_when_the_folder_has_no_instance() {
        let folder_index = br#"{"directory": {"item": []}}"#;
        let mut state = test_state("0000003545-26-000047", folder_index);

        let expected_result = None;

        let _ = state.compute_output_data_async().await;
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let mut state = test_state("0000003545-26-000047", INLINE_XBRL_FOLDER_INDEX);

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_output_when_calling_sync_compute_outside_tokio_runtime() {
        let mut state = test_state("0000003545-26-000047", INLINE_XBRL_FOLDER_INDEX);

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "compute_output_data failed")]
    fn should_panic_when_calling_sync_compute_with_folder_without_instance() {
        let mut state = test_state("0000003545-26-000047", br#"{"directory": {"item": []}}"#);

        state.compute_output_data();
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_select_filing_documents() {
        implements_value_traits::<SelectFilingDocuments>();
    }
}
