//! # Validate Index Date Output
//!
//! Provides the [`ValidateIndexDateOutput`] produced by the
//! [`ValidateIndexDate`](crate::implementations::states::extract::validate_index_date::ValidateIndexDate)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::index_date::IndexDate;
use crate::traits::state_machine::state::StateData;

/// Output data of the [`ValidateIndexDate`](super::super::ValidateIndexDate) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct ValidateIndexDateOutput {
    /// The validated index date.
    pub index_date: IndexDate,
}

impl ValidateIndexDateOutput {
    /// Creates a new [`ValidateIndexDateOutput`] from a validated index date.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::validate_index_date::ValidateIndexDateOutput;
    /// use sec::shared::index_date::IndexDate;
    ///
    /// let date = IndexDate::new("2026-10-27")
    ///     .expect("Given a hardcoded past date, parsing should always succeed");
    /// let data = ValidateIndexDateOutput::new(date);
    ///
    /// let expected_result = date;
    ///
    /// let result = data.index_date;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(index_date: IndexDate) -> Self {
        Self { index_date }
    }
}

impl StateData for ValidateIndexDateOutput {
    /// Replaces the index date if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(index_date) = updates.index_date {
            self.index_date = index_date;
        }
        Ok(())
    }
}

impl SMStateData for ValidateIndexDateOutput {
    type UpdateType = ValidateIndexDateOutputUpdater;

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

impl fmt::Display for ValidateIndexDateOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tValid Index Date: {}", self.index_date)
    }
}

/// Updater for [`ValidateIndexDateOutput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateIndexDateOutputUpdater {
    /// The new index date.
    pub index_date: Option<IndexDate>,
}

impl ValidateIndexDateOutputUpdater {
    /// Returns a [`ValidateIndexDateOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> ValidateIndexDateOutputUpdaterBuilder {
        ValidateIndexDateOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`ValidateIndexDateOutputUpdater`].
#[derive(Debug, Default)]
pub struct ValidateIndexDateOutputUpdaterBuilder {
    index_date: Option<IndexDate>,
}

impl ValidateIndexDateOutputUpdaterBuilder {
    /// Creates a [`ValidateIndexDateOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self { index_date: None }
    }

    /// Sets the index date.
    #[must_use]
    pub const fn index_date(mut self, index_date: IndexDate) -> Self {
        self.index_date = Some(index_date);
        self
    }

    /// Builds the [`ValidateIndexDateOutputUpdater`].
    #[must_use]
    pub const fn build(self) -> ValidateIndexDateOutputUpdater {
        ValidateIndexDateOutputUpdater {
            index_date: self.index_date,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use state_maschine::prelude::StateData as SMStateData;

    use super::*;

    fn index_date(date: &str) -> IndexDate {
        IndexDate::new(date).expect("Given a hardcoded past date, parsing should always succeed")
    }

    #[test]
    fn should_update_index_date_when_update_contains_index_date() {
        let mut output = ValidateIndexDateOutput::new(index_date("2026-10-27"));
        let update = ValidateIndexDateOutputUpdater::builder()
            .index_date(index_date("2026-03-02"))
            .build();

        let expected_result = &ValidateIndexDateOutput::new(index_date("2026-03-02"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_unchanged_when_update_is_empty() {
        let mut output = ValidateIndexDateOutput::new(index_date("2026-10-27"));
        let update = ValidateIndexDateOutputUpdater::builder().build();

        let expected_result = &ValidateIndexDateOutput::new(index_date("2026-10-27"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_validate_index_date_output() {
        implements_value_traits::<ValidateIndexDateOutput>();
    }
}
