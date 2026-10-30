//! # Prepare SEC Request Output
//!
//! Provides the [`PrepareSecRequestOutput`] produced by the
//! [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest)
//! state, along with its updater and builder.
//!
//! It holds the [`SecRequest`] of the request kind `K`.
//!
//! ## See Also
//!
//! - [`input`](super::input): The request input this output is prepared from.
//! - [`crate::shared::request`]: The SEC request type carried here.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::request::implementations::sec_request::{RequestKind, SecRequest};
use crate::traits::state_machine::state::StateData;

/// Output data of the [`PrepareSecRequest`](super::super::PrepareSecRequest) state.
///
/// Holds the prepared [`SecRequest`] of the request kind `K`.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
#[serde(bound(serialize = ""))]
pub struct PrepareSecRequestOutput<K: RequestKind> {
    /// The prepared SEC request.
    pub request: SecRequest<K>,
}

impl<K: RequestKind> PrepareSecRequestOutput<K> {
    /// Creates a new [`PrepareSecRequestOutput`] from a prepared request.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::prepare_sec_request::data::output::PrepareSecRequestOutput;
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    /// use sec::shared::cik::Cik;
    ///
    /// let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
    /// let request = SecRequest::builder().all_company_facts().cik(cik).build();
    /// let output = PrepareSecRequestOutput::new(request.clone());
    ///
    /// let expected_result = &request;
    ///
    /// let result = output.request();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(request: SecRequest<K>) -> Self {
        Self { request }
    }

    /// Returns a reference to the prepared SEC request.
    #[must_use]
    pub const fn request(&self) -> &SecRequest<K> {
        &self.request
    }
}

impl<K: RequestKind> StateData for PrepareSecRequestOutput<K> {
    /// Updates the state data using the provided updater.
    ///
    /// If `request` is `Some`, updates the SEC request; otherwise, leaves it unchanged.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(request) = updates.request {
            self.request = request;
        }
        Ok(())
    }
}

impl<K: RequestKind> SMStateData for PrepareSecRequestOutput<K> {
    type UpdateType = PrepareSecRequestOutputUpdater<K>;

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

impl<K: RequestKind> fmt::Display for PrepareSecRequestOutput<K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tURL: {}", self.request.inner.url())
    }
}

/// Updater for modifying [`PrepareSecRequestOutput`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug)]
pub struct PrepareSecRequestOutputUpdater<K: RequestKind> {
    /// Optional new value for the SEC request.
    pub request: Option<SecRequest<K>>,
}

impl<K: RequestKind> PrepareSecRequestOutputUpdater<K> {
    /// Creates a new builder for constructing [`PrepareSecRequestOutputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> PrepareSecRequestOutputUpdaterBuilder<K> {
        PrepareSecRequestOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`PrepareSecRequestOutputUpdater`].
pub struct PrepareSecRequestOutputUpdaterBuilder<K: RequestKind> {
    request: Option<SecRequest<K>>,
}

impl<K: RequestKind> PrepareSecRequestOutputUpdaterBuilder<K> {
    /// Creates a new [`PrepareSecRequestOutputUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self { request: None }
    }

    /// Sets the SEC request field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn request(mut self, request: SecRequest<K>) -> Self {
        self.request = Some(request);
        self
    }

    /// Builds the [`PrepareSecRequestOutputUpdater`] instance from the builder.
    #[must_use]
    pub fn build(self) -> PrepareSecRequestOutputUpdater<K> {
        PrepareSecRequestOutputUpdater {
            request: self.request,
        }
    }
}

impl<K: RequestKind> Default for PrepareSecRequestOutputUpdaterBuilder<K> {
    /// Returns a new [`PrepareSecRequestOutputUpdaterBuilder`] with all fields initialized to `None`.
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
    use crate::shared::request::implementations::sec_request::{CompanyFacts, SecRequest};
    use crate::traits::state_machine::state::StateData;

    /// Creates a known-good baseline `PrepareSecRequestOutput` for use in tests.
    fn create_baseline_output() -> PrepareSecRequestOutput<CompanyFacts> {
        let cik = Cik::new("0001067983").expect("Hardcoded CIK string should be valid format");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        PrepareSecRequestOutput::new(request)
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK string should be valid format");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let prepare_output_state_data = PrepareSecRequestOutput::new(request);

        let expected_result = &create_baseline_output();

        let result = prepare_output_state_data.state();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_specified_values_when_update_contains_specified_values() {
        let mut state_data = create_baseline_output();
        let cik = Cik::new("1234567890").expect("Hardcoded CIK string should be valid format");
        let new_request = SecRequest::builder().all_company_facts().cik(cik).build();
        let update = PrepareSecRequestOutputUpdaterBuilder::default()
            .request(new_request.clone())
            .build();

        let expected_result = &PrepareSecRequestOutput::new(new_request);

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
    fn should_return_request_when_accessor_method_is_called() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK string should be valid format");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let prepare_output_state_data = PrepareSecRequestOutput::new(request.clone());

        let expected_result = &request;

        let result = prepare_output_state_data.request();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_output_data_successfully_when_valid_request_provided() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK string should be valid format");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();

        let expected_result = PrepareSecRequestOutput {
            request: request.clone(),
        };

        let result = PrepareSecRequestOutput::new(request);

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_prepare_sec_request_output() {
        implements_auto_traits::<PrepareSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_prepare_sec_request_output() {
        implements_send::<PrepareSecRequestOutput<CompanyFacts>>();
    }

    #[test]
    const fn should_implement_sync_for_prepare_sec_request_output() {
        implements_sync::<PrepareSecRequestOutput<CompanyFacts>>();
    }

    #[test]
    const fn should_be_thread_safe_for_prepare_sec_request_output() {
        implements_send::<PrepareSecRequestOutput<CompanyFacts>>();
        implements_sync::<PrepareSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_prepare_sec_request_output() {
        implements_sized::<PrepareSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_prepare_sec_request_output() {
        implements_hash::<PrepareSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_prepare_sec_request_output() {
        implements_partial_eq::<PrepareSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_prepare_sec_request_output() {
        implements_eq::<PrepareSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_prepare_sec_request_output() {
        implements_partial_ord::<PrepareSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_prepare_sec_request_output() {
        implements_ord::<PrepareSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_prepare_sec_request_output() {
        implements_debug::<PrepareSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_prepare_sec_request_output() {
        implements_clone::<PrepareSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_prepare_sec_request_output() {
        implements_unpin::<PrepareSecRequestOutput<CompanyFacts>>();
    }
}
