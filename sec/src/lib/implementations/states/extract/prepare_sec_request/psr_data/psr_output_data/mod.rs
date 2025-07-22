//! # `PrepareSecRequestOutputData` Module
//!
//! This module defines the output data structure and updater patterns for the `PrepareSecRequest` state
//! within the SEC extraction state machine. It encapsulates the prepared SEC client and request objects
//! and provides builders and updaters for controlled mutation of output data.
//!
//! ## Types
//! - [`PrepareSecRequestOutputData`]: Holds the prepared SEC client and HTTP request after successful preparation.
//! - [`PrepareSecRequestOutputDataUpdater`]: Updater type for modifying the output data in a controlled manner.
//! - [`PrepareSecRequestOutputDataUpdaterBuilder`]: Builder for constructing updater instances with optional fields.
//!
//! ## Integration
//! - Implements [`StateData`](state_maschine::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request) to produce and update request output data.
//!
//! ## Usage
//! This module is intended for use in the output phase of SEC request preparation. It supports builder-based updates and
//! integrates with the state machine's updater and state data traits for robust, testable workflows.
//!
//! ## See Also
//! - [`psr_input_data`](super::psr_input_data): Input data structure for request preparation parameters.
//! - [`crate::shared::sec_client`]: Utilities for SEC client creation.
//! - [`crate::shared::sec_request`]: Utilities for SEC request construction.
//! - [`state_maschine::prelude::StateData`]: Trait for state data integration.
//!
//! ## Examples
//! See the unit tests in this module for usage patterns and updater logic.

use std::{fmt, hash::Hash};

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::sec_client::SecClient;
use crate::shared::sec_request::SecRequest;
use crate::traits::state_machine::state::StateData;

/// Output data containing a prepared SEC client and request.
///
/// This struct holds a prepared [`SecClient`] and [`SecRequest`] value, produced by the [`PrepareSecRequest`] state
/// after successful preparation. It is used as output in the SEC extraction state machine,
/// and supports builder-based updates and integration with the state machine framework.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestOutputData {
    /// The prepared SEC client for making HTTP requests.
    pub client: SecClient,
    /// The prepared SEC request targeting a specific CIK.
    pub request: SecRequest,
}

impl PrepareSecRequestOutputData {
    /// Creates a new instance of the output data for the prepare SEC request state.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::prepare_sec_request::psr_data::psr_output_data::PrepareSecRequestOutputData;
    /// use sec::shared::sec_client::SecClient;
    /// use sec::shared::sec_request::SecRequest;
    /// use sec::shared::cik::Cik;
    /// use sec::shared::user_agent::UserAgent;
    ///
    /// let user_agent = "Test Company contact@test.com";
    /// let client = SecClient::new(&user_agent).expect("Valid client");
    /// let cik = Cik::new("1067983").expect("Valid CIK");
    /// let request = SecRequest::new(&cik);
    /// let output_data = PrepareSecRequestOutputData::new(client, request).expect("Valid output data");
    /// ```
    ///
    /// # Errors
    /// Returns a [`StateError`] if the output data cannot be created from the provided data.
    pub const fn new(client: SecClient, request: SecRequest) -> Result<Self, StateError> {
        Ok(Self { client, request })
    }

    /// Returns a reference to the prepared SEC client.
    #[must_use]
    pub const fn client(&self) -> &SecClient {
        &self.client
    }

    /// Returns a reference to the prepared SEC request.
    #[must_use]
    pub const fn request(&self) -> &SecRequest {
        &self.request
    }
}

impl StateData for PrepareSecRequestOutputData {
    /// Updates the state data using the provided updater.
    ///
    /// If `client` is `Some`, updates the SEC client; if `request` is `Some`, updates the SEC request;
    /// otherwise, leaves the respective fields unchanged.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(client) = updates.client {
            self.client = client;
        }
        if let Some(request) = updates.request {
            self.request = request;
        }
        Ok(())
    }
}
impl SMStateData for PrepareSecRequestOutputData {
    type UpdateType = PrepareSecRequestOutputDataUpdater;

    /// Returns a reference to the current state data, which represents the output data of this state.
    fn get_state(&self) -> &Self {
        self
    }

    /// Provided by `SecStateData` trait. Not used in this context.
    fn update_state(&mut self, _updates: Self::UpdateType) {
        // This method is not used in this context.
    }
}

impl fmt::Display for PrepareSecRequestOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tURL: {}", self.request.inner.url())
    }
}

