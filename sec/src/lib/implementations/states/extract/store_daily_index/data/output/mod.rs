//! # Store Daily Index Output
//!
//! Provides the [`StoreDailyIndexOutput`] produced by the
//! [`StoreDailyIndex`](crate::implementations::states::extract::store_daily_index::StoreDailyIndex)
//! state, the [`StoredDailyIndex`] it holds, and the output's updater and builder.

use std::fmt;

use bytes::Bytes;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use state_maschine::prelude::StateData as SMStateData;
use storage::implementations::repository::filesystem::DocumentKey;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

/// The key and the text of one stored daily index.
///
/// The text shares the body buffer of the downloaded response, so it holds the bytes exactly as
/// the SEC sent them.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoredDailyIndex {
    /// The key the index is stored under.
    pub key: DocumentKey,
    /// The text of the index, as the SEC sent it.
    pub text: Bytes,
}

/// Serializes the key and the size of the text, without the text.
impl Serialize for StoredDailyIndex {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("StoredDailyIndex", 2)?;
        state.serialize_field("key", &self.key)?;
        state.serialize_field("size_in_bytes", &self.text.len())?;
        state.end()
    }
}

/// Output data of the [`StoreDailyIndex`](super::super::StoreDailyIndex) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct StoreDailyIndexOutput {
    /// The stored daily index, or `None` if the day had no filings.
    pub index: Option<StoredDailyIndex>,
}

impl StoreDailyIndexOutput {
    /// Creates a new [`StoreDailyIndexOutput`] from a stored daily index.
    #[must_use]
    pub const fn new(index: StoredDailyIndex) -> Self {
        Self { index: Some(index) }
    }

    /// Creates a new [`StoreDailyIndexOutput`] for a day without filings.
    #[must_use]
    pub const fn without_filings() -> Self {
        Self { index: None }
    }
}

impl StateData for StoreDailyIndexOutput {
    /// Replaces the stored index if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(index) = updates.index {
            self.index = index;
        }
        Ok(())
    }
}

impl SMStateData for StoreDailyIndexOutput {
    type UpdateType = StoreDailyIndexOutputUpdater;

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

impl fmt::Display for StoreDailyIndexOutput {
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

/// Updater for [`StoreDailyIndexOutput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug)]
pub struct StoreDailyIndexOutputUpdater {
    /// The new stored index. `Some(None)` marks a day without filings.
    pub index: Option<Option<StoredDailyIndex>>,
}

impl StoreDailyIndexOutputUpdater {
    /// Returns a [`StoreDailyIndexOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> StoreDailyIndexOutputUpdaterBuilder {
        StoreDailyIndexOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreDailyIndexOutputUpdater`].
#[derive(Debug)]
pub struct StoreDailyIndexOutputUpdaterBuilder {
    updater: StoreDailyIndexOutputUpdater,
}

impl StoreDailyIndexOutputUpdaterBuilder {
    /// Creates a [`StoreDailyIndexOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            updater: StoreDailyIndexOutputUpdater { index: None },
        }
    }

    /// Sets the stored index. `None` marks a day without filings.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn index(mut self, index: Option<StoredDailyIndex>) -> Self {
        self.updater.index = Some(index);
        self
    }

    /// Builds the [`StoreDailyIndexOutputUpdater`].
    #[must_use]
    pub fn build(self) -> StoreDailyIndexOutputUpdater {
        self.updater
    }
}

impl Default for StoreDailyIndexOutputUpdaterBuilder {
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

    fn stored_index(text: &'static [u8]) -> StoredDailyIndex {
        StoredDailyIndex {
            key: DocumentKey::new("sec/daily-index/form.20261027.idx")
                .expect("Given a hardcoded valid relative path, the key should always build"),
            text: Bytes::from_static(text),
        }
    }

    #[test]
    fn should_serialize_the_key_and_the_size_without_the_text_for_a_stored_daily_index() {
        let output = StoreDailyIndexOutput::new(stored_index(b"Form Type"));

        let expected_result = serde_json::json!({
            "index": { "key": "sec/daily-index/form.20261027.idx", "size_in_bytes": 9 }
        });

        let result = serde_json::to_value(&output)
            .expect("Given a key and a size, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_key_and_the_size_when_the_output_holds_an_index() {
        let output = StoreDailyIndexOutput::new(stored_index(b"Form Type"));

        let expected_result = "\tKey: sec/daily-index/form.20261027.idx, Size: 9 bytes";

        let result = output.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_no_filings_for_a_day_without_filings() {
        let output = StoreDailyIndexOutput::without_filings();

        let expected_result = "\tNo filings on this day";

        let result = output.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_index_when_update_contains_index() {
        let mut output = StoreDailyIndexOutput::without_filings();
        let update = StoreDailyIndexOutputUpdater::builder()
            .index(Some(stored_index(b"Form Type")))
            .build();

        let expected_result = &StoreDailyIndexOutput::new(stored_index(b"Form Type"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_clear_the_index_when_update_marks_a_day_without_filings() {
        let mut output = StoreDailyIndexOutput::new(stored_index(b"Form Type"));
        let update = StoreDailyIndexOutputUpdater::builder().index(None).build();

        let expected_result = &StoreDailyIndexOutput::without_filings();

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_unchanged_when_update_is_empty() {
        let mut output = StoreDailyIndexOutput::new(stored_index(b"Form Type"));
        let update = StoreDailyIndexOutputUpdater::builder().build();

        let expected_result = &StoreDailyIndexOutput::new(stored_index(b"Form Type"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_daily_index_output() {
        implements_value_traits::<StoreDailyIndexOutput>();
    }
}
