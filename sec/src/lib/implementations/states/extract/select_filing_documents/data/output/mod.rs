//! # Select Filing Documents Output
//!
//! Provides the [`SelectFilingDocumentsOutput`] produced by the
//! [`SelectFilingDocuments`](crate::implementations::states::extract::select_filing_documents::SelectFilingDocuments)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::filing_documents::FilingDocuments;
use crate::traits::state_machine::state::StateData;

/// Output data of the [`SelectFilingDocuments`](super::super::SelectFilingDocuments) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct SelectFilingDocumentsOutput {
    /// The file names of the XBRL instance and the optional label file.
    pub documents: FilingDocuments,
}

impl SelectFilingDocumentsOutput {
    /// Creates a new [`SelectFilingDocumentsOutput`] from the selected documents.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::select_filing_documents::SelectFilingDocumentsOutput;
    /// use sec::shared::filing_documents::FilingDocuments;
    ///
    /// let output = SelectFilingDocumentsOutput::new(FilingDocuments::new("aapl-20170930.xml", None));
    ///
    /// let expected_result = "aapl-20170930.xml";
    ///
    /// let result = output.documents.instance();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(documents: FilingDocuments) -> Self {
        Self { documents }
    }
}

impl StateData for SelectFilingDocumentsOutput {
    /// Replaces the documents if the updater sets them.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(documents) = updates.documents {
            self.documents = documents;
        }
        Ok(())
    }
}

impl SMStateData for SelectFilingDocumentsOutput {
    type UpdateType = SelectFilingDocumentsOutputUpdater;

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

impl fmt::Display for SelectFilingDocumentsOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tInstance: {}, Label File: {}",
            self.documents.instance(),
            self.documents.label_file().unwrap_or("None")
        )
    }
}

/// Updater for [`SelectFilingDocumentsOutput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SelectFilingDocumentsOutputUpdater {
    /// The new selected documents.
    pub documents: Option<FilingDocuments>,
}

impl SelectFilingDocumentsOutputUpdater {
    /// Returns a [`SelectFilingDocumentsOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> SelectFilingDocumentsOutputUpdaterBuilder {
        SelectFilingDocumentsOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`SelectFilingDocumentsOutputUpdater`].
#[derive(Debug)]
pub struct SelectFilingDocumentsOutputUpdaterBuilder {
    updater: SelectFilingDocumentsOutputUpdater,
}

impl SelectFilingDocumentsOutputUpdaterBuilder {
    /// Creates a [`SelectFilingDocumentsOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            updater: SelectFilingDocumentsOutputUpdater { documents: None },
        }
    }

    /// Sets the selected documents.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn documents(mut self, documents: FilingDocuments) -> Self {
        self.updater.documents = Some(documents);
        self
    }

    /// Builds the [`SelectFilingDocumentsOutputUpdater`].
    #[must_use]
    pub fn build(self) -> SelectFilingDocumentsOutputUpdater {
        self.updater
    }
}

impl Default for SelectFilingDocumentsOutputUpdaterBuilder {
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

    fn inline_documents() -> FilingDocuments {
        FilingDocuments::new(
            "alco-20260908_htm.xml",
            Some("alco-20260908_lab.xml".to_string()),
        )
    }

    #[test]
    fn should_display_both_file_names_when_the_filing_has_a_label_file() {
        let output = SelectFilingDocumentsOutput::new(inline_documents());

        let expected_result =
            "\tInstance: alco-20260908_htm.xml, Label File: alco-20260908_lab.xml";

        let result = output.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_none_when_the_filing_has_no_label_file() {
        let output =
            SelectFilingDocumentsOutput::new(FilingDocuments::new("aapl-20170930.xml", None));

        let expected_result = "\tInstance: aapl-20170930.xml, Label File: None";

        let result = output.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_documents_when_update_contains_documents() {
        let mut output =
            SelectFilingDocumentsOutput::new(FilingDocuments::new("aapl-20170930.xml", None));
        let update = SelectFilingDocumentsOutputUpdater::builder()
            .documents(inline_documents())
            .build();

        let expected_result = &SelectFilingDocumentsOutput::new(inline_documents());

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_unchanged_when_update_is_empty() {
        let mut output = SelectFilingDocumentsOutput::new(inline_documents());
        let update = SelectFilingDocumentsOutputUpdater::builder().build();

        let expected_result = &SelectFilingDocumentsOutput::new(inline_documents());

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_select_filing_documents_output() {
        implements_value_traits::<SelectFilingDocumentsOutput>();
    }
}
