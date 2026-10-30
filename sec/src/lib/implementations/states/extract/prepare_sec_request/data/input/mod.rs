//! # Prepare SEC Request Input
//!
//! Provides the [`PrepareSecRequestInput`] fed into the
//! [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest)
//! state, along with its updater and builder.
//!
//! It carries the input of the request kind `K`, for example the validated
//! [`Cik`](crate::shared::cik::Cik) of a Company Facts request.
//!
//! ## See Also
//!
//! - [`output`](super::output): The prepared SEC request produced from this input.
//! - [`crate::shared::request::implementations::sec_request::RequestKind`]: The kind that sets the input type.

use std::fmt;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::request::implementations::sec_request::RequestKind;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`PrepareSecRequest`](super::super::PrepareSecRequest) state.
///
/// Holds the input of the request kind `K`.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestInput<K: RequestKind> {
    /// The input of the request, for example the validated `Cik` of a Company Facts request.
    pub request_input: K::Input,
}

impl<K: RequestKind> PrepareSecRequestInput<K> {
    /// Creates a new [`PrepareSecRequestInput`] from the input of the request kind `K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::prepare_sec_request::data::input::PrepareSecRequestInput;
    /// use sec::shared::cik::Cik;
    /// use sec::shared::request::implementations::sec_request::CompanyFacts;
    ///
    /// let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
    /// let input = PrepareSecRequestInput::<CompanyFacts>::new(cik.clone());
    ///
    /// let expected_result = &cik;
    ///
    /// let result = input.request_input();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(request_input: K::Input) -> Self {
        Self { request_input }
    }

    /// Returns a reference to the request input.
    #[must_use]
    pub const fn request_input(&self) -> &K::Input {
        &self.request_input
    }
}

/// Serializes the request input under the name its kind gives it, for example `cik`.
impl<K: RequestKind> Serialize for PrepareSecRequestInput<K> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("PrepareSecRequestInput", 1)?;
        state.serialize_field(K::INPUT_NAME, &self.request_input)?;
        state.end()
    }
}

impl<K: RequestKind> StateData for PrepareSecRequestInput<K> {
    /// Updates the state data using the provided updater.
    ///
    /// Only fields set to `Some` in the updater are modified; others are left unchanged.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(request_input) = updates.request_input {
            self.request_input = request_input;
        }
        Ok(())
    }
}

impl<K: RequestKind> SMStateData for PrepareSecRequestInput<K> {
    type UpdateType = PrepareSecRequestInputUpdater<K>;

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

impl<K: RequestKind> fmt::Display for PrepareSecRequestInput<K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\t{}: {}", K::INPUT_NAME, self.request_input)
    }
}

/// Updater for modifying [`PrepareSecRequestInput`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestInputUpdater<K: RequestKind> {
    /// Optional new value for the request input.
    pub request_input: Option<K::Input>,
}

impl<K: RequestKind> PrepareSecRequestInputUpdater<K> {
    /// Creates a new builder for constructing [`PrepareSecRequestInputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> PrepareSecRequestInputUpdaterBuilder<K> {
        PrepareSecRequestInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`PrepareSecRequestInputUpdater`].
pub struct PrepareSecRequestInputUpdaterBuilder<K: RequestKind> {
    request_input: Option<K::Input>,
}

impl<K: RequestKind> PrepareSecRequestInputUpdaterBuilder<K> {
    /// Creates a new [`PrepareSecRequestInputUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            request_input: None,
        }
    }

    /// Sets the request input field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn request_input(mut self, request_input: K::Input) -> Self {
        self.request_input = Some(request_input);
        self
    }

    /// Builds the [`PrepareSecRequestInputUpdater`] instance from the builder.
    #[must_use]
    pub fn build(self) -> PrepareSecRequestInputUpdater<K> {
        PrepareSecRequestInputUpdater {
            request_input: self.request_input,
        }
    }
}

impl<K: RequestKind> Default for PrepareSecRequestInputUpdaterBuilder<K> {
    /// Returns a new [`PrepareSecRequestInputUpdaterBuilder`] with all fields initialized to `None`.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::StateData as SMStateData;

    use super::{PrepareSecRequestInput, PrepareSecRequestInputUpdaterBuilder};
    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::implementations::sec_request::{CompanyFacts, NotesFile};
    use crate::traits::state_machine::state::StateData;

    fn create_test_input() -> PrepareSecRequestInput<CompanyFacts> {
        let cik =
            Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid");
        PrepareSecRequestInput::new(cik)
    }

