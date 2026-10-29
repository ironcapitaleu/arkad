//! # Prepare Notes Request Input
//!
//! Provides the [`PrepareNotesRequestInput`] fed into the
//! [`PrepareNotesRequest`](crate::implementations::states::extract::prepare_notes_request::PrepareNotesRequest)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::notes_period::NotesPeriod;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`PrepareNotesRequest`](super::super::PrepareNotesRequest) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct PrepareNotesRequestInput {
    /// The period of the Notes file to request.
    pub notes_period: NotesPeriod,
}

impl PrepareNotesRequestInput {
    /// Creates a new [`PrepareNotesRequestInput`] from the period of the Notes file.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::prepare_notes_request::PrepareNotesRequestInput;
    /// use sec::shared::notes_period::NotesPeriod;
    ///
    /// let period = NotesPeriod::new("2024q1")
    ///     .expect("Given a hardcoded published quarter, parsing should always succeed");
    /// let data = PrepareNotesRequestInput::new(period);
    ///
    /// let expected_result = period;
    ///
    /// let result = data.notes_period;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(notes_period: NotesPeriod) -> Self {
        Self { notes_period }
    }
}

impl StateData for PrepareNotesRequestInput {
    /// Replaces the Notes period if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(notes_period) = updates.notes_period {
            self.notes_period = notes_period;
        }
        Ok(())
    }
}

impl SMStateData for PrepareNotesRequestInput {
    type UpdateType = PrepareNotesRequestInputUpdater;

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

impl fmt::Display for PrepareNotesRequestInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tNotes Period: {}", self.notes_period)
    }
}

/// Updater for [`PrepareNotesRequestInput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareNotesRequestInputUpdater {
    /// The new Notes period.
    pub notes_period: Option<NotesPeriod>,
}

impl PrepareNotesRequestInputUpdater {
    /// Returns a [`PrepareNotesRequestInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> PrepareNotesRequestInputUpdaterBuilder {
        PrepareNotesRequestInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`PrepareNotesRequestInputUpdater`].
#[derive(Debug, Default)]
pub struct PrepareNotesRequestInputUpdaterBuilder {
    notes_period: Option<NotesPeriod>,
}

impl PrepareNotesRequestInputUpdaterBuilder {
    /// Creates a [`PrepareNotesRequestInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self { notes_period: None }
    }

    /// Sets the Notes period.
    #[must_use]
    pub const fn notes_period(mut self, notes_period: NotesPeriod) -> Self {
        self.notes_period = Some(notes_period);
        self
    }

    /// Builds the [`PrepareNotesRequestInputUpdater`].
    #[must_use]
    pub const fn build(self) -> PrepareNotesRequestInputUpdater {
        PrepareNotesRequestInputUpdater {
            notes_period: self.notes_period,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use state_maschine::prelude::StateData as SMStateData;

    use super::*;

    fn notes_period(period: &str) -> NotesPeriod {
        NotesPeriod::new(period)
            .expect("Given a hardcoded published period, parsing should always succeed")
    }

    #[test]
    fn should_update_notes_period_when_update_contains_notes_period() {
        let mut input = PrepareNotesRequestInput::new(notes_period("2024q1"));
        let update = PrepareNotesRequestInputUpdater::builder()
            .notes_period(notes_period("2025_10"))
            .build();

        let expected_result = &PrepareNotesRequestInput::new(notes_period("2025_10"));

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_input_unchanged_when_update_is_empty() {
        let mut input = PrepareNotesRequestInput::new(notes_period("2024q1"));
        let update = PrepareNotesRequestInputUpdater::builder().build();

        let expected_result = &PrepareNotesRequestInput::new(notes_period("2024q1"));

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_prepare_notes_request_input() {
        implements_value_traits::<PrepareNotesRequestInput>();
    }
}
