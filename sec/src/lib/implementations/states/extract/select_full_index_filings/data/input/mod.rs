//! # Select Full Index Filings Input
//!
//! Provides the [`SelectFullIndexFilingsInput`] fed into the
//! [`SelectFullIndexFilings`](crate::implementations::states::extract::select_full_index_filings::SelectFullIndexFilings)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::stored_form_index::StoredFormIndex;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`SelectFullIndexFilings`](super::super::SelectFullIndexFilings) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct SelectFullIndexFilingsInput {
    /// The stored full index of one quarter.
    pub index: StoredFormIndex,
}

impl SelectFullIndexFilingsInput {
    /// Creates a new [`SelectFullIndexFilingsInput`] from a stored full index.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    /// use sec::implementations::states::extract::select_full_index_filings::SelectFullIndexFilingsInput;
    /// use sec::shared::stored_form_index::StoredFormIndex;
    /// use storage::Sha256Digest;
    /// use storage::implementations::repository::filesystem::DocumentKey;
    ///
    /// let index = StoredFormIndex {
    ///     key: DocumentKey::new("sec/full-index/2026/QTR4/form.idx")
    ///         .expect("Given a hardcoded valid relative path, the key should always build"),
    ///     sha256: Sha256Digest::of(b"---"),
    ///     text: Bytes::from_static(b"---"),
    /// };
    ///
    /// let expected_result = 3;
    ///
    /// let result = SelectFullIndexFilingsInput::new(index).index.text.len();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(index: StoredFormIndex) -> Self {
        Self { index }
    }
}

impl StateData for SelectFullIndexFilingsInput {
    /// Replaces the stored index if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(index) = updates.index {
            self.index = index;
        }
        Ok(())
    }
}

impl SMStateData for SelectFullIndexFilingsInput {
    type UpdateType = SelectFullIndexFilingsInputUpdater;

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

impl fmt::Display for SelectFullIndexFilingsInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tKey: {}, Size: {} bytes",
            self.index.key.as_path().display(),
            self.index.text.len()
        )
    }
}

/// Updater for [`SelectFullIndexFilingsInput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug)]
pub struct SelectFullIndexFilingsInputUpdater {
    /// The new stored index.
    pub index: Option<StoredFormIndex>,
}

impl SelectFullIndexFilingsInputUpdater {
    /// Returns a [`SelectFullIndexFilingsInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> SelectFullIndexFilingsInputUpdaterBuilder {
        SelectFullIndexFilingsInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`SelectFullIndexFilingsInputUpdater`].
#[derive(Debug, Default)]
pub struct SelectFullIndexFilingsInputUpdaterBuilder {
    index: Option<StoredFormIndex>,
}

impl SelectFullIndexFilingsInputUpdaterBuilder {
    /// Creates a [`SelectFullIndexFilingsInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self { index: None }
    }

    /// Sets the stored index.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn index(mut self, index: StoredFormIndex) -> Self {
        self.index = Some(index);
        self
    }

    /// Builds the [`SelectFullIndexFilingsInputUpdater`].
    #[must_use]
    pub fn build(self) -> SelectFullIndexFilingsInputUpdater {
        SelectFullIndexFilingsInputUpdater { index: self.index }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use bytes::Bytes;
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::StateData as SMStateData;
    use storage::Sha256Digest;
    use storage::implementations::repository::filesystem::DocumentKey;

    use super::*;

    fn stored_index(folder: &str) -> StoredFormIndex {
        StoredFormIndex {
            key: DocumentKey::new(format!("sec/full-index/2026/{folder}/form.idx"))
                .expect("Given a hardcoded valid relative path, the key should always build"),
            sha256: Sha256Digest::of(b"Form Type"),
            text: Bytes::from_static(b"Form Type"),
        }
    }

    #[test]
    fn should_update_index_when_update_contains_index() {
        let mut input = SelectFullIndexFilingsInput::new(stored_index("QTR3"));
        let update = SelectFullIndexFilingsInputUpdater::builder()
            .index(stored_index("QTR4"))
            .build();

        let expected_result = &SelectFullIndexFilingsInput::new(stored_index("QTR4"));

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_key_and_the_size_for_select_full_index_filings_input() {
        let input = SelectFullIndexFilingsInput::new(stored_index("QTR4"));

        let expected_result = "\tKey: sec/full-index/2026/QTR4/form.idx, Size: 9 bytes";

        let result = input.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_select_full_index_filings_input() {
        implements_value_traits::<SelectFullIndexFilingsInput>();
    }
}
