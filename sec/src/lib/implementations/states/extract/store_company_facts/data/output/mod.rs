//! # Store Company Facts Output
//!
//! Provides the [`StoreCompanyFactsOutput`] produced by the
//! [`StoreCompanyFacts`](crate::implementations::states::extract::store_company_facts::StoreCompanyFacts)
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

/// Output data of the [`StoreCompanyFacts`](super::super::StoreCompanyFacts) state.
///
/// Holds the [`DocumentKey`] under which the store keeps the document.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoreCompanyFactsOutput {
    /// The key under which the state stored the document.
    pub key: DocumentKey,
}

impl StoreCompanyFactsOutput {
    /// Creates a new [`StoreCompanyFactsOutput`] from the key of the stored document.
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

/// Serializes as `{ "key": "sec/companyfacts/CIK0001067983.json" }`.
impl Serialize for StoreCompanyFactsOutput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("StoreCompanyFactsOutput", 1)?;
        state.serialize_field("key", &self.key.as_path().to_string_lossy())?;
        state.end()
    }
}

impl StateData for StoreCompanyFactsOutput {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(key) = updates.key {
            self.key = key;
        }
        Ok(())
    }
}

impl SMStateData for StoreCompanyFactsOutput {
    type UpdateType = StoreCompanyFactsOutputUpdater;

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

impl fmt::Display for StoreCompanyFactsOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tKey: {}", self.key.as_path().display())
    }
}

/// Updater for modifying [`StoreCompanyFactsOutput`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoreCompanyFactsOutputUpdater {
    /// Optional new value for the key.
    pub key: Option<DocumentKey>,
}

impl StoreCompanyFactsOutputUpdater {
    /// Creates a new builder for constructing [`StoreCompanyFactsOutputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> StoreCompanyFactsOutputUpdaterBuilder {
        StoreCompanyFactsOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreCompanyFactsOutputUpdater`].
pub struct StoreCompanyFactsOutputUpdaterBuilder {
    key: Option<DocumentKey>,
}

impl StoreCompanyFactsOutputUpdaterBuilder {
    /// Creates a new [`StoreCompanyFactsOutputUpdaterBuilder`] with all fields initialized to `None`.
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

    /// Builds the [`StoreCompanyFactsOutputUpdater`].
    #[must_use]
    pub fn build(self) -> StoreCompanyFactsOutputUpdater {
        StoreCompanyFactsOutputUpdater { key: self.key }
    }
}

impl Default for StoreCompanyFactsOutputUpdaterBuilder {
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
        DocumentKey::new(format!("sec/companyfacts/CIK{cik}.json"))
            .expect("Given a relative path with no special components, the key should always build")
    }

    #[test]
    fn should_return_reference_to_self_when_accessing_state() {
        let output = StoreCompanyFactsOutput::new(sample_key("0001067983"));

        let expected_result = &output.clone();

        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_key_when_updater_contains_key() {
        let mut output = StoreCompanyFactsOutput::new(sample_key("0001067983"));
        let updater = StoreCompanyFactsOutputUpdater::builder()
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
        let mut output = StoreCompanyFactsOutput::new(sample_key("0001067983"));
        let updater = StoreCompanyFactsOutputUpdater::builder().build();

        let expected_result = StoreCompanyFactsOutput::new(sample_key("0001067983"));

        StateData::update_state(&mut output, updater)
            .expect("Given an empty update, updating the output should always succeed");
        let result = output;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_key_as_a_path_string_for_store_company_facts_output() {
        let output = StoreCompanyFactsOutput::new(sample_key("0001067983"));

        let expected_result = serde_json::json!({ "key": "sec/companyfacts/CIK0001067983.json" });

        let result = serde_json::to_value(output)
            .expect("Given an output with a UTF-8 key, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_key_when_formatted() {
        let output = StoreCompanyFactsOutput::new(sample_key("0001067983"));

        let expected_result = "\tKey: sec/companyfacts/CIK0001067983.json";

        let result = output.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_store_company_facts_output() {
        implements_auto_traits::<StoreCompanyFactsOutput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_be_thread_safe_for_store_company_facts_output() {
        implements_send::<StoreCompanyFactsOutput>();
        implements_sync::<StoreCompanyFactsOutput>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_company_facts_output() {
        implements_value_traits::<StoreCompanyFactsOutput>();
    }
}
