//! # Validate Notes Period Output
//!
//! Provides the [`ValidateNotesPeriodOutput`] produced by the
//! [`ValidateNotesPeriod`](crate::implementations::states::extract::validate_notes_period::ValidateNotesPeriod)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::notes_period::NotesPeriod;
use crate::traits::state_machine::state::StateData;

/// Output data of the [`ValidateNotesPeriod`](super::super::ValidateNotesPeriod) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct ValidateNotesPeriodOutput {
    /// The validated Notes period.
    pub notes_period: NotesPeriod,
}

impl ValidateNotesPeriodOutput {
    /// Creates a new [`ValidateNotesPeriodOutput`] from a validated Notes period.
    #[must_use]
    pub const fn new(notes_period: NotesPeriod) -> Self {
        Self { notes_period }
    }
}

impl StateData for ValidateNotesPeriodOutput {
    /// Replaces the Notes period if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(notes_period) = updates.notes_period {
            self.notes_period = notes_period;
        }
        Ok(())
    }
}

impl SMStateData for ValidateNotesPeriodOutput {
    type UpdateType = ValidateNotesPeriodOutputUpdater;

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

impl fmt::Display for ValidateNotesPeriodOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tValid Notes Period: {}", self.notes_period)
    }
}

/// Updater for [`ValidateNotesPeriodOutput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateNotesPeriodOutputUpdater {
    /// The new Notes period.
    pub notes_period: Option<NotesPeriod>,
}

impl ValidateNotesPeriodOutputUpdater {
    /// Returns a [`ValidateNotesPeriodOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> ValidateNotesPeriodOutputUpdaterBuilder {
        ValidateNotesPeriodOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`ValidateNotesPeriodOutputUpdater`].
#[derive(Debug, Default)]
pub struct ValidateNotesPeriodOutputUpdaterBuilder {
    notes_period: Option<NotesPeriod>,
}

impl ValidateNotesPeriodOutputUpdaterBuilder {
    /// Creates a [`ValidateNotesPeriodOutputUpdaterBuilder`] with no field set.
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

    /// Builds the [`ValidateNotesPeriodOutputUpdater`].
    #[must_use]
    pub const fn build(self) -> ValidateNotesPeriodOutputUpdater {
        ValidateNotesPeriodOutputUpdater {
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
        let mut output = ValidateNotesPeriodOutput::new(notes_period("2024q1"));
        let update = ValidateNotesPeriodOutputUpdater::builder()
            .notes_period(notes_period("2025_10"))
            .build();

        let expected_result = &ValidateNotesPeriodOutput::new(notes_period("2025_10"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_unchanged_when_update_is_empty() {
        let mut output = ValidateNotesPeriodOutput::new(notes_period("2024q1"));
        let update = ValidateNotesPeriodOutputUpdater::builder().build();

        let expected_result = &ValidateNotesPeriodOutput::new(notes_period("2024q1"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_validate_notes_period_output() {
        implements_value_traits::<ValidateNotesPeriodOutput>();
    }
}
