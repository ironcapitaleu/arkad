//! # Parse Company Facts Input
//!
//! Provides the [`ParseCompanyFactsInput`] fed into the
//! [`ParseCompanyFacts`](crate::implementations::states::transform::parse_company_facts::ParseCompanyFacts)
//! state, along with its updater and builder.
//!
//! It carries the parsed Company Facts JSON response, plus the [`Sha256Digest`] of the raw body, so
//! the JSON, which implements neither `Hash` nor `Ord`, can still take part in equality and ordering.
//!
//! ## See Also
//!
//! - [`output`](super::output): The structured company data produced from this input.

use std::fmt;

use state_maschine::prelude::StateData as SMStateData;
use storage::shared::Sha256Digest;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`ParseCompanyFacts`](super::super::ParseCompanyFacts) state.
///
/// Holds the Company Facts JSON to be parsed, alongside the [`Sha256Digest`] of the raw body.
///
/// The digest backs the equality, ordering, and hashing impls, because [`serde_json::Value`]
/// implements neither `Hash` nor `Ord`. Two inputs are equal if and only if their digests are.
#[derive(Debug, Clone)]
pub struct ParseCompanyFactsInput {
    /// The parsed SEC Company Facts JSON response.
    pub response: serde_json::Value,
    /// The SHA-256 digest of the raw response body.
    body_digest: Sha256Digest,
}

impl ParseCompanyFactsInput {
    /// Creates a new [`ParseCompanyFactsInput`] from a JSON response and the digest of its raw body.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::transform::parse_company_facts::data::input::ParseCompanyFactsInput;
    /// use storage::shared::Sha256Digest;
    ///
    /// let json = serde_json::json!({"cik": 320193, "entityName": "Apple Inc.", "facts": {}});
    /// let digest = Sha256Digest::of(json.to_string());
    /// let input = ParseCompanyFactsInput::new(json.clone(), digest);
    ///
    /// let expected_result = &json;
    ///
    /// let result = input.response();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(response: serde_json::Value, body_digest: Sha256Digest) -> Self {
        Self {
            response,
            body_digest,
        }
    }

    /// Returns a reference to the raw JSON response.
    #[must_use]
    pub const fn response(&self) -> &serde_json::Value {
        &self.response
    }

    /// Returns the SHA-256 digest of the raw response body.
    #[must_use]
    pub const fn body_digest(&self) -> &Sha256Digest {
        &self.body_digest
    }
}

// Compares the digest, not the response, so that `Eq` agrees with `Hash` and `Ord`.
impl PartialEq for ParseCompanyFactsInput {
    fn eq(&self, other: &Self) -> bool {
        self.body_digest == other.body_digest
    }
}

impl Eq for ParseCompanyFactsInput {}

// `serde_json::Value` does not implement `Hash`.
// Uses the `Sha256Digest` of the raw body instead of re-serializing.
impl std::hash::Hash for ParseCompanyFactsInput {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.body_digest.hash(state);
    }
}

impl PartialOrd for ParseCompanyFactsInput {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// `serde_json::Value` does not implement `Ord`.
// Uses the `Sha256Digest` of the raw body instead of re-serializing.
impl Ord for ParseCompanyFactsInput {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.body_digest.cmp(&other.body_digest)
    }
}

impl serde::Serialize for ParseCompanyFactsInput {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ParseCompanyFactsInput", 1)?;
        state.serialize_field(
            "response_keys",
            &self
                .response
                .as_object()
                .map(|o| o.keys().collect::<Vec<_>>()),
        )?;
        state.end()
    }
}

impl StateData for ParseCompanyFactsInput {
    /// Updates the state data using the provided updater.
    ///
    /// If `response` is `Some`, updates the JSON response; otherwise, leaves it unchanged.
    ///
    /// An updater holds no raw body, so the new digest is the digest of the serialized response.
    /// A stale digest makes `Eq`, `Hash`, and `Ord` describe the old response.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(response) = updates.response {
            self.body_digest = Sha256Digest::of(response.to_string());
            self.response = response;
        }
        Ok(())
    }
}

impl SMStateData for ParseCompanyFactsInput {
    type UpdateType = ParseCompanyFactsInputUpdater;

    /// Returns a reference to the current state data, which represents the input data of this state.
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

impl fmt::Display for ParseCompanyFactsInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tResponse Keys: {:?}",
            self.response
                .as_object()
                .map(|o| o.keys().collect::<Vec<_>>())
                .unwrap_or_default()
        )
    }
}

#[derive(Debug, Clone)]
/// Updater for modifying [`ParseCompanyFactsInput`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
pub struct ParseCompanyFactsInputUpdater {
    /// Optional new value for the JSON response.
    pub response: Option<serde_json::Value>,
}

