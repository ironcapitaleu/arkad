//! # Store Filing Index Output
//!
//! Provides the [`StoreFilingIndexOutput`] produced by the
//! [`StoreFilingIndex`](crate::implementations::states::extract::store_filing_index::StoreFilingIndex)
//! state, along with its updater and builder.
//!
//! It holds the [`DocumentKey`] under which the state stored the document.

use std::fmt;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use state_maschine::prelude::StateData as SMStateData;
use storage::implementations::repository::filesystem::DocumentKey;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

/// Output data of the [`StoreFilingIndex`](super::super::StoreFilingIndex) state.
///
/// Holds the [`DocumentKey`] under which the store keeps the document.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoreFilingIndexOutput {
    /// The key under which the state stored the document.
    pub key: DocumentKey,
}

impl StoreFilingIndexOutput {
    /// Creates a new [`StoreFilingIndexOutput`] from the key of the stored document.
    #[must_use]
    pub const fn new(key: DocumentKey) -> Self {
        Self { key }
    }

    /// Returns a reference to the key of the stored document.
    #[must_use]
    pub const fn key(&self) -> &DocumentKey {
        &self.key
    }
}

/// Serializes as `{ "key": "sec/filings/CIK0001067983/0001067983-26-000001/index.json" }`.
impl Serialize for StoreFilingIndexOutput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("StoreFilingIndexOutput", 1)?;
        state.serialize_field("key", &self.key.as_path().to_string_lossy())?;
        state.end()
    }
}

impl StateData for StoreFilingIndexOutput {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(key) = updates.key {
            self.key = key;
        }
        Ok(())
    }
}

impl SMStateData for StoreFilingIndexOutput {
    type UpdateType = StoreFilingIndexOutputUpdater;

    fn state(&self) -> &Self {
        self
    }

    /// Delegates to the SEC [`StateData::update_state`] implementation.
    ///
    /// # Panics
    /// Panics if the fallible SEC update returns an error.
    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Err(e) = <Self as StateData>::update_state(self, updates) {
            panic!("StateData::update_state failed: {e}")
        }
    }
}

impl fmt::Display for StoreFilingIndexOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tKey: {}", self.key.as_path().display())
    }
}

/// Updater for modifying [`StoreFilingIndexOutput`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoreFilingIndexOutputUpdater {
    /// Optional new value for the key.
    pub key: Option<DocumentKey>,
}

impl StoreFilingIndexOutputUpdater {
    /// Creates a new builder for constructing [`StoreFilingIndexOutputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> StoreFilingIndexOutputUpdaterBuilder {
        StoreFilingIndexOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreFilingIndexOutputUpdater`].
pub struct StoreFilingIndexOutputUpdaterBuilder {
    key: Option<DocumentKey>,
}

impl StoreFilingIndexOutputUpdaterBuilder {
    /// Creates a new [`StoreFilingIndexOutputUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self { key: None }
    }

    /// Sets the key field.
    #[must_use]
    pub fn key(mut self, key: DocumentKey) -> Self {
        self.key = Some(key);
        self
    }

    /// Builds the [`StoreFilingIndexOutputUpdater`].
    #[must_use]
    pub fn build(self) -> StoreFilingIndexOutputUpdater {
        StoreFilingIndexOutputUpdater { key: self.key }
    }
}

impl Default for StoreFilingIndexOutputUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;

    fn sample_key(cik: &str) -> DocumentKey {
        DocumentKey::new(format!("sec/filings/CIK{cik}/{cik}-26-000001/index.json"))
            .expect("Given a relative path with no special components, the key should always build")
    }

    #[test]
    fn should_return_reference_to_self_when_accessing_state() {
        let output = StoreFilingIndexOutput::new(sample_key("0001067983"));

        let expected_result = &output.clone();

        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_key_when_updater_contains_key() {
        let mut output = StoreFilingIndexOutput::new(sample_key("0001067983"));
        let updater = StoreFilingIndexOutputUpdater::builder()
            .key(sample_key("0000320193"))
            .build();

        let expected_result = &sample_key("0000320193");

        StateData::update_state(&mut output, updater)
            .expect("Given a non-empty update, updating the output should always succeed");
        let result = output.key();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_unchanged_when_updater_is_empty() {
        let mut output = StoreFilingIndexOutput::new(sample_key("0001067983"));
        let updater = StoreFilingIndexOutputUpdater::builder().build();

        let expected_result = StoreFilingIndexOutput::new(sample_key("0001067983"));

        StateData::update_state(&mut output, updater)
            .expect("Given an empty update, updating the output should always succeed");
        let result = output;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_key_as_a_path_string_for_store_filing_index_output() {
        let output = StoreFilingIndexOutput::new(sample_key("0001067983"));

        let expected_result = serde_json::json!({ "key": "sec/filings/CIK0001067983/0001067983-26-000001/index.json" });

        let result = serde_json::to_value(output)
            .expect("Given an output with a UTF-8 key, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_key_when_formatted() {
        let output = StoreFilingIndexOutput::new(sample_key("0001067983"));

        let expected_result = "\tKey: sec/filings/CIK0001067983/0001067983-26-000001/index.json";

        let result = output.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_store_filing_index_output() {
        implements_auto_traits::<StoreFilingIndexOutput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_be_thread_safe_for_store_filing_index_output() {
        implements_send::<StoreFilingIndexOutput>();
        implements_sync::<StoreFilingIndexOutput>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_filing_index_output() {
        implements_value_traits::<StoreFilingIndexOutput>();
    }
}
