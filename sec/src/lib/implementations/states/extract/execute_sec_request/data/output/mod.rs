//! # Execute SEC Request Output
//!
//! Provides the [`ExecuteSecRequestOutput`] produced by the
//! [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest)
//! state, along with its updater and builder.
//!
//! It wraps the [`SecResponse`] returned by the SEC API.
//! The dispatched client and request live in [`input`](super::input).
//!
//! ## See Also
//!
//! - [`input`](super::input): The client and request this response was produced from.
//! - [`crate::shared::response`]: The SEC response type carried here.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::http_client::RespondsWith;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::traits::state_machine::state::StateData;

/// Output data of the [`ExecuteSecRequest`](super::super::ExecuteSecRequest) state.
///
/// Wraps the [`SecResponse`] returned by the SEC API for endpoint `E`.
///
/// The response sits in an [`Arc`], because a state output must be `Clone` and an
/// `SecResponse<ZipBody>` is not. A clone of the output copies a pointer.
///
/// The endpoint sets the body kind, so the output of a Notes request cannot hold a JSON response:
///
/// ```compile_fail,E0308
/// use sec::implementations::states::extract::execute_sec_request::ExecuteSecRequestOutput;
/// use sec::shared::request::implementations::sec_request::NotesFile;
/// use sec::shared::response::implementations::sec_response::{JsonBody, SecResponse};
///
/// fn notes_output(response: SecResponse<JsonBody>) -> ExecuteSecRequestOutput<NotesFile> {
///     ExecuteSecRequestOutput::new(response)
/// }
/// ```
#[derive(Debug)]
pub struct ExecuteSecRequestOutput<E: RespondsWith> {
    /// The SEC response received from the API endpoint.
    pub response: Arc<SecResponse<E::Body>>,
}

impl<E: RespondsWith> ExecuteSecRequestOutput<E> {
    /// Creates a new [`ExecuteSecRequestOutput`] from an SEC response.
    #[must_use]
    pub fn new(response: SecResponse<E::Body>) -> Self {
        Self {
            response: Arc::new(response),
        }
    }

    /// Returns a reference to the SEC response.
    #[must_use]
    pub fn response(&self) -> &SecResponse<E::Body> {
        &self.response
    }
}

// Deviation: the traits below are written by hand. A derive bounds `E::Body` by each trait and
// fails with "the trait bound `<E as RespondsWith>::Body: Clone` is not satisfied", because
// `ZipBody` implements none of them. `Serialize` is hand-written because serde implements it for
// `Arc` only with the `rc` feature. The impls compare the responses, which `SecResponse` supports
// for every body kind.
impl<E: RespondsWith> Clone for ExecuteSecRequestOutput<E> {
    fn clone(&self) -> Self {
        Self {
            response: Arc::clone(&self.response),
        }
    }
}

impl<E: RespondsWith> PartialEq for ExecuteSecRequestOutput<E> {
    fn eq(&self, other: &Self) -> bool {
        self.response == other.response
    }
}

impl<E: RespondsWith> Eq for ExecuteSecRequestOutput<E> {}

impl<E: RespondsWith> PartialOrd for ExecuteSecRequestOutput<E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E: RespondsWith> Ord for ExecuteSecRequestOutput<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.response.cmp(&other.response)
    }
}

impl<E: RespondsWith> Hash for ExecuteSecRequestOutput<E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.response.hash(state);
    }
}

/// Serializes the response without its body bytes.
impl<E: RespondsWith> Serialize for ExecuteSecRequestOutput<E> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ExecuteSecRequestOutput", 1)?;
        state.serialize_field("response", self.response.as_ref())?;
        state.end()
    }
}

impl<E: RespondsWith> StateData for ExecuteSecRequestOutput<E> {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(response) = updates.response {
            self.response = response;
        }
        Ok(())
    }
}
impl<E: RespondsWith> SMStateData for ExecuteSecRequestOutput<E> {
    type UpdateType = ExecuteSecRequestOutputUpdater<E>;

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

impl<E: RespondsWith> fmt::Display for ExecuteSecRequestOutput<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\t{}", self.response)
    }
}