/// Updater for [`PrepareSecRequestOutputData`].
///
/// This struct is used to specify updates to the output data in a controlled, partial manner.
/// Fields set to `None` will not be updated. Used in conjunction with the state machine's
/// update mechanism to ensure safe and explicit state transitions.
#[derive(Debug)]
pub struct PrepareSecRequestOutputDataUpdater {
    /// Optional new value for the SEC client.
    pub client: Option<SecClient>,
    /// Optional new value for the SEC request.
    pub request: Option<SecRequest>,
}

/// Builder for [`PrepareSecRequestOutputDataUpdater`].
///
/// This builder allows for ergonomic and explicit construction of updater instances,
/// supporting method chaining and optional fields. Use `.build()` to produce the updater.
pub struct PrepareSecRequestOutputDataUpdaterBuilder {
    client: Option<SecClient>,
    request: Option<SecRequest>,
}

impl PrepareSecRequestOutputDataUpdaterBuilder {
    /// Creates a new [`PrepareSecRequestOutputDataUpdaterBuilder`] with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            client: None,
            request: None,
        }
    }

    /// Sets the SEC client value to be updated.
    ///
    /// # Arguments
    ///
    /// * `client` - The new [`SecClient`] value.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn client(mut self, client: SecClient) -> Self {
        self.client = Some(client);
        self
    }

    /// Sets the SEC request value to be updated.
    ///
    /// # Arguments
    ///
    /// * `request` - The new [`SecRequest`] value.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn request(mut self, request: SecRequest) -> Self {
        self.request = Some(request);
        self
    }

    /// Builds the [`PrepareSecRequestOutputDataUpdater`] instance from the builder.
    #[must_use]
    pub fn build(self) -> PrepareSecRequestOutputDataUpdater {
        PrepareSecRequestOutputDataUpdater {
            client: self.client,
            request: self.request,
        }
    }
}

impl Default for PrepareSecRequestOutputDataUpdaterBuilder {
    /// Returns a new [`PrepareSecRequestOutputDataUpdaterBuilder`] with no fields set.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::{assert_eq, assert_ne};

    use super::{PrepareSecRequestOutputData, PrepareSecRequestOutputDataUpdaterBuilder};
    use crate::shared::cik::Cik;
    use crate::shared::sec_client::SecClient;
    use crate::shared::sec_request::SecRequest;
    use crate::shared::user_agent::UserAgent;
    use crate::traits::state_machine::state::StateData;
    use state_maschine::prelude::StateData as SMStateData;

