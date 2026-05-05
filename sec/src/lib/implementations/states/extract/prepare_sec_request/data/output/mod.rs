//! # `PrepareSecRequestOutput` Module
//!
//! This module defines the output data structure and updater patterns for the `PrepareSecRequest` state
//! within the SEC extraction state machine. It encapsulates the prepared SEC client and request objects
//! and provides builders and updaters for controlled mutation of output data.
//!
//! ## Types
//! - [`PrepareSecRequestOutput`]: Holds the prepared SEC client and HTTP request after successful preparation.
//! - [`PrepareSecRequestOutputUpdater`]: Updater type for modifying the output data in a controlled manner.
//! - [`PrepareSecRequestOutputUpdaterBuilder`]: Builder for constructing updater instances with optional fields.
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
//! - [`input`](super::input): Input data structure for request preparation parameters.
//! - [`crate::shared::http_client`]: Utilities for SEC client creation.
//! - [`crate::shared::request`]: Utilities for SEC request construction.
//! - [`state_maschine::prelude::StateData`]: Trait for state data integration.
//!
//! ## Examples
//! See the unit tests in this module for usage patterns and updater logic.

use std::{fmt, hash::Hash};

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::traits::state_machine::state::StateData;

/// Output data containing a prepared SEC client and request.
///
/// This struct holds a prepared [`SecClient`] and [`SecRequest`] value, produced by the [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request) state
/// after successful preparation. It is used as output in the SEC extraction state machine,
/// and supports builder-based updates and integration with the state machine framework.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct PrepareSecRequestOutput {
    /// The prepared SEC client for making HTTP requests.
    pub client: SecClient,
    /// The prepared SEC request targeting a specific CIK.
    pub request: SecRequest,
}

impl PrepareSecRequestOutput {
    /// Creates a new instance of the output data for the prepare SEC request state.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::prepare_sec_request::data::output::PrepareSecRequestOutput;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    /// use sec::shared::cik::Cik;
    ///
    /// let client = SecClient::default();
    /// let cik = Cik::new("1067983").expect("Hardcoded CIK string should be valid format");
    /// let request = SecRequest::builder()
    ///     .all_company_facts()
    ///     .cik(cik)
    ///     .build();
    /// let output_data = PrepareSecRequestOutput::new(client, request);
    /// ```
    ///
    /// # Errors
    /// Returns a [`StateError`] if the output data cannot be created from the provided data.
    pub const fn new(client: SecClient, request: SecRequest) -> Self {
        Self { client, request }
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

impl StateData for PrepareSecRequestOutput {
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
impl SMStateData for PrepareSecRequestOutput {
    type UpdateType = PrepareSecRequestOutputUpdater;

    /// Returns a reference to the current state data, which represents the output data of this state.
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

impl fmt::Display for PrepareSecRequestOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tURL: {}", self.request.inner.url())
    }
}

/// Updater for [`PrepareSecRequestOutput`].
///
/// This struct is used to specify updates to the output data in a controlled, partial manner.
/// Fields set to `None` will not be updated. Used in conjunction with the state machine's
/// update mechanism to ensure safe and explicit state transitions.
#[derive(Debug)]
pub struct PrepareSecRequestOutputUpdater {
    /// Optional new value for the SEC client.
    pub client: Option<SecClient>,
    /// Optional new value for the SEC request.
    pub request: Option<SecRequest>,
}

impl PrepareSecRequestOutputUpdater {
    /// Creates a new builder for constructing [`PrepareSecRequestOutputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> PrepareSecRequestOutputUpdaterBuilder {
        PrepareSecRequestOutputUpdaterBuilder::new()
    }
}

/// Builder for [`PrepareSecRequestOutputUpdater`].
///
/// This builder allows for ergonomic and explicit construction of updater instances,
/// supporting method chaining and optional fields. Use `.build()` to produce the updater.
pub struct PrepareSecRequestOutputUpdaterBuilder {
    client: Option<SecClient>,
    request: Option<SecRequest>,
}

impl PrepareSecRequestOutputUpdaterBuilder {
    /// Creates a new [`PrepareSecRequestOutputUpdaterBuilder`] with no fields set.
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

    /// Builds the [`PrepareSecRequestOutputUpdater`] instance from the builder.
    #[must_use]
    pub fn build(self) -> PrepareSecRequestOutputUpdater {
        PrepareSecRequestOutputUpdater {
            client: self.client,
            request: self.request,
        }
    }
}

impl Default for PrepareSecRequestOutputUpdaterBuilder {
    /// Returns a new [`PrepareSecRequestOutputUpdaterBuilder`] with no fields set.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::StateData as SMStateData;