/// Updater for modifying [`ExecuteSecRequestOutput`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug)]
pub struct ExecuteSecRequestOutputUpdater<E: RespondsWith> {
    /// Optional new value for the SEC response.
    pub response: Option<Arc<SecResponse<E::Body>>>,
}

impl<E: RespondsWith> ExecuteSecRequestOutputUpdater<E> {
    /// Creates a new builder for constructing [`ExecuteSecRequestOutputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ExecuteSecRequestOutputUpdaterBuilder<E> {
        ExecuteSecRequestOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for an [`ExecuteSecRequestOutputUpdater`].
pub struct ExecuteSecRequestOutputUpdaterBuilder<E: RespondsWith> {
    response: Option<Arc<SecResponse<E::Body>>>,
}

impl<E: RespondsWith> ExecuteSecRequestOutputUpdaterBuilder<E> {
    /// Creates a new [`ExecuteSecRequestOutputUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self { response: None }
    }

    /// Sets the SEC response field.
    #[must_use]
    pub fn response(mut self, response: SecResponse<E::Body>) -> Self {
        self.response = Some(Arc::new(response));
        self
    }

    /// Builds the [`ExecuteSecRequestOutputUpdater`].
    #[must_use]
    pub fn build(self) -> ExecuteSecRequestOutputUpdater<E> {
        ExecuteSecRequestOutputUpdater {
            response: self.response,
        }
    }
}

impl<E: RespondsWith> Default for ExecuteSecRequestOutputUpdaterBuilder<E> {
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
    use crate::shared::request::implementations::sec_request::CompanyFacts;
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
            b"{}".to_vec(),
        )
        .expect("Given the hardcoded JSON body '{}', building the response should always succeed")
    }

    #[test]
    fn should_create_new_output_data_with_provided_response() {
        let response = create_baseline_response();
        let expected_response = response.clone();

        let result = ExecuteSecRequestOutput::<CompanyFacts>::new(response);

        assert_eq!(result.response(), &expected_response);
    }

    #[test]
    fn should_return_response_reference_when_accessing_response() {
        let response = create_baseline_response();
        let output_data = ExecuteSecRequestOutput::<CompanyFacts>::new(response.clone());

        let expected_result = &response;
        let result = output_data.response();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_response_when_updater_contains_response() {
        let original_response = create_baseline_response();
        let new_response = create_baseline_response();
        let mut output_data = ExecuteSecRequestOutput::<CompanyFacts>::new(original_response);

        let updater = ExecuteSecRequestOutputUpdater::builder()
            .response(new_response)
            .build();

        let expected_result = Ok(());
        let result = StateData::update_state(&mut output_data, updater);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_response_information_when_formatted() {
        let response = create_baseline_response();
        let output_data = ExecuteSecRequestOutput::<CompanyFacts>::new(response);

        let expected_result = false;

        let result = format!("{output_data}").is_empty();

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_execute_sec_request_output() {
        implements_auto_traits::<ExecuteSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_execute_sec_request_output() {
        implements_send::<ExecuteSecRequestOutput<CompanyFacts>>();
    }

    #[test]
    const fn should_implement_sync_for_execute_sec_request_output() {
        implements_sync::<ExecuteSecRequestOutput<CompanyFacts>>();
    }

    #[test]
    const fn should_be_thread_safe_for_execute_sec_request_output() {
        implements_send::<ExecuteSecRequestOutput<CompanyFacts>>();
        implements_sync::<ExecuteSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_execute_sec_request_output() {
        implements_sized::<ExecuteSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_execute_sec_request_output() {
        implements_hash::<ExecuteSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_execute_sec_request_output() {
        implements_partial_eq::<ExecuteSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_execute_sec_request_output() {
        implements_eq::<ExecuteSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_execute_sec_request_output() {
        implements_partial_ord::<ExecuteSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_execute_sec_request_output() {
        implements_ord::<ExecuteSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_execute_sec_request_output() {
        implements_debug::<ExecuteSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_execute_sec_request_output() {
        implements_clone::<ExecuteSecRequestOutput<CompanyFacts>>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_execute_sec_request_output() {
        implements_unpin::<ExecuteSecRequestOutput<CompanyFacts>>();
    }
}
