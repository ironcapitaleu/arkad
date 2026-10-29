//! # Validate Notes Period Input
//!
//! Provides the [`ValidateNotesPeriodInput`] fed into the
//! [`ValidateNotesPeriod`](crate::implementations::states::extract::validate_notes_period::ValidateNotesPeriod)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`ValidateNotesPeriod`](super::super::ValidateNotesPeriod) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct ValidateNotesPeriodInput {
    /// The unvalidated period string, for example `2024q1`.
    pub raw_period: String,
}

impl ValidateNotesPeriodInput {
    /// Creates a new [`ValidateNotesPeriodInput`] from a raw period string.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::validate_notes_period::ValidateNotesPeriodInput;
    ///
    /// let input = ValidateNotesPeriodInput::new("2024q1");
    ///
    /// let expected_result = "2024q1";
    ///
    /// let result = input.raw_period.as_str();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn new(raw_period: impl Into<String>) -> Self {
        Self {
            raw_period: raw_period.into(),
        }
    }
}

impl StateData for ValidateNotesPeriodInput {
    /// Replaces the raw period if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(raw_period) = updates.raw_period {
            self.raw_period = raw_period;
        }
        Ok(())
    }
}

impl SMStateData for ValidateNotesPeriodInput {
    type UpdateType = ValidateNotesPeriodInputUpdater;

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

impl fmt::Display for ValidateNotesPeriodInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tNotes Period: {}", self.raw_period)
    }
}

/// Updater for [`ValidateNotesPeriodInput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateNotesPeriodInputUpdater {
    /// The new raw period string.
    pub raw_period: Option<String>,
}

impl ValidateNotesPeriodInputUpdater {
    /// Returns a [`ValidateNotesPeriodInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> ValidateNotesPeriodInputUpdaterBuilder {
        ValidateNotesPeriodInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`ValidateNotesPeriodInputUpdater`].
#[derive(Debug, Default)]
pub struct ValidateNotesPeriodInputUpdaterBuilder {
    raw_period: Option<String>,
}

impl ValidateNotesPeriodInputUpdaterBuilder {
    /// Creates a [`ValidateNotesPeriodInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self { raw_period: None }
    }

    /// Sets the raw period.
    #[must_use]
    pub fn raw_period(mut self, raw_period: impl Into<String>) -> Self {
        self.raw_period = Some(raw_period.into());
        self
    }

    /// Builds the [`ValidateNotesPeriodInputUpdater`].
    #[must_use]
    pub fn build(self) -> ValidateNotesPeriodInputUpdater {
        ValidateNotesPeriodInputUpdater {
            raw_period: self.raw_period,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use state_maschine::prelude::StateData as SMStateData;

    use super::*;

    #[test]
    fn should_update_raw_period_when_update_contains_raw_period() {
        let mut input = ValidateNotesPeriodInput::new("2024q1");
        let update = ValidateNotesPeriodInputUpdater::builder()
            .raw_period("2025_10")
            .build();

        let expected_result = &ValidateNotesPeriodInput::new("2025_10");

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_input_unchanged_when_update_is_empty() {
        let mut input = ValidateNotesPeriodInput::new("2024q1");
        let update = ValidateNotesPeriodInputUpdater::builder().build();

        let expected_result = &ValidateNotesPeriodInput::new("2024q1");

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_validate_notes_period_input() {
        implements_value_traits::<ValidateNotesPeriodInput>();
    }
}
