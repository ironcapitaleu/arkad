//! # Prepare SEC Request State
//!
//! Provides the [`PrepareSecRequest`] state, the second step of the extract phase, which
//! builds the [`SecRequest`] targeting the SEC API endpoint for a validated CIK.
//!
//! Taking a validated [`Cik`](crate::shared::cik::Cik) and the shared HTTP client, it
//! assembles the company-facts request and pairs it with that client, leaving a ready-to-send
//! request for the next state. Separating preparation from execution keeps request
//! construction synchronous and independently testable, without touching the network.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`PrepareSecRequestContext`] carried alongside the state.
//! - [`data`]: The [`PrepareSecRequestInput`] and [`PrepareSecRequestOutput`] data types.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::prepare_sec_request::*;
//! use sec::prelude::*;
//! use sec::shared::cik::Cik;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
//! let input = PrepareSecRequestInput::new(cik.clone(), SecClient::default());
//! let context = PrepareSecRequestContext::new(cik);
//!
//! let mut state = PrepareSecRequest::new(input, context);
//! state
//!     .compute_output_data_async()
//!     .await
//!     .expect("Request preparation is infallible for a valid CIK");
//!
//! let output = state
//!     .output_data()
//!     .expect("Output is present after a successful computation");
//! let _request = output.request();
//! # }
//! ```
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract`]: Parent module describing the extraction flow.
//! - [`crate::shared::request::implementations::sec_request::SecRequest`]: The request type produced by this state.
//! - [`crate::traits::state_machine::state::State`]: The trait implemented by [`PrepareSecRequest`].

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::PrepareSecRequestContext;
pub use data::PrepareSecRequestInput;
pub use data::PrepareSecRequestOutput;

/// Second extract-phase state, building the [`SecRequest`] for a validated CIK.
///
/// Takes a validated [`Cik`](crate::shared::cik::Cik) and the shared HTTP client and
/// assembles a company-facts request targeting the matching SEC endpoint, paired with that
/// client for the executing state. The work is purely local: no network call is made here.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct PrepareSecRequest {
    input: PrepareSecRequestInput,
    context: PrepareSecRequestContext,
    output: Option<PrepareSecRequestOutput>,
}

impl PrepareSecRequest {
    /// Creates a new state from its input and context, with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::prepare_sec_request::*;
    /// use sec::shared::cik::Cik;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    ///
    /// let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
    /// let input = PrepareSecRequestInput::new(cik.clone(), SecClient::default());
    /// let context = PrepareSecRequestContext::new(cik);
    ///
    /// let state = PrepareSecRequest::new(input, context);
    /// ```
    #[must_use]
    pub const fn new(input: PrepareSecRequestInput, context: PrepareSecRequestContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    ///
    /// Used by transitions to move the prepared request into the next state without cloning.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        PrepareSecRequestInput,
        Option<PrepareSecRequestOutput>,
        PrepareSecRequestContext,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl State for PrepareSecRequest {
    /// Builds the company-facts [`SecRequest`] for the input CIK and stores it as output.
    ///
    /// # Errors
    ///
    /// Infallible in practice; the [`Result`] exists to satisfy the [`State`] contract.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let sec_client = self.input.sec_client.clone();
        let sec_request = SecRequest::builder()
            .all_company_facts()
            .cik(self.input.validated_cik.clone())
            .build();

        self.output = Some(PrepareSecRequestOutput::new(sec_client, sec_request));

        Ok(())
    }
}

impl SMState for PrepareSecRequest {
    type InputData = PrepareSecRequestInput;
    type OutputData = PrepareSecRequestOutput;
    type Context = PrepareSecRequestContext;

    /// Returns the human-readable name of this state.
    fn state_name(&self) -> impl ToString {
        STATE_NAME
    }

