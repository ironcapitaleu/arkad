//! # Select Filings Input
//!
//! Provides the [`SelectFilingsInput`] fed into the
//! [`SelectFilings`](crate::implementations::states::extract::select_filings::SelectFilings)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::implementations::states::extract::store_daily_index::StoredDailyIndex;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`SelectFilings`](super::super::SelectFilings) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct SelectFilingsInput {
    /// The stored daily index, or `None` if the day had no filings, as in
    /// [`StoreDailyIndexOutput`](crate::implementations::states::extract::store_daily_index::StoreDailyIndexOutput).
    pub index: Option<StoredDailyIndex>,
}

impl SelectFilingsInput {
    /// Creates a new [`SelectFilingsInput`]. An `index` of `None` marks a day without filings.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::select_filings::SelectFilingsInput;
    ///
    /// let input = SelectFilingsInput::new(None);
    ///
    /// let expected_result = None;
    ///
    /// let result = input.index;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(index: Option<StoredDailyIndex>) -> Self {
        Self { index }
    }
}

impl StateData for SelectFilingsInput {
    /// Replaces the stored index if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(index) = updates.index {
            self.index = index;
        }
        Ok(())
    }
}

impl SMStateData for SelectFilingsInput {
    type UpdateType = SelectFilingsInputUpdater;

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

impl fmt::Display for SelectFilingsInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.index {
            Some(index) => write!(
                f,
                "\tKey: {}, Size: {} bytes",
                index.key.as_path().display(),
                index.text.len()
            ),
            None => write!(f, "\tNo filings on this day"),
        }
    }
}

/// Updater for [`SelectFilingsInput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug)]
pub struct SelectFilingsInputUpdater {
    /// The new stored index. `Some(None)` marks a day without filings.
    pub index: Option<Option<StoredDailyIndex>>,
}

impl SelectFilingsInputUpdater {
    /// Returns a [`SelectFilingsInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> SelectFilingsInputUpdaterBuilder {
        SelectFilingsInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`SelectFilingsInputUpdater`].
#[derive(Debug)]
pub struct SelectFilingsInputUpdaterBuilder {
    updater: SelectFilingsInputUpdater,
}

impl SelectFilingsInputUpdaterBuilder {
    /// Creates a [`SelectFilingsInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            updater: SelectFilingsInputUpdater { index: None },
        }
    }

    /// Sets the stored index. `None` marks a day without filings.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn index(mut self, index: Option<StoredDailyIndex>) -> Self {
        self.updater.index = Some(index);
        self
    }

    /// Builds the [`SelectFilingsInputUpdater`].
    #[must_use]
    pub fn build(self) -> SelectFilingsInputUpdater {
        self.updater
    }
}

impl Default for SelectFilingsInputUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use bytes::Bytes;
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::StateData as SMStateData;
    use storage::implementations::repository::filesystem::DocumentKey;

    use super::*;

    fn stored_index(text: &'static [u8]) -> StoredDailyIndex {
        StoredDailyIndex {
            key: DocumentKey::new("sec/daily-index/form.20261027.idx")
                .expect("Given a hardcoded valid relative path, the key should always build"),
            text: Bytes::from_static(text),
        }
    }

    #[test]
    fn should_display_the_key_and_the_size_when_the_input_holds_an_index() {
        let input = SelectFilingsInput::new(Some(stored_index(b"Form Type")));

        let expected_result = "\tKey: sec/daily-index/form.20261027.idx, Size: 9 bytes";

        let result = input.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_no_filings_for_a_day_without_filings() {
        let input = SelectFilingsInput::new(None);

        let expected_result = "\tNo filings on this day";

        let result = input.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_index_when_update_contains_index() {
        let mut input = SelectFilingsInput::new(None);
        let update = SelectFilingsInputUpdater::builder()
            .index(Some(stored_index(b"Form Type")))
            .build();

        let expected_result = &SelectFilingsInput::new(Some(stored_index(b"Form Type")));

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_input_unchanged_when_update_is_empty() {
        let mut input = SelectFilingsInput::new(Some(stored_index(b"Form Type")));
        let update = SelectFilingsInputUpdater::builder().build();

        let expected_result = &SelectFilingsInput::new(Some(stored_index(b"Form Type")));

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_select_filings_input() {
        implements_value_traits::<SelectFilingsInput>();
    }
}
