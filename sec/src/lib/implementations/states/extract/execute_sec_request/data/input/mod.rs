//! # `ExecuteSecRequestInput` Module
//!
//! This module defines the input data structure and updater patterns for the `ExecuteSecRequest` state
//! within the SEC extraction state machine. It provides types and builders for representing and updating
//! the prepared SEC client and request objects required for executing HTTP requests to SEC API endpoints.
//!
//! ## Types
//! - [`ExecuteSecRequestInput`]: Holds the prepared SEC client and request to be executed by the execute request state.
//! - [`ExecuteSecRequestInputUpdater`]: Updater type for modifying the input data in a controlled manner.
//! - [`ExecuteSecRequestInputUpdaterBuilder`]: Builder for constructing updater instances with optional fields.
//!
//! ## Integration
//! - Implements [`StateData`](state_maschine::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request) to receive and update execution parameters.
//!
//! ## Usage
//! This module is intended for use in the execution phase of SEC API requests. It supports builder-based updates and
//! integrates with the state machine's updater and state data traits for robust, testable workflows.
//!
//! ## See Also
//! - [`crate::shared::sec_client`]: Utilities for SEC client creation and management.
//! - [`crate::shared::sec_request`]: Utilities for SEC request construction.
//! - [`state_maschine::prelude::StateData`]: Trait for state data integration.
//!
//! ## Examples
//! See the unit tests in this module for usage patterns and updater logic.

use std::fmt;

use crate::error::State as StateError;
use crate::shared::sec_client::SecClient;
use crate::shared::sec_request::SecRequest;
use crate::traits::state_machine::state::StateData;

use state_maschine::prelude::StateData as SMStateData;

/// Input data for executing SEC API requests.
///
/// This struct holds the prepared SEC client and request objects that will be used
/// to execute HTTP requests to SEC API endpoints. It is designed to be used as part
/// of the SEC document extraction workflow, and supports builder-based updates and
/// integration with the state machine framework.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Default)]
pub struct ExecuteSecRequestInput {
    /// The prepared SEC client that will execute the HTTP request.
    pub sec_client: SecClient,
    /// The prepared SEC request targeting a specific CIK.
    pub sec_request: SecRequest,
}

impl ExecuteSecRequestInput {
    /// Creates a new instance of the input data for executing SEC requests.
    ///
    /// # Arguments
    ///
    /// * `sec_client` - The prepared [`SecClient`] configured with proper user agent.
    /// * `sec_request` - The prepared [`SecRequest`] targeting the desired CIK.
    ///
    /// # Returns
    ///
    /// Returns a new [`ExecuteSecRequestInput`] instance ready for state processing.
    pub const fn new(sec_client: SecClient, sec_request: SecRequest) -> Self {
        Self {
            sec_client,
            sec_request,
        }
    }

    /// Returns a reference to the SEC client.
    ///
    /// # Returns
    ///
    /// A reference to the [`SecClient`] that will be used for HTTP requests.
    #[must_use]
    pub const fn sec_client(&self) -> &SecClient {
        &self.sec_client
    }

    /// Returns a reference to the SEC request.
    ///
    /// # Returns
    ///
    /// A reference to the [`SecRequest`] that will be executed.
    #[must_use]
    pub const fn sec_request(&self) -> &SecRequest {
        &self.sec_request
    }
}

impl StateData for ExecuteSecRequestInput {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(sec_client) = updates.sec_client {
            self.sec_client = sec_client;
        }
        if let Some(sec_request) = updates.sec_request {
            self.sec_request = sec_request;
        }
        Ok(())
    }
}

impl SMStateData for ExecuteSecRequestInput {
    type UpdateType = ExecuteSecRequestInputUpdater;

    fn state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl fmt::Display for ExecuteSecRequestInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SEC Client ID: {}\nSEC Request URL: {}",
            self.sec_client.id(),
            self.sec_request.inner.url()
        )
    }
}

/// Updater for modifying [`ExecuteSecRequestInput`] in a controlled manner.
///
/// This struct allows for partial updates to input data fields while maintaining
/// type safety and avoiding unnecessary allocations for unchanged fields.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ExecuteSecRequestInputUpdater {
    /// Optional new SEC client to replace the current one.
    pub sec_client: Option<SecClient>,
    /// Optional new SEC request to replace the current one.
    pub sec_request: Option<SecRequest>,
}

