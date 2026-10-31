//! # Select Filing Documents Input
//!
//! Provides the [`SelectFilingDocumentsInput`] fed into the
//! [`SelectFilingDocuments`](crate::implementations::states::extract::select_filing_documents::SelectFilingDocuments)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::financial::accession_number::AccessionNumber;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`SelectFilingDocuments`](super::super::SelectFilingDocuments) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct SelectFilingDocumentsInput {
    /// The accession number of the filing, which an error names.
    pub accession_number: AccessionNumber,
    /// The response to the [`FilingIndex`](crate::shared::request::implementations::sec_request::FilingIndex)
    /// request: the folder index JSON of the filing.
    pub folder_index: SecResponse,
}

impl SelectFilingDocumentsInput {
    /// Creates a new [`SelectFilingDocumentsInput`] from the accession number and the folder
    /// index response of one filing.
    #[must_use]
    pub const fn new(accession_number: AccessionNumber, folder_index: SecResponse) -> Self {
        Self {
            accession_number,
            folder_index,
        }
    }
}

impl StateData for SelectFilingDocumentsInput {
    /// Replaces each field that the updater sets.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(accession_number) = updates.accession_number {
            self.accession_number = accession_number;
        }
        if let Some(folder_index) = updates.folder_index {
            self.folder_index = folder_index;
        }
        Ok(())
    }
}

impl SMStateData for SelectFilingDocumentsInput {
    type UpdateType = SelectFilingDocumentsInputUpdater;

    fn state(&self) -> &Self {
        self
    }

    /// Delegates to the SEC [`StateData::update_state`] implementation.
    ///
    /// # Panics
    ///
    /// Panics if the fallible SEC update returns an error.
    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Err(e) = <Self as StateData>::update_state(self, updates) {
            panic!("StateData::update_state failed: {e}")
        }
    }
}

impl fmt::Display for SelectFilingDocumentsInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tFiling: {}, Folder Index Size: {} bytes",
            self.accession_number,
            self.folder_index.raw_body().len()
        )
    }
}

/// Updater for [`SelectFilingDocumentsInput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug)]
pub struct SelectFilingDocumentsInputUpdater {
    /// The new accession number.
    pub accession_number: Option<AccessionNumber>,
    /// The new folder index response.
    pub folder_index: Option<SecResponse>,
}

impl SelectFilingDocumentsInputUpdater {
    /// Returns a [`SelectFilingDocumentsInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> SelectFilingDocumentsInputUpdaterBuilder {
        SelectFilingDocumentsInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`SelectFilingDocumentsInputUpdater`].
#[derive(Debug)]
pub struct SelectFilingDocumentsInputUpdaterBuilder {
    updater: SelectFilingDocumentsInputUpdater,
}

impl SelectFilingDocumentsInputUpdaterBuilder {
    /// Creates a [`SelectFilingDocumentsInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            updater: SelectFilingDocumentsInputUpdater {
                accession_number: None,
                folder_index: None,
            },
        }
    }

    /// Sets the accession number.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn accession_number(mut self, accession_number: AccessionNumber) -> Self {
        self.updater.accession_number = Some(accession_number);
        self
    }

    /// Sets the folder index response.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn folder_index(mut self, folder_index: SecResponse) -> Self {
        self.updater.folder_index = Some(folder_index);
        self
    }

    /// Builds the [`SelectFilingDocumentsInputUpdater`].
    #[must_use]
    pub fn build(self) -> SelectFilingDocumentsInputUpdater {
        self.updater
    }
}

impl Default for SelectFilingDocumentsInputUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use state_maschine::prelude::StateData as SMStateData;

    use super::*;
    use crate::implementations::states::extract::select_filing_documents::tests::folder_index_response;

    fn test_input() -> SelectFilingDocumentsInput {
        SelectFilingDocumentsInput::new(
            AccessionNumber::new("0000003545-26-000047"),
            folder_index_response(br#"{"directory": {"item": []}}"#),
        )
    }

    #[test]
    fn should_display_the_accession_number_and_the_size_for_select_filing_documents_input() {
        let input = test_input();

        let expected_result = "\tFiling: 0000003545-26-000047, Folder Index Size: 27 bytes";

        let result = input.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_accession_number_when_update_contains_accession_number() {
        let mut input = test_input();
        let update = SelectFilingDocumentsInputUpdater::builder()
            .accession_number(AccessionNumber::new("0000320193-17-000070"))
            .build();

        let expected_result = &AccessionNumber::new("0000320193-17-000070");

        SMStateData::update_state(&mut input, update);
        let result = &input.state().accession_number;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_folder_index_when_update_contains_folder_index() {
        let mut input = test_input();
        let update = SelectFilingDocumentsInputUpdater::builder()
            .folder_index(folder_index_response(b"{}"))
            .build();

        let expected_result: &[u8] = b"{}";

        SMStateData::update_state(&mut input, update);
        let result = input.state().folder_index.raw_body();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_input_unchanged_when_update_is_empty() {
        let mut input = test_input();
        let update = SelectFilingDocumentsInputUpdater::builder().build();

        let expected_result = &test_input();

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_select_filing_documents_input() {
        implements_value_traits::<SelectFilingDocumentsInput>();
    }
}
