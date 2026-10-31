//! # Select Filings Output
//!
//! Provides the [`SelectFilingsOutput`] produced by the
//! [`SelectFilings`](crate::implementations::states::extract::select_filings::SelectFilings)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::filing_ref::FilingRef;
use crate::traits::state_machine::state::StateData;

/// Output data of the [`SelectFilings`](super::super::SelectFilings) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct SelectFilingsOutput {
    /// The selected filings, one per accession number, in index order.
    pub filings: Vec<FilingRef>,
}

impl SelectFilingsOutput {
    /// Creates a new [`SelectFilingsOutput`] from the selected filings.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::select_filings::SelectFilingsOutput;
    ///
    /// let output = SelectFilingsOutput::new(Vec::new());
    ///
    /// let expected_result = 0;
    ///
    /// let result = output.filings.len();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(filings: Vec<FilingRef>) -> Self {
        Self { filings }
    }
}

impl StateData for SelectFilingsOutput {
    /// Replaces the filings if the updater sets them.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(filings) = updates.filings {
            self.filings = filings;
        }
        Ok(())
    }
}

impl SMStateData for SelectFilingsOutput {
    type UpdateType = SelectFilingsOutputUpdater;

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

impl fmt::Display for SelectFilingsOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tSelected Filings: {}", self.filings.len())
    }
}

/// Updater for [`SelectFilingsOutput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SelectFilingsOutputUpdater {
    /// The new selected filings.
    pub filings: Option<Vec<FilingRef>>,
}

impl SelectFilingsOutputUpdater {
    /// Returns a [`SelectFilingsOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> SelectFilingsOutputUpdaterBuilder {
        SelectFilingsOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`SelectFilingsOutputUpdater`].
#[derive(Debug)]
pub struct SelectFilingsOutputUpdaterBuilder {
    updater: SelectFilingsOutputUpdater,
}

impl SelectFilingsOutputUpdaterBuilder {
    /// Creates a [`SelectFilingsOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            updater: SelectFilingsOutputUpdater { filings: None },
        }
    }

    /// Sets the selected filings.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn filings(mut self, filings: Vec<FilingRef>) -> Self {
        self.updater.filings = Some(filings);
        self
    }

    /// Builds the [`SelectFilingsOutputUpdater`].
    #[must_use]
    pub fn build(self) -> SelectFilingsOutputUpdater {
        self.updater
    }
}

impl Default for SelectFilingsOutputUpdaterBuilder {
    fn default() -> Self {
        Self::new()
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
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};

    fn filing_ref() -> FilingRef {
        FilingRef::new(
            Cik::new("320193").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000320193-26-000106"),
            ParsedForm::new(Form::TenK, false),
            NaiveDate::from_ymd_opt(2026, 10, 27)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    #[test]
    fn should_display_the_number_of_filings_for_a_select_filings_output() {
        let output = SelectFilingsOutput::new(vec![filing_ref()]);

        let expected_result = "\tSelected Filings: 1";

        let result = output.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_filings_when_update_contains_filings() {
        let mut output = SelectFilingsOutput::new(Vec::new());
        let update = SelectFilingsOutputUpdater::builder()
            .filings(vec![filing_ref()])
            .build();

        let expected_result = &SelectFilingsOutput::new(vec![filing_ref()]);

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_unchanged_when_update_is_empty() {
        let mut output = SelectFilingsOutput::new(vec![filing_ref()]);
        let update = SelectFilingsOutputUpdater::builder().build();

        let expected_result = &SelectFilingsOutput::new(vec![filing_ref()]);

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_select_filings_output() {
        implements_value_traits::<SelectFilingsOutput>();
    }
}
