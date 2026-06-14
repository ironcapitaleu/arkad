//! # Execute SEC Request Output
//!
//! Provides the [`ExecuteSecRequestOutput`] produced by the
//! [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest)
//! state, along with its updater and builder.
//!
//! It wraps the [`SecResponse`] returned by the SEC API, which the transform phase consumes.
//! The dispatched client and request live in [`input`](super::input).
//!
//! ## See Also
//!
//! - [`input`](super::input): The client and request this response was produced from.
//! - [`crate::shared::response`]: The SEC response type carried here.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::traits::state_machine::state::StateData;

/// Output data of the [`ExecuteSecRequest`](super::super::ExecuteSecRequest) state.
///
/// Wraps the [`SecResponse`] returned by the SEC API, ready for the transform phase to parse.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct ExecuteSecRequestOutput {
    /// The SEC response received from the API endpoint.
    pub response: SecResponse,
}

impl ExecuteSecRequestOutput {
    /// Creates output data from a received SEC response.
    #[must_use]
    pub const fn new(response: SecResponse) -> Self {
        Self { response }
    }

    /// Returns a reference to the SEC response.
    #[must_use]
    pub const fn response(&self) -> &SecResponse {
        &self.response
    }
}
impl StateData for ExecuteSecRequestOutput {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(response) = updates.response {
            self.response = response;
        }
        Ok(())
    }
}
impl SMStateData for ExecuteSecRequestOutput {
    type UpdateType = ExecuteSecRequestOutputUpdater;

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

impl fmt::Display for ExecuteSecRequestOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\t{}", self.response)
    }
}

/// Partial update for an [`ExecuteSecRequestOutput`].
///
/// When `response` is `None` the output is left unchanged.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ExecuteSecRequestOutputUpdater {
    /// Optional new value for the SEC response.
    pub response: Option<SecResponse>,
}

impl ExecuteSecRequestOutputUpdater {
    /// Creates a new builder for constructing [`ExecuteSecRequestOutputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ExecuteSecRequestOutputUpdaterBuilder {
        ExecuteSecRequestOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for an [`ExecuteSecRequestOutputUpdater`].
pub struct ExecuteSecRequestOutputUpdaterBuilder {
    response: Option<SecResponse>,
}

impl ExecuteSecRequestOutputUpdaterBuilder {
    /// Creates a new builder with no fields set to be updated.
    ///
    /// # Returns
    ///
    /// A new [`ExecuteSecRequestOutputUpdaterBuilder`] with all fields set to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self { response: None }
    }

    /// Sets the SEC response to be updated.
    ///
    /// # Arguments
    ///
    /// * `response` - The new [`SecResponse`] to set in the output data.
    ///
    /// # Returns
    ///
    /// The builder instance with the response field set for update.
    #[must_use]
    pub fn response(mut self, response: SecResponse) -> Self {
        self.response = Some(response);
        self
    }

    /// Builds the updater with the configured fields.
    ///
    /// # Returns
    ///
    /// A new [`ExecuteSecRequestOutputUpdater`] with the fields set by this builder.
    #[must_use]
    pub fn build(self) -> ExecuteSecRequestOutputUpdater {
        ExecuteSecRequestOutputUpdater {
            response: self.response,
        }
    }
}

impl Default for ExecuteSecRequestOutputUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::response::implementations::sec_response::SecResponse;
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;

    /// Creates a known-good baseline `SecResponse` for use in tests.
    fn create_baseline_response() -> SecResponse {
        let url: Url = "https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json"
            .parse()
            .expect("Hardcoded URL should always parse successfully");

        let mut raw_headers = HashMap::new();
        raw_headers.insert("content-type".to_string(), "application/json".to_string());
        let headers = Headers::new(raw_headers);

        SecResponse::from_parts(
            url,
            headers,
            ContentType::Json,
            StatusCode::Ok,
            serde_json::json!({}),
        )
    }

    #[test]
    fn should_create_new_output_data_with_provided_response() {
        let response = create_baseline_response();
        let expected_response = response.clone();

        let result = ExecuteSecRequestOutput::new(response);

        assert_eq!(result.response(), &expected_response);
    }

    #[test]
    fn should_return_response_reference_when_accessing_response() {
        let response = create_baseline_response();
        let output_data = ExecuteSecRequestOutput::new(response.clone());

        let expected_result = &response;
        let result = output_data.response();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_response_when_updater_contains_response() {
        let original_response = create_baseline_response();
        let new_response = create_baseline_response();
        let mut output_data = ExecuteSecRequestOutput::new(original_response);

        let updater = ExecuteSecRequestOutputUpdater::builder()
            .response(new_response.clone())
            .build();

        let expected_result = Ok(());
        let result = StateData::update_state(&mut output_data, updater);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_response_information_when_formatted() {
        let response = create_baseline_response();
        let output_data = ExecuteSecRequestOutput::new(response);

        let expected_result = false;

        let result = format!("{output_data}").is_empty();

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_data_trait() {
        implements_auto_traits::<ExecuteSecRequestOutput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_data_trait() {
        implements_send::<ExecuteSecRequestOutput>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_data_trait() {
        implements_sync::<ExecuteSecRequestOutput>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_data_trait() {
        implements_send::<ExecuteSecRequestOutput>();
        implements_sync::<ExecuteSecRequestOutput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_data_trait() {
        implements_sized::<ExecuteSecRequestOutput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_data_trait() {
        implements_hash::<ExecuteSecRequestOutput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_data_trait() {
        implements_partial_eq::<ExecuteSecRequestOutput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_data_trait() {
        implements_eq::<ExecuteSecRequestOutput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_data_trait() {
        implements_partial_ord::<ExecuteSecRequestOutput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_data_trait() {
        implements_ord::<ExecuteSecRequestOutput>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_data_trait() {
        implements_debug::<ExecuteSecRequestOutput>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_data_trait() {
        implements_clone::<ExecuteSecRequestOutput>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_data_trait() {
        implements_unpin::<ExecuteSecRequestOutput>();
    }
}
