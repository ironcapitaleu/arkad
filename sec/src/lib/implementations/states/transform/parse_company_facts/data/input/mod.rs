//! # `ParseCompanyFactsInput` Module
//!
//! This module defines the input data structure and updater patterns for the `ParseCompanyFacts` state
//! within the SEC transformation state machine. It provides types and builders for representing and updating
//! the raw JSON response from the SEC Company Facts API that will be parsed into structured financial data.
//!
//! ## Types
//! - [`ParseCompanyFactsInput`]: Holds the raw SEC Company Facts JSON response to be parsed.
//! - [`ParseCompanyFactsInputUpdater`]: Updater type for modifying the input data in a controlled manner.
//! - [`ParseCompanyFactsInputUpdaterBuilder`]: Builder for constructing updater instances with optional fields.
//!
//! ## Integration
//! - Implements [`StateData`](state_maschine::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`ParseCompanyFacts`](crate::implementations::states::transform::parse_company_facts) to receive and update JSON input.
//!
//! ## Usage
//! This module is intended for use in the input phase of company facts parsing. It supports builder-based
//! updates and integrates with the state machine's updater and state data traits for robust, testable workflows.
//!
//! ## See Also
//! - [`output`](super::output): Output data structure for parsed company data.
//! - [`state_maschine::prelude::StateData`]: Trait for state data integration.
//!
//! ## Examples
//! See the unit tests in this module for usage patterns and updater logic.

use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::response::implementations::sec_response::body_digest::BodyDigest;
use crate::traits::state_machine::state::StateData;

/// Input data for parsing SEC Company Facts JSON.
///
/// This struct holds the raw JSON response from the SEC Company Facts API
/// that will be parsed by the `ParseCompanyFacts` state. It is designed
/// to be used as part of the SEC transformation workflow, and supports
/// builder-based updates and integration with the state machine framework.
#[derive(Debug, Clone)]
pub struct ParseCompanyFactsInput {
    /// The raw SEC Company Facts JSON response to be parsed.
    pub response: serde_json::Value,
    /// Precomputed digest of the response body for efficient hashing and ordering.
    body_digest: BodyDigest,
}

impl ParseCompanyFactsInput {
    /// Creates a new instance of the input data for the company facts parsing.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::transform::parse_company_facts::data::input::ParseCompanyFactsInput;
    /// use sec::shared::response::implementations::sec_response::body_digest::BodyDigest;
    ///
    /// let json = serde_json::json!({"cik": 320193, "entityName": "Apple Inc.", "facts": {}});
    /// let digest = BodyDigest::from_body_text(&json.to_string());
    /// let input = ParseCompanyFactsInput::new(json, digest);
    /// ```
    #[must_use]
    pub const fn new(response: serde_json::Value, body_digest: BodyDigest) -> Self {
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

    /// Returns the precomputed body digest.
    #[must_use]
    pub const fn body_digest(&self) -> BodyDigest {
        self.body_digest
    }
}

impl PartialEq for ParseCompanyFactsInput {
    fn eq(&self, other: &Self) -> bool {
        self.response == other.response
    }
}

impl Eq for ParseCompanyFactsInput {}

// `serde_json::Value` does not implement `Hash`.
// Uses the precomputed `BodyDigest` instead of re-serializing.
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
// Uses the precomputed `BodyDigest` instead of re-serializing.
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
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(response) = updates.response {
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
/// Updater for [`ParseCompanyFactsInput`].
///
/// This struct is used to specify updates to the input data in a controlled, partial manner.
/// Fields set to `None` will not be updated.
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

/// Builder for [`ParseCompanyFactsInputUpdater`].
///
/// This builder allows for ergonomic and explicit construction of updater instances,
/// supporting method chaining and optional fields. Use `.build()` to produce the updater.
pub struct ParseCompanyFactsInputUpdaterBuilder {
    response: Option<serde_json::Value>,
}

impl ParseCompanyFactsInputUpdaterBuilder {
    /// Creates a new updater builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self { response: None }
    }

    /// Sets the response value to the one to be updated to.
    ///
    /// # Arguments
    ///
    /// * `response` - The new JSON response value.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn response(mut self, response: serde_json::Value) -> Self {
        self.response = Some(response);
        self
    }

    /// Builds the updater instance from the builder.
    #[must_use]
    pub fn build(self) -> ParseCompanyFactsInputUpdater {
        ParseCompanyFactsInputUpdater {
            response: self.response,
        }
    }
}

impl Default for ParseCompanyFactsInputUpdaterBuilder {
    /// Returns a new updater builder with no fields set.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::{assert_eq, assert_ne};

    use super::{ParseCompanyFactsInput, ParseCompanyFactsInputUpdaterBuilder};
    use crate::shared::response::implementations::sec_response::body_digest::BodyDigest;
    use crate::traits::state_machine::state::StateData;
    use state_maschine::prelude::StateData as SMStateData;

    fn test_input() -> ParseCompanyFactsInput {
        let json = serde_json::json!({});
        let digest = BodyDigest::from_body_text(&json.to_string());
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
        let digest = BodyDigest::from_body_text(&json.to_string());
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

        let digest = BodyDigest::from_body_text(&new_json.to_string());
        let expected_result = &ParseCompanyFactsInput::new(new_json, digest);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
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
    const fn should_still_implement_auto_traits_when_implementing_input_data_trait() {
        implements_auto_traits::<ParseCompanyFactsInput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_input_data_trait() {
        implements_send::<ParseCompanyFactsInput>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_input_data_trait() {
        implements_sync::<ParseCompanyFactsInput>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_input_data_trait() {
        implements_send::<ParseCompanyFactsInput>();
        implements_sync::<ParseCompanyFactsInput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_input_data_trait() {
        implements_sized::<ParseCompanyFactsInput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_input_data_trait() {
        implements_hash::<ParseCompanyFactsInput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_input_data_trait() {
        implements_partial_eq::<ParseCompanyFactsInput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_input_data_trait() {
        implements_eq::<ParseCompanyFactsInput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_input_data_trait() {
        implements_partial_ord::<ParseCompanyFactsInput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_input_data_trait() {
        implements_ord::<ParseCompanyFactsInput>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_input_data_trait() {
        implements_debug::<ParseCompanyFactsInput>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_input_data_trait() {
        implements_clone::<ParseCompanyFactsInput>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_input_data_trait() {
        implements_unpin::<ParseCompanyFactsInput>();
    }
}