    #[test]
    fn should_return_reference_to_default_prepare_state_data_when_initialized_with_default() {
        let default_prepare_state_data = create_test_input();

        let expected_result = &create_test_input();

        let result = default_prepare_state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let prepare_state_data = PrepareSecRequestInput::new(cik);

        let default_prepare_state_data = &create_test_input();

        let result = prepare_state_data.state();

        assert_ne!(result, default_prepare_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_values_when_update_contains_specified_values() {
        let mut state_data = create_test_input();
        let new_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let update = PrepareSecRequestInputUpdaterBuilder::<CompanyFacts>::default()
            .request_input(new_cik.clone())
            .build();
        let expected_result = &PrepareSecRequestInput::new(new_cik);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_only_cik_when_update_contains_only_cik() {
        let mut state_data = create_test_input();
        let new_cik = Cik::new("9876543210").expect("Hardcoded CIK should always be valid");
        let update = PrepareSecRequestInputUpdaterBuilder::<CompanyFacts>::default()
            .request_input(new_cik.clone())
            .build();
        let expected_result = &PrepareSecRequestInput::new(new_cik);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_values_when_multiple_updates_in_builder() {
        let mut state_data = create_test_input();
        let first_cik = Cik::new("1111111111").expect("Hardcoded CIK should always be valid");
        let final_cik = Cik::new("2222222222").expect("Hardcoded CIK should always be valid");

        let update = PrepareSecRequestInputUpdaterBuilder::<CompanyFacts>::default()
            .request_input(first_cik)
            .request_input(final_cik.clone())
            .build();
        let expected_result = &PrepareSecRequestInput::new(final_cik);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = create_test_input();
        let empty_update = PrepareSecRequestInputUpdaterBuilder::<CompanyFacts>::default().build();

        let expected_result = &create_test_input();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_request_input_when_accessor_method_is_called() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let prepare_state_data = PrepareSecRequestInput::<CompanyFacts>::new(cik.clone());

        let expected_result = &cik;

        let result = prepare_state_data.request_input();

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_prepare_sec_request_input() {
        implements_auto_traits::<PrepareSecRequestInput<CompanyFacts>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_prepare_sec_request_input() {
        implements_send::<PrepareSecRequestInput<CompanyFacts>>();
    }

    #[test]
    const fn should_implement_sync_for_prepare_sec_request_input() {
        implements_sync::<PrepareSecRequestInput<CompanyFacts>>();
    }

    #[test]
    const fn should_be_thread_safe_for_prepare_sec_request_input() {
        implements_send::<PrepareSecRequestInput<CompanyFacts>>();
        implements_sync::<PrepareSecRequestInput<CompanyFacts>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_prepare_sec_request_input() {
        implements_sized::<PrepareSecRequestInput<CompanyFacts>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_prepare_sec_request_input() {
        implements_hash::<PrepareSecRequestInput<CompanyFacts>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_prepare_sec_request_input() {
        implements_partial_eq::<PrepareSecRequestInput<CompanyFacts>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_prepare_sec_request_input() {
        implements_eq::<PrepareSecRequestInput<CompanyFacts>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_prepare_sec_request_input() {
        implements_partial_ord::<PrepareSecRequestInput<CompanyFacts>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_prepare_sec_request_input() {
        implements_ord::<PrepareSecRequestInput<CompanyFacts>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_prepare_sec_request_input() {
        implements_debug::<PrepareSecRequestInput<CompanyFacts>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_prepare_sec_request_input() {
        implements_clone::<PrepareSecRequestInput<CompanyFacts>>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_prepare_sec_request_input() {
        implements_unpin::<PrepareSecRequestInput<CompanyFacts>>();
    }

    #[test]
    fn should_write_request_input_under_cik_when_serializing_company_facts_input() {
        let input = create_test_input();

        let expected_result = serde_json::json!({ "cik": { "value": "0001067983" } });

        let result = serde_json::to_value(input)
            .expect("Given an input of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_request_input_under_notes_period_when_serializing_notes_file_input() {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a published period, NotesPeriod creation should always succeed");
        let input = PrepareSecRequestInput::<NotesFile>::new(period);

        let expected_result = serde_json::json!({
            "notes_period": serde_json::to_value(period)
                .expect("Given a domain type, serialization should always succeed"),
        });

        let result = serde_json::to_value(input)
            .expect("Given an input of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_request_input_by_kind_when_displaying_company_facts_input() {
        let input = create_test_input();

        let expected_result = "\tcik: 0001067983";

        let result = input.to_string();

        assert_eq!(result, expected_result);
    }
}
