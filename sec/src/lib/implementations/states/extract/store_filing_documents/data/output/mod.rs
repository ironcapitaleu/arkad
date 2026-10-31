//! # Store Filing Documents Output
//!
//! Provides the [`StoreFilingDocumentsOutput`] produced by the
//! [`StoreFilingDocuments`](crate::implementations::states::extract::store_filing_documents::StoreFilingDocuments)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::filing_documents_ref::FilingDocumentsRef;
use crate::traits::state_machine::state::StateData;

/// Output data of the [`StoreFilingDocuments`](super::super::StoreFilingDocuments) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct StoreFilingDocumentsOutput {
    /// The reference to the stored files of the filing.
    pub documents: FilingDocumentsRef,
}

impl StoreFilingDocumentsOutput {
    /// Creates a new [`StoreFilingDocumentsOutput`] from the reference to the stored files.
    #[must_use]
    pub const fn new(documents: FilingDocumentsRef) -> Self {
        Self { documents }
    }
}

impl StateData for StoreFilingDocumentsOutput {
    /// Replaces the documents reference if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(documents) = updates.documents {
            self.documents = documents;
        }
        Ok(())
    }
}

impl SMStateData for StoreFilingDocumentsOutput {
    type UpdateType = StoreFilingDocumentsOutputUpdater;

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

impl fmt::Display for StoreFilingDocumentsOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\t{}", self.documents)
    }
}

/// Updater for [`StoreFilingDocumentsOutput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug)]
pub struct StoreFilingDocumentsOutputUpdater {
    /// The new documents reference.
    pub documents: Option<FilingDocumentsRef>,
}

impl StoreFilingDocumentsOutputUpdater {
    /// Returns a [`StoreFilingDocumentsOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> StoreFilingDocumentsOutputUpdaterBuilder {
        StoreFilingDocumentsOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreFilingDocumentsOutputUpdater`].
#[derive(Debug, Default)]
pub struct StoreFilingDocumentsOutputUpdaterBuilder {
    documents: Option<FilingDocumentsRef>,
}

impl StoreFilingDocumentsOutputUpdaterBuilder {
    /// Creates a [`StoreFilingDocumentsOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self { documents: None }
    }

    /// Sets the documents reference.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn documents(mut self, documents: FilingDocumentsRef) -> Self {
        self.documents = Some(documents);
        self
    }

    /// Builds the [`StoreFilingDocumentsOutputUpdater`].
    #[must_use]
    pub fn build(self) -> StoreFilingDocumentsOutputUpdater {
        StoreFilingDocumentsOutputUpdater {
            documents: self.documents,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::StateData as SMStateData;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};

    fn documents(has_label_file: bool) -> FilingDocumentsRef {
        let filing = FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenK, false),
            NaiveDate::from_ymd_opt(2026, 9, 8)
                .expect("Given a hardcoded calendar date, the date should always build"),
        );
        FilingDocumentsRef::new(filing, has_label_file)
    }

    #[test]
    fn should_update_documents_when_update_contains_documents() {
        let mut output = StoreFilingDocumentsOutput::new(documents(true));
        let update = StoreFilingDocumentsOutputUpdater::builder()
            .documents(documents(false))
            .build();

        let expected_result = &StoreFilingDocumentsOutput::new(documents(false));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_unchanged_when_update_is_empty() {
        let mut output = StoreFilingDocumentsOutput::new(documents(true));
        let update = StoreFilingDocumentsOutputUpdater::builder().build();

        let expected_result = &StoreFilingDocumentsOutput::new(documents(true));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_filing_documents_output() {
        implements_value_traits::<StoreFilingDocumentsOutput>();
    }
}