impl ExecuteSecRequestInputUpdater {
    /// Creates a new builder for constructing [`ExecuteSecRequestInputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ExecuteSecRequestInputUpdaterBuilder {
        ExecuteSecRequestInputUpdaterBuilder::new()
    }
}

/// Builder for constructing [`ExecuteSecRequestInputUpdater`] instances.
///
/// This builder provides a fluent API for constructing updaters with only
/// the fields that need to be changed, following the builder pattern.
pub struct ExecuteSecRequestInputUpdaterBuilder {
    sec_client: Option<SecClient>,
    sec_request: Option<SecRequest>,
}

impl ExecuteSecRequestInputUpdaterBuilder {
    /// Creates a new builder with no fields set to be updated.
    ///
    /// # Returns
    ///
    /// A new [`ExecuteSecRequestInputUpdaterBuilder`] with all fields set to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sec_client: None,
            sec_request: None,
        }
    }

    /// Sets the SEC client to be updated.
    ///
    /// # Arguments
    ///
    /// * `sec_client` - The new [`SecClient`] to set in the input data.
    ///
    /// # Returns
    ///
    /// The builder instance with the SEC client field set for update.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn sec_client(mut self, sec_client: SecClient) -> Self {
        self.sec_client = Some(sec_client);
        self
    }

    /// Sets the SEC request to be updated.
    ///
    /// # Arguments
    ///
    /// * `sec_request` - The new [`SecRequest`] to set in the input data.
    ///
    /// # Returns
    ///
    /// The builder instance with the SEC request field set for update.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn sec_request(mut self, sec_request: SecRequest) -> Self {
        self.sec_request = Some(sec_request);
        self
    }

    /// Builds the updater with the configured fields.
    ///
    /// # Returns
    ///
    /// A new [`ExecuteSecRequestInputUpdater`] with the fields set by this builder.
    #[must_use]
    pub fn build(self) -> ExecuteSecRequestInputUpdater {
        ExecuteSecRequestInputUpdater {
            sec_client: self.sec_client,
            sec_request: self.sec_request,
        }
    }
}

impl Default for ExecuteSecRequestInputUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use super::*;
    use crate::shared::cik::Cik;

    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_new_input_data_with_provided_client_and_request() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let client = SecClient::new("Test Company contact@test.com")
            .expect("Hardcoded user agent should always be valid.");
        let request = SecRequest::new(&cik);

        let expected_result = ExecuteSecRequestInput {
            sec_client: client.clone(),
            sec_request: request.clone(),
        };

        let result = ExecuteSecRequestInput::new(client, request);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_client_reference_when_accessing_sec_client() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let client = SecClient::new("Test Company contact@test.com")
            .expect("Hardcoded user agent should always be valid.");
        let request = SecRequest::new(&cik);
        let input_data = ExecuteSecRequestInput::new(client.clone(), request);

        let expected_result = &client;

        let result = input_data.sec_client();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_request_reference_when_accessing_sec_request() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let client = SecClient::new("Test Company contact@test.com")
            .expect("Hardcoded user agent should always be valid.");
        let request = SecRequest::new(&cik);
        let input_data = ExecuteSecRequestInput::new(client, request.clone());

        let expected_result = &request;

        let result = input_data.sec_request();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_ok_when_updating_with_updater() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let client = SecClient::new("Test Company contact@test.com")
            .expect("Hardcoded user agent should always be valid.");
        let request = SecRequest::new(&cik);
        let mut input_data = ExecuteSecRequestInput::new(client, request);

        let updater = ExecuteSecRequestInputUpdater::builder().build();

        let expected_result = Ok(());

        let result = StateData::update_state(&mut input_data, updater);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_sec_client_when_updater_contains_client() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let original_client = SecClient::new("Original Company contact@original.com")
            .expect("Hardcoded user agent should always be valid.");
        let new_client = SecClient::new("New Company contact@new.com")
            .expect("Hardcoded user agent should always be valid.");
        let request = SecRequest::new(&cik);
        let mut input_data = ExecuteSecRequestInput::new(original_client, request);

        let updater = ExecuteSecRequestInputUpdater::builder()
            .sec_client(new_client.clone())
            .build();

        let _ = StateData::update_state(&mut input_data, updater);

        let expected_result = &new_client;

        let result = input_data.sec_client();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_sec_request_when_updater_contains_request() {
        let original_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let new_cik = Cik::new("0987654321").expect("Hardcoded CIK should always be valid.");
        let client = SecClient::new("Test Company contact@test.com")
            .expect("Hardcoded user agent should always be valid.");
        let original_request = SecRequest::new(&original_cik);
        let new_request = SecRequest::new(&new_cik);
        let mut input_data = ExecuteSecRequestInput::new(client, original_request);

        let updater = ExecuteSecRequestInputUpdater::builder()
            .sec_request(new_request.clone())
            .build();

        let _ = StateData::update_state(&mut input_data, updater);

        let expected_result = &new_request;

        let result = input_data.sec_request();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_sec_client_when_updater_contains_both_fields() {
        let original_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let new_cik = Cik::new("0987654321").expect("Hardcoded CIK should always be valid.");
        let original_client = SecClient::new("Original Company contact@original.com")
            .expect("Hardcoded user agent should always be valid.");
        let new_client = SecClient::new("New Company contact@new.com")
            .expect("Hardcoded user agent should always be valid.");
        let original_request = SecRequest::new(&original_cik);
        let new_request = SecRequest::new(&new_cik);
        let mut input_data = ExecuteSecRequestInput::new(original_client, original_request);

        let updater = ExecuteSecRequestInputUpdater::builder()
            .sec_client(new_client.clone())
            .sec_request(new_request.clone())
            .build();

        let _ = StateData::update_state(&mut input_data, updater);

        let expected_result = &new_client;

        let result = input_data.sec_client();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_sec_request_when_updater_contains_both_fields() {
        let original_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let new_cik = Cik::new("0987654321").expect("Hardcoded CIK should always be valid.");
        let original_client = SecClient::new("Original Company contact@original.com")
            .expect("Hardcoded user agent should always be valid.");
        let new_client = SecClient::new("New Company contact@new.com")
            .expect("Hardcoded user agent should always be valid.");
        let original_request = SecRequest::new(&original_cik);
        let new_request = SecRequest::new(&new_cik);
        let mut input_data = ExecuteSecRequestInput::new(original_client, original_request);

        let updater = ExecuteSecRequestInputUpdater::builder()
            .sec_client(new_client.clone())
            .sec_request(new_request.clone())
            .build();

        let _ = StateData::update_state(&mut input_data, updater);

        let expected_result = &new_request;

        let result = input_data.sec_request();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_update_fields_when_updater_is_empty() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let client = SecClient::new("Test Company contact@test.com")
            .expect("Hardcoded user agent should always be valid.");
        let request = SecRequest::new(&cik);
        let original_input_data = ExecuteSecRequestInput::new(client, request);
        let mut input_data = original_input_data.clone();

        let updater = ExecuteSecRequestInputUpdater::builder().build();

        let _ = StateData::update_state(&mut input_data, updater);

        let expected_result = original_input_data;

        let result = input_data;

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_data_trait() {
        implements_auto_traits::<ExecuteSecRequestInput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_data_trait() {
        implements_send::<ExecuteSecRequestInput>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_data_trait() {
        implements_sync::<ExecuteSecRequestInput>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_data_trait() {
        implements_send::<ExecuteSecRequestInput>();
        implements_sync::<ExecuteSecRequestInput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_data_trait() {
        implements_sized::<ExecuteSecRequestInput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_data_trait() {
        implements_hash::<ExecuteSecRequestInput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_data_trait() {
        implements_partial_eq::<ExecuteSecRequestInput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_data_trait() {
        implements_eq::<ExecuteSecRequestInput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_data_trait() {
        implements_partial_ord::<ExecuteSecRequestInput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_data_trait() {
        implements_ord::<ExecuteSecRequestInput>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_state_data_trait() {
        implements_default::<ExecuteSecRequestInput>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_data_trait() {
        implements_debug::<ExecuteSecRequestInput>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_data_trait() {
        implements_clone::<ExecuteSecRequestInput>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_data_trait() {
        implements_unpin::<ExecuteSecRequestInput>();
    }
}
