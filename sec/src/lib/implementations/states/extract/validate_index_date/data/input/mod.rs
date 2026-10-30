//! # Validate Index Date Input
//!
//! Provides the [`ValidateIndexDateInput`] fed into the
//! [`ValidateIndexDate`](crate::implementations::states::extract::validate_index_date::ValidateIndexDate)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`ValidateIndexDate`](super::super::ValidateIndexDate) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct ValidateIndexDateInput {
    /// The unvalidated date string, for example `2026-10-27`.
    pub raw_date: String,
}

impl ValidateIndexDateInput {
    /// Creates a new [`ValidateIndexDateInput`] from a raw date string.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::validate_index_date::ValidateIndexDateInput;
    ///
    /// let input = ValidateIndexDateInput::new("2026-10-27");
    ///
    /// let expected_result = "2026-10-27";
    ///
    /// let result = input.raw_date.as_str();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn new(raw_date: impl Into<String>) -> Self {
        Self {
            raw_date: raw_date.into(),
        }
    }
}

impl StateData for ValidateIndexDateInput {
    /// Replaces the raw date if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(raw_date) = updates.raw_date {
            self.raw_date = raw_date;
        }
        Ok(())
    }
}

impl SMStateData for ValidateIndexDateInput {
    type UpdateType = ValidateIndexDateInputUpdater;

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

impl fmt::Display for ValidateIndexDateInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tIndex Date: {}", self.raw_date)
    }
}

/// Updater for [`ValidateIndexDateInput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateIndexDateInputUpdater {
    /// The new raw date string.
    pub raw_date: Option<String>,
}

impl ValidateIndexDateInputUpdater {
    /// Returns a [`ValidateIndexDateInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> ValidateIndexDateInputUpdaterBuilder {
        ValidateIndexDateInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`ValidateIndexDateInputUpdater`].
#[derive(Debug, Default)]
pub struct ValidateIndexDateInputUpdaterBuilder {
    raw_date: Option<String>,
}

impl ValidateIndexDateInputUpdaterBuilder {
    /// Creates a [`ValidateIndexDateInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self { raw_date: None }
    }

    /// Sets the raw date.
    #[must_use]
    pub fn raw_date(mut self, raw_date: impl Into<String>) -> Self {
        self.raw_date = Some(raw_date.into());
        self
    }

    /// Builds the [`ValidateIndexDateInputUpdater`].
    #[must_use]
    pub fn build(self) -> ValidateIndexDateInputUpdater {
        ValidateIndexDateInputUpdater {
            raw_date: self.raw_date,
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
    fn should_update_raw_date_when_update_contains_raw_date() {
        let mut input = ValidateIndexDateInput::new("2026-10-27");
        let update = ValidateIndexDateInputUpdater::builder()
            .raw_date("2026-03-02")
            .build();

        let expected_result = &ValidateIndexDateInput::new("2026-03-02");

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_input_unchanged_when_update_is_empty() {
        let mut input = ValidateIndexDateInput::new("2026-10-27");
        let update = ValidateIndexDateInputUpdater::builder().build();

        let expected_result = &ValidateIndexDateInput::new("2026-10-27");

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_validate_index_date_input() {
        implements_value_traits::<ValidateIndexDateInput>();
    }
}
