//! # Execute SEC Request Input
//!
//! Provides the [`ExecuteSecRequestInput`] fed into the
//! [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest)
//! state, along with its updater and builder.
//!
//! It carries the prepared SEC client and [`SecRequest`] needed to execute the request. The client
//! is generic over [`ExecutableSecClient`], so tests can replace the real [`SecClient`] with a fake.
//!
//! ## See Also
//!
//! - [`output`](super::output): The SEC response produced from this input.
//! - [`crate::shared::request`]: The SEC request type carried here.

use std::fmt;
use std::hash::Hash;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::request::SecRequest as SecRequestTrait;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::traits::state_machine::state::StateData;

/// An SEC client that the [`ExecuteSecRequest`](super::super::ExecuteSecRequest) state can hold
/// and run.
///
/// The state needs a client that executes a [`SecRequest`] into a [`SecResponse`], and that meets
/// the trait bounds every state data type carries. Every type that meets these bounds implements
/// this trait. The real [`SecClient`] is one of them.
pub trait ExecutableSecClient:
    SecClientTrait<Request = SecRequest, Response = SecResponse, Error = FailedSecRequest>
    + Clone
    + Unpin
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Hash
    + Serialize
{
}

impl<T> ExecutableSecClient for T where
    T: SecClientTrait<Request = SecRequest, Response = SecResponse, Error = FailedSecRequest>
        + Clone
        + Unpin
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + Serialize
{
}

/// Input data for the [`ExecuteSecRequest`](super::super::ExecuteSecRequest) state.
///
/// Bundles the prepared SEC client and [`SecRequest`] needed to execute the request. The client
/// type `C` defaults to the real [`SecClient`].
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct ExecuteSecRequestInput<C = SecClient> {
    /// The prepared SEC client that will execute the HTTP request.
    pub sec_client: C,
    /// The prepared SEC request targeting a specific CIK.
    pub sec_request: SecRequest,
}

impl<C> ExecuteSecRequestInput<C> {
    /// Creates a new [`ExecuteSecRequestInput`] from an SEC client and an SEC request.
    pub const fn new(sec_client: C, sec_request: SecRequest) -> Self {
        Self {
            sec_client,
            sec_request,
        }
    }

    /// Returns a reference to the SEC client.
    #[must_use]
    pub const fn sec_client(&self) -> &C {
        &self.sec_client
    }

    /// Returns a reference to the SEC request.
    #[must_use]
    pub const fn sec_request(&self) -> &SecRequest {
        &self.sec_request
    }
}

impl<C: ExecutableSecClient> StateData for ExecuteSecRequestInput<C> {
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

impl<C: ExecutableSecClient> SMStateData for ExecuteSecRequestInput<C> {
    type UpdateType = ExecuteSecRequestInputUpdater<C>;

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

impl<C> fmt::Display for ExecuteSecRequestInput<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SEC Request URL: {}", self.sec_request.url())
    }
}

/// Updater for modifying [`ExecuteSecRequestInput`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ExecuteSecRequestInputUpdater<C = SecClient> {
    /// Optional new value for the SEC client.
    pub sec_client: Option<C>,
    /// Optional new value for the SEC request.
    pub sec_request: Option<SecRequest>,
}

impl<C> ExecuteSecRequestInputUpdater<C> {
    /// Creates a new builder for constructing [`ExecuteSecRequestInputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ExecuteSecRequestInputUpdaterBuilder<C> {
        ExecuteSecRequestInputUpdaterBuilder::new()
    }
}

/// Fluent builder for an [`ExecuteSecRequestInputUpdater`].
pub struct ExecuteSecRequestInputUpdaterBuilder<C = SecClient> {
    sec_client: Option<C>,
    sec_request: Option<SecRequest>,
}

impl<C> ExecuteSecRequestInputUpdaterBuilder<C> {
    /// Creates a new [`ExecuteSecRequestInputUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sec_client: None,
            sec_request: None,
        }
    }

    /// Sets the SEC client field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn sec_client(mut self, sec_client: C) -> Self {
        self.sec_client = Some(sec_client);
        self
    }

    /// Sets the SEC request field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn sec_request(mut self, sec_request: SecRequest) -> Self {
        self.sec_request = Some(sec_request);
        self
    }

    /// Builds the [`ExecuteSecRequestInputUpdater`].
    #[must_use]
    pub fn build(self) -> ExecuteSecRequestInputUpdater<C> {
        ExecuteSecRequestInputUpdater {
            sec_client: self.sec_client,
            sec_request: self.sec_request,
        }
    }
}