    #[test]
    fn should_return_reference_to_default_prepare_output_state_data_when_initialized_with_default()
    {
        let default_prepare_output_state_data = PrepareSecRequestOutputData::default();

        let expected_result = &PrepareSecRequestOutputData::default();

        let result = default_prepare_output_state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let user_agent = UserAgent::new("Test Company contact@test.com").expect("Valid user agent");
        let client = SecClient::new(user_agent.inner()).expect("Valid client");
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let request = SecRequest::new(&cik);
        let prepare_output_state_data =
            PrepareSecRequestOutputData::new(client, request).expect("Valid output data");

        let default_prepare_output_state_data = &PrepareSecRequestOutputData::default();

        let result = prepare_output_state_data.get_state();

        assert_ne!(result, default_prepare_output_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_values_when_update_contains_specified_values() {
        let mut state_data = PrepareSecRequestOutputData::default();
        let user_agent =
            UserAgent::new("Updated Company contact@updated.com").expect("Valid user agent");
        let new_client = SecClient::new(user_agent.inner()).expect("Valid client");
        let new_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let new_request = SecRequest::new(&new_cik);
        let update = PrepareSecRequestOutputDataUpdaterBuilder::default()
            .client(new_client.clone())
            .request(new_request.clone())
            .build();

        let expected_result =
            &PrepareSecRequestOutputData::new(new_client, new_request).expect("Valid output data");

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_only_client_when_update_contains_only_client() {
        let mut state_data = PrepareSecRequestOutputData::default();
        let original_request = state_data.request.clone();
        let user_agent =
            UserAgent::new("New Client Company contact@newclient.com").expect("Valid user agent");
        let new_client = SecClient::new(user_agent.inner()).expect("Valid client");
        let update = PrepareSecRequestOutputDataUpdaterBuilder::default()
            .client(new_client.clone())
            .build();

        let expected_result = &PrepareSecRequestOutputData::new(new_client, original_request)
            .expect("Valid output data");

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_only_request_when_update_contains_only_request() {
        let mut state_data = PrepareSecRequestOutputData::default();
        let original_client = state_data.client.clone();
        let new_cik = Cik::new("9876543210").expect("Hardcoded CIK should always be valid.");
        let new_request = SecRequest::new(&new_cik);
        let update = PrepareSecRequestOutputDataUpdaterBuilder::default()
            .request(new_request.clone())
            .build();

        let expected_result = &PrepareSecRequestOutputData::new(original_client, new_request)
            .expect("Valid output data");

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_values_when_multiple_updates_in_builder() {
        let mut state_data = PrepareSecRequestOutputData::default();

        let first_user_agent =
            UserAgent::new("First Company contact@first.com").expect("Valid user agent");
        let first_client = SecClient::new(first_user_agent.inner()).expect("Valid client");
        let first_cik = Cik::new("1111111111").expect("Hardcoded CIK should always be valid.");
        let first_request = SecRequest::new(&first_cik);

        let final_user_agent =
            UserAgent::new("Final Company contact@final.com").expect("Valid user agent");
        let final_client = SecClient::new(final_user_agent.inner()).expect("Valid client");
        let final_cik = Cik::new("2222222222").expect("Hardcoded CIK should always be valid.");
        let final_request = SecRequest::new(&final_cik);

        let update = PrepareSecRequestOutputDataUpdaterBuilder::default()
            .client(first_client)
            .request(first_request)
            .client(final_client.clone())
            .request(final_request.clone())
            .build();

        let expected_result = &PrepareSecRequestOutputData::new(final_client, final_request)
            .expect("Valid output data");

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = PrepareSecRequestOutputData::default();
        let empty_update = PrepareSecRequestOutputDataUpdaterBuilder::default().build();

        let expected_result = &PrepareSecRequestOutputData::default();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_client_when_accessor_method_is_called() {
        let user_agent = UserAgent::new("Test Company contact@test.com").expect("Valid user agent");
        let client = SecClient::new(user_agent.inner()).expect("Valid client");
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let request = SecRequest::new(&cik);
        let prepare_output_state_data =
            PrepareSecRequestOutputData::new(client.clone(), request).expect("Valid output data");

        let expected_result = &client;

        let result = prepare_output_state_data.client();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_request_when_accessor_method_is_called() {
        let user_agent = UserAgent::new("Test Company contact@test.com").expect("Valid user agent");
        let client = SecClient::new(user_agent.inner()).expect("Valid client");
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let request = SecRequest::new(&cik);
        let prepare_output_state_data =
            PrepareSecRequestOutputData::new(client, request.clone()).expect("Valid output data");

        let expected_result = &request;

        let result = prepare_output_state_data.request();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_output_data_successfully_when_valid_client_and_request_provided() {
        let user_agent = UserAgent::new("Test Company contact@test.com").expect("Valid user agent");
        let client = SecClient::new(user_agent.inner()).expect("Valid client");
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let request = SecRequest::new(&cik);

        let expected_result = PrepareSecRequestOutputData {
            client: client.clone(),
            request: request.clone(),
        };

        let result = PrepareSecRequestOutputData::new(client, request)
            .expect("Valid output data creation should succeed");

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_output_data_trait() {
        implements_auto_traits::<PrepareSecRequestOutputData>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_output_data_trait() {
        implements_send::<PrepareSecRequestOutputData>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_output_data_trait() {
        implements_sync::<PrepareSecRequestOutputData>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_output_data_trait() {
        implements_send::<PrepareSecRequestOutputData>();
        implements_sync::<PrepareSecRequestOutputData>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_output_data_trait() {
        implements_sized::<PrepareSecRequestOutputData>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_output_data_trait() {
        implements_hash::<PrepareSecRequestOutputData>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_output_data_trait() {
        implements_partial_eq::<PrepareSecRequestOutputData>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_output_data_trait() {
        implements_eq::<PrepareSecRequestOutputData>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_output_data_trait() {
        implements_partial_ord::<PrepareSecRequestOutputData>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_output_data_trait() {
        implements_ord::<PrepareSecRequestOutputData>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_output_data_trait() {
        implements_default::<PrepareSecRequestOutputData>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_output_data_trait() {
        implements_debug::<PrepareSecRequestOutputData>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_output_data_trait() {
        implements_clone::<PrepareSecRequestOutputData>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_output_data_trait() {
        implements_unpin::<PrepareSecRequestOutputData>();
    }
}