    /// Blocking wrapper around [`compute_output_data_async`](crate::traits::state_machine::state::State::compute_output_data_async).
    ///
    /// Detects whether a tokio runtime is available and runs the async computation
    /// synchronously. This allows SEC states to be used as regular `SMState` implementations.
    ///
    /// # Panics
    /// - Panics if the async computation returns an error.
    /// - Panics if called inside a `current_thread` runtime (`block_in_place`
    ///   requires a multi-thread runtime). All binaries and tests in this project
    ///   use `flavor = "multi_thread"`.
    fn compute_output_data(&mut self) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(self.compute_output_data_async()))
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.compute_output_data_async())
        };

        if let Err(e) = result {
            let state_err: StateError = e;
            panic!("compute_output_data failed: {state_err}")
        }
    }

    fn context_data(&self) -> &Self::Context {
        &self.context
    }

    fn input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl fmt::Display for PrepareSecRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::traits::state_machine::state::State;
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};
    use tokio;

    fn create_test_cik() -> Cik {
        Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid")
    }

    fn create_test_input() -> PrepareSecRequestInput {
        let sec_client = SecClient::default();
        PrepareSecRequestInput::new(create_test_cik(), sec_client)
    }

    fn create_test_context() -> PrepareSecRequestContext {
        PrepareSecRequestContext::new(create_test_cik())
    }

    fn create_test_state() -> PrepareSecRequest {
        PrepareSecRequest::new(create_test_input(), create_test_context())
    }

    #[test]
    fn should_return_name_of_prepare_state_when_in_prepare_state() {
        let prepare_state = create_test_state();
        let expected_result = String::from("Prepare SEC Request");
        let result = prepare_state.state_name().to_string();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_prepare_data_struct_as_input_data_when_in_initial_prepare_state() {
        let prepare_state = create_test_state();
        let expected_result = &create_test_input();
        let result = prepare_state.input_data();
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let prepare_state = create_test_state();
        let _result = prepare_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let prepare_state = create_test_state();
        let expected_result = false;
        let result = prepare_state.has_output_data_been_computed();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let prepare_state = create_test_state();
        let expected_result = &create_test_context();
        let result = prepare_state.context_data();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_new_prepare_state_with_provided_input_and_context() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let sec_client = SecClient::default();
        let input = PrepareSecRequestInput::new(cik, sec_client);
        let context = create_test_context();

        let expected_input = input.clone();
        let expected_context = context.clone();

        let result = PrepareSecRequest::new(input, context);

        assert_eq!(result.input_data(), &expected_input);
        assert_eq!(result.context_data(), &expected_context);
        assert!(result.output_data().is_none());
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_trait() {
        implements_auto_traits::<PrepareSecRequest>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_trait() {
        implements_send::<PrepareSecRequest>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_trait() {
        implements_sync::<PrepareSecRequest>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_trait() {
        implements_send::<PrepareSecRequest>();
        implements_sync::<PrepareSecRequest>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_trait() {
        implements_sized::<PrepareSecRequest>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_trait() {
        implements_hash::<PrepareSecRequest>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_trait() {
        implements_partial_eq::<PrepareSecRequest>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_trait() {
        implements_eq::<PrepareSecRequest>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_trait() {
        implements_partial_ord::<PrepareSecRequest>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_trait() {
        implements_ord::<PrepareSecRequest>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_trait() {
        implements_debug::<PrepareSecRequest>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_trait() {
        implements_clone::<PrepareSecRequest>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_trait() {
        implements_unpin::<PrepareSecRequest>();
    }

    #[test]
    fn should_return_default_context_data_when_called_with_state_reference() {
        let prepare_state = &create_test_state();
        let ref_to_prepare_state = &create_test_state();

        let expected_result = prepare_state.context_data();
        let result = ref_to_prepare_state.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_prepare_state = &mut create_test_state();
        let expected_result = false;
        let result = ref_to_prepare_state.has_output_data_been_computed();
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_reference_state()
     {
        let ref_to_prepare_state = &create_test_state();
        let _result = ref_to_prepare_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_name_of_prepare_state_when_calling_reference_to_prepare_state() {
        let ref_to_prepare_state = &create_test_state();
        let expected_result = String::from("Prepare SEC Request");
        let result = ref_to_prepare_state.state_name().to_string();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_as_input_data_when_reference_prepare_state_in_initial_state()
     {
        let ref_to_prepare_state = &create_test_state();
        let expected_result = &create_test_input();
        let result = ref_to_prepare_state.input_data();
        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_change_input_data_when_computing_output_data() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let sec_client = SecClient::default();
        let input = PrepareSecRequestInput::new(cik, sec_client);
        let context = create_test_context();
        let mut prepare_state = PrepareSecRequest::new(input, context);

        let expected_result = &prepare_state.input_data().clone();

        prepare_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");
        let result = prepare_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_correct_output_data_when_computing_output_data() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let sec_client = SecClient::default();
        let input = PrepareSecRequestInput::new(cik, sec_client);
        let context = create_test_context();
        let mut prepare_state = PrepareSecRequest::new(input, context);

        let expected_result = "https://data.sec.gov/api/xbrl/companyfacts/CIK1234567890.json";

        prepare_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");
        let result = prepare_state
            .output_data()
            .unwrap()
            .request
            .inner
            .url()
            .as_str();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_true_when_output_data_has_been_computed() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let sec_client = SecClient::default();
        let input = PrepareSecRequestInput::new(cik, sec_client);
        let context = create_test_context();
        let mut prepare_state = PrepareSecRequest::new(input, context);

        let expected_result = true;

        prepare_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");
        let result = prepare_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_output_when_calling_sync_compute_outside_tokio_runtime() {
        let mut state = create_test_state();

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let mut state = create_test_state();

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }
}