impl<C> Default for ExecuteSecRequestInputUpdaterBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::request::implementations::sec_request::SecRequest;

    #[test]
    fn should_create_new_input_data_with_provided_client_and_request() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let client = SecClient::default();
        let request = SecRequest::builder().all_company_facts().cik(cik).build();

        let expected_result = ExecuteSecRequestInput {
            sec_client: client.clone(),
            sec_request: request.clone(),
        };

        let result = ExecuteSecRequestInput::new(client, request);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_client_reference_when_accessing_sec_client() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let client = SecClient::default();
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let input_data = ExecuteSecRequestInput::new(client.clone(), request);

        let expected_result = &client;

        let result = input_data.sec_client();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_request_reference_when_accessing_sec_request() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let client = SecClient::default();
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let input_data = ExecuteSecRequestInput::new(client, request.clone());

        let expected_result = &request;

        let result = input_data.sec_request();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_ok_when_updating_with_updater() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let client = SecClient::default();
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let mut input_data = ExecuteSecRequestInput::new(client, request);

        let updater = ExecuteSecRequestInputUpdater::builder().build();

        let expected_result = Ok(());

        let result = StateData::update_state(&mut input_data, updater);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_sec_client_when_updater_contains_client() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let original_client = SecClient::default();
        let new_client = SecClient::default();
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
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
        let original_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let new_cik = Cik::new("0987654321").expect("Hardcoded CIK should always be valid");
        let client = SecClient::default();
        let original_request = SecRequest::builder()
            .all_company_facts()
            .cik(original_cik)
            .build();
        let new_request = SecRequest::builder()
            .all_company_facts()
            .cik(new_cik)
            .build();
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
        let original_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let new_cik = Cik::new("0987654321").expect("Hardcoded CIK should always be valid");
        let original_client = SecClient::default();
        let new_client = SecClient::default();
        let original_request = SecRequest::builder()
            .all_company_facts()
            .cik(original_cik)
            .build();
        let new_request = SecRequest::builder()
            .all_company_facts()
            .cik(new_cik)
            .build();
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
        let original_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let new_cik = Cik::new("0987654321").expect("Hardcoded CIK should always be valid");
        let original_client = SecClient::default();
        let new_client = SecClient::default();
        let original_request = SecRequest::builder()
            .all_company_facts()
            .cik(original_cik)
            .build();
        let new_request = SecRequest::builder()
            .all_company_facts()
            .cik(new_cik)
            .build();
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
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let client = SecClient::default();
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
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
    const fn should_implement_auto_traits_for_execute_sec_request_input() {
        implements_auto_traits::<ExecuteSecRequestInput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_execute_sec_request_input() {
        implements_send::<ExecuteSecRequestInput>();
    }

    #[test]
    const fn should_implement_sync_for_execute_sec_request_input() {
        implements_sync::<ExecuteSecRequestInput>();
    }

    #[test]
    const fn should_be_thread_safe_for_execute_sec_request_input() {
        implements_send::<ExecuteSecRequestInput>();
        implements_sync::<ExecuteSecRequestInput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_execute_sec_request_input() {
        implements_sized::<ExecuteSecRequestInput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_execute_sec_request_input() {
        implements_hash::<ExecuteSecRequestInput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_execute_sec_request_input() {
        implements_partial_eq::<ExecuteSecRequestInput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_execute_sec_request_input() {
        implements_eq::<ExecuteSecRequestInput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_execute_sec_request_input() {
        implements_partial_ord::<ExecuteSecRequestInput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_execute_sec_request_input() {
        implements_ord::<ExecuteSecRequestInput>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_execute_sec_request_input() {
        implements_debug::<ExecuteSecRequestInput>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_execute_sec_request_input() {
        implements_clone::<ExecuteSecRequestInput>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_execute_sec_request_input() {
        implements_unpin::<ExecuteSecRequestInput>();
    }
}