    use super::{PrepareSecRequestOutput, PrepareSecRequestOutputUpdaterBuilder};
    use crate::shared::cik::Cik;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::request::implementations::sec_request::SecRequest;
    use crate::traits::state_machine::state::StateData;

    /// Creates a known-good baseline `PrepareSecRequestOutput` for use in tests.
    fn create_baseline_output() -> PrepareSecRequestOutput {
        let client = SecClient::default();
        let cik = Cik::new("0001067983").expect("Hardcoded CIK string should be valid format");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        PrepareSecRequestOutput::new(client, request)
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let client = SecClient::default();
        let cik = Cik::new("1234567890").expect("Hardcoded CIK string should be valid format");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let prepare_output_state_data = PrepareSecRequestOutput::new(client, request);

        let expected_result = &create_baseline_output();

        let result = prepare_output_state_data.state();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_specified_values_when_update_contains_specified_values() {
        let mut state_data = create_baseline_output();
        let new_client = SecClient::default();
        let cik = Cik::new("1234567890").expect("Hardcoded CIK string should be valid format");
        let new_request = SecRequest::builder().all_company_facts().cik(cik).build();
        let update = PrepareSecRequestOutputUpdaterBuilder::default()
            .client(new_client.clone())
            .request(new_request.clone())
            .build();

        let expected_result = &PrepareSecRequestOutput::new(new_client, new_request);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_only_request_when_update_contains_only_request() {
        let mut state_data = create_baseline_output();
        let original_client = state_data.client.clone();
        let cik = Cik::new("9876543210").expect("Hardcoded CIK string should be valid format");
        let new_request = SecRequest::builder().all_company_facts().cik(cik).build();
        let update = PrepareSecRequestOutputUpdaterBuilder::default()
            .request(new_request.clone())
            .build();

        let expected_result = &PrepareSecRequestOutput::new(original_client, new_request);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_only_client_when_update_contains_only_client() {
        let mut state_data = create_baseline_output();
        let original_request = state_data.request.clone();
        let new_client = SecClient::default();
        let update = PrepareSecRequestOutputUpdaterBuilder::default()
            .client(new_client.clone())
            .build();

        let expected_result = &PrepareSecRequestOutput::new(new_client, original_request);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = create_baseline_output();
        let empty_update = PrepareSecRequestOutputUpdaterBuilder::default().build();

        let expected_result = &create_baseline_output();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_client_when_accessor_method_is_called() {
        let client = SecClient::default();
        let cik = Cik::new("1234567890").expect("Hardcoded CIK string should be valid format");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let prepare_output_state_data = PrepareSecRequestOutput::new(client.clone(), request);

        let expected_result = &client;

        let result = prepare_output_state_data.client();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_request_when_accessor_method_is_called() {
        let client = SecClient::default();
        let cik = Cik::new("1234567890").expect("Hardcoded CIK string should be valid format");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let prepare_output_state_data = PrepareSecRequestOutput::new(client, request.clone());

        let expected_result = &request;

        let result = prepare_output_state_data.request();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_output_data_successfully_when_valid_client_and_request_provided() {
        let client = SecClient::default();
        let cik = Cik::new("1234567890").expect("Hardcoded CIK string should be valid format");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();

        let expected_result = PrepareSecRequestOutput {
            client: client.clone(),
            request: request.clone(),
        };

        let result = PrepareSecRequestOutput::new(client, request);

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_output_data_trait() {
        implements_auto_traits::<PrepareSecRequestOutput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_output_data_trait() {
        implements_send::<PrepareSecRequestOutput>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_output_data_trait() {
        implements_sync::<PrepareSecRequestOutput>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_output_data_trait() {
        implements_send::<PrepareSecRequestOutput>();
        implements_sync::<PrepareSecRequestOutput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_output_data_trait() {
        implements_sized::<PrepareSecRequestOutput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_output_data_trait() {
        implements_hash::<PrepareSecRequestOutput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_output_data_trait() {
        implements_partial_eq::<PrepareSecRequestOutput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_output_data_trait() {
        implements_eq::<PrepareSecRequestOutput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_output_data_trait() {
        implements_partial_ord::<PrepareSecRequestOutput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_output_data_trait() {
        implements_ord::<PrepareSecRequestOutput>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_output_data_trait() {
        implements_debug::<PrepareSecRequestOutput>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_output_data_trait() {
        implements_clone::<PrepareSecRequestOutput>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_output_data_trait() {
        implements_unpin::<PrepareSecRequestOutput>();
    }
}