impl ParseCompanyFactsInputUpdater {
    /// Creates a new builder for constructing [`ParseCompanyFactsInputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ParseCompanyFactsInputUpdaterBuilder {
        ParseCompanyFactsInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`ParseCompanyFactsInputUpdater`].
pub struct ParseCompanyFactsInputUpdaterBuilder {
    response: Option<serde_json::Value>,
}

impl ParseCompanyFactsInputUpdaterBuilder {
    /// Creates a new [`ParseCompanyFactsInputUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self { response: None }
    }

    /// Sets the response field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn response(mut self, response: serde_json::Value) -> Self {
        self.response = Some(response);
        self
    }

    /// Builds the [`ParseCompanyFactsInputUpdater`].
    #[must_use]
    pub fn build(self) -> ParseCompanyFactsInputUpdater {
        ParseCompanyFactsInputUpdater {
            response: self.response,
        }
    }
}

impl Default for ParseCompanyFactsInputUpdaterBuilder {
    /// Returns a new [`ParseCompanyFactsInputUpdaterBuilder`] with all fields initialized to `None`.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::StateData as SMStateData;
    use storage::shared::Sha256Digest;

    use super::{ParseCompanyFactsInput, ParseCompanyFactsInputUpdaterBuilder};
    use crate::traits::state_machine::state::StateData;

    fn test_input() -> ParseCompanyFactsInput {
        let json = serde_json::json!({});
        let digest = Sha256Digest::of(json.to_string());
        ParseCompanyFactsInput::new(json, digest)
    }

    #[test]
    fn should_return_reference_to_input_data_when_initialized_with_test_input() {
        let input_data = test_input();

        let expected_result = &test_input();

        let result = input_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_input_data_with_custom_json_when_using_new_as_constructor() {
        let json = serde_json::json!({"cik": 320_193});
        let digest = Sha256Digest::of(json.to_string());
        let input_data = &ParseCompanyFactsInput::new(json, digest);

        let default_input_data = &test_input();

        let result = input_data.state();

        assert_ne!(result, default_input_data);
    }

    #[test]
    fn should_update_state_data_when_update_contains_new_response() {
        let mut state_data = test_input();
        let new_json = serde_json::json!({"cik": 12345});
        let update = ParseCompanyFactsInputUpdaterBuilder::default()
            .response(new_json.clone())
            .build();

        let digest = Sha256Digest::of(new_json.to_string());
        let expected_result = &ParseCompanyFactsInput::new(new_json, digest);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_compare_unequal_when_two_inputs_hold_one_response_with_different_digests() {
        // Both raw bodies parse to the same JSON. The second body has spaces, so its SHA-256
        // differs, and `Eq` compares only the digest.
        let json = serde_json::json!({"cik": 320_193});
        let compact_input =
            ParseCompanyFactsInput::new(json.clone(), Sha256Digest::of(br#"{"cik":320193}"#));

        let expected_result = compact_input;

        let result = ParseCompanyFactsInput::new(json, Sha256Digest::of(br#"{ "cik": 320193 }"#));

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = test_input();
        let empty_update = ParseCompanyFactsInputUpdaterBuilder::default().build();

        let expected_result = &test_input();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_parse_company_facts_input() {
        implements_auto_traits::<ParseCompanyFactsInput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_parse_company_facts_input() {
        implements_send::<ParseCompanyFactsInput>();
    }

    #[test]
    const fn should_implement_sync_for_parse_company_facts_input() {
        implements_sync::<ParseCompanyFactsInput>();
    }

    #[test]
    const fn should_be_thread_safe_for_parse_company_facts_input() {
        implements_send::<ParseCompanyFactsInput>();
        implements_sync::<ParseCompanyFactsInput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_parse_company_facts_input() {
        implements_sized::<ParseCompanyFactsInput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_parse_company_facts_input() {
        implements_hash::<ParseCompanyFactsInput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_parse_company_facts_input() {
        implements_partial_eq::<ParseCompanyFactsInput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_parse_company_facts_input() {
        implements_eq::<ParseCompanyFactsInput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_parse_company_facts_input() {
        implements_partial_ord::<ParseCompanyFactsInput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_parse_company_facts_input() {
        implements_ord::<ParseCompanyFactsInput>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_parse_company_facts_input() {
        implements_debug::<ParseCompanyFactsInput>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_parse_company_facts_input() {
        implements_clone::<ParseCompanyFactsInput>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_parse_company_facts_input() {
        implements_unpin::<ParseCompanyFactsInput>();
    }
}
