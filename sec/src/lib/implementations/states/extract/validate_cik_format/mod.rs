//! # Validate CIK Format State
//!
//! Provides the [`ValidateCikFormat`] state, the first step of the extract phase, which
//! syntactically validates and normalizes a raw CIK (Central Index Key) string.
//!
//! The SEC identifies every filer by a CIK that must be exactly ten zero-padded digits.
//! This state encapsulates that invariant at the pipeline entry point so that every
//! downstream state can rely on a well-formed [`Cik`] without re-validating. It performs
//! *syntactic* validation only and does not verify that the CIK exists in the SEC's records.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`ValidateCikFormatContext`] carried alongside the state.
//! - [`data`]: The [`ValidateCikFormatInput`] and [`ValidateCikFormatOutput`] data types.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::validate_cik_format::*;
//! use sec::prelude::*;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let input = ValidateCikFormatInput::new("1234");
//! let context = ValidateCikFormatContext::new("1234", SecClient::default());
//!
//! let mut state = ValidateCikFormat::new(input, context);
//! state
//!     .compute_output_data_async()
//!     .await
//!     .expect("A syntactically valid CIK should always validate successfully");
//!
//! let validated = state
//!     .output_data()
//!     .expect("Output is present after a successful computation");
//! assert_eq!(validated.validated_cik.value(), "0000001234");
//! # }
//! ```
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract`]: Parent module describing the extraction flow.
//! - [`crate::shared::cik::Cik`]: The validated CIK type produced by this state.
//! - [`crate::traits::state_machine::state::State`]: The trait implemented by [`ValidateCikFormat`].

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::error::state_machine::state::InvalidCikFormat;
use crate::traits::error::FromDomainError;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::ValidateCikFormatContext;
pub use data::ValidateCikFormatInput;
pub use data::ValidateCikFormatOutput;

use crate::shared::cik::Cik;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
/// First extract-phase state, validating and normalizing a raw CIK into a [`Cik`].
///
/// Takes an unvalidated CIK string, trims whitespace, rejects any non-numeric input,
/// and zero-pads the result to the ten-digit form the SEC requires. Existing as a
/// dedicated state means the rest of the pipeline can consume a guaranteed-valid
/// [`Cik`] rather than re-checking the format at every step.
///
/// Validation is **syntactic only**: a well-formed CIK that does not correspond to any
/// real filer still passes. Existence is established later, when the SEC request is executed.
pub struct ValidateCikFormat {
    input: ValidateCikFormatInput,
    context: ValidateCikFormatContext,
    output: Option<ValidateCikFormatOutput>,
}

impl ValidateCikFormat {
    /// Creates a new state from its input and context, with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::validate_cik_format::*;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    ///
    /// let input = ValidateCikFormatInput::new("1234");
    /// let context = ValidateCikFormatContext::new("1234", SecClient::default());
    ///
    /// let state = ValidateCikFormat::new(input, context);
    /// ```
    #[must_use]
    pub const fn new(input: ValidateCikFormatInput, context: ValidateCikFormatContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    ///
    /// Used by transitions to move the validated [`Cik`] into the next state without cloning.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        ValidateCikFormatInput,
        Option<ValidateCikFormatOutput>,
        ValidateCikFormatContext,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl State for ValidateCikFormat {
    /// Validates the raw CIK and, on success, stores the normalized [`Cik`] as output.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::InvalidCikFormat`] when the raw CIK contains non-numeric
    /// characters or exceeds the maximum allowed length.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        // Validate the CIK format
        let cik = Cik::new(&self.input.raw_cik);

        match cik {
            Ok(cik) => {
                // If the CIK is valid, set the output data
                self.output = Some(ValidateCikFormatOutput { validated_cik: cik });
            }
            Err(e) => {
                let e: StateError =
                    InvalidCikFormat::from_domain_error(self.state_name().to_string(), e).into();
                // If the CIK is invalid, return an error
                return Err(e);
            }
        }

        Ok(())
    }
}

impl SMState for ValidateCikFormat {
    type InputData = ValidateCikFormatInput;
    type OutputData = ValidateCikFormatOutput;
    type Context = ValidateCikFormatContext;

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

impl fmt::Display for ValidateCikFormat {
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
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::shared::http_client::implementations::sec_client::SecClient;

    use super::*;
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};
    use tokio;

    fn test_context() -> ValidateCikFormatContext {
        let sec_client = SecClient::default();
        ValidateCikFormatContext::new(BERKSHIRE_HATHAWAY_CIK_RAW, sec_client)
    }

    fn test_input() -> ValidateCikFormatInput {
        ValidateCikFormatInput::new(BERKSHIRE_HATHAWAY_CIK_RAW)
    }

    fn test_state() -> ValidateCikFormat {
        ValidateCikFormat::new(test_input(), test_context())
    }

    #[test]
    fn should_return_name_of_validation_state_when_in_validation_state() {
        let validation_state = test_state();

        let expected_result = String::from("Validate CIK Format");

        let result = validation_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_validation_data_struct_as_input_data_when_in_initial_validation_state()
    {
        let validation_state = test_state();

        let expected_result = &test_input();

        let result = validation_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let validation_state = test_state();

        let _result = validation_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let validation_state = test_state();

        let expected_result = false;

        let result = validation_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let validation_state = test_state();

        let expected_result = &test_context();

        let result = validation_state.context_data();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_trait() {
        implements_auto_traits::<ValidateCikFormat>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_trait() {
        implements_send::<ValidateCikFormat>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_trait() {
        implements_sync::<ValidateCikFormat>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_trait() {
        implements_send::<ValidateCikFormat>();
        implements_sync::<ValidateCikFormat>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_trait() {
        implements_sized::<ValidateCikFormat>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_trait() {
        implements_hash::<ValidateCikFormat>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_trait() {
        implements_partial_eq::<ValidateCikFormat>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_trait() {
        implements_eq::<ValidateCikFormat>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_trait() {
        implements_partial_ord::<ValidateCikFormat>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_trait() {
        implements_ord::<ValidateCikFormat>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_trait() {
        implements_debug::<ValidateCikFormat>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_trait() {
        implements_clone::<ValidateCikFormat>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_trait() {
        implements_unpin::<ValidateCikFormat>();
    }

    #[test]
    fn should_return_default_context_data_when_called_with_state_reference() {
        let validation_state = &test_state();
        let ref_to_validation_state = &test_state();

        let expected_result = validation_state.context_data();

        let result = ref_to_validation_state.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_validation_state = &mut test_state();

        let expected_result = false;

        let result = ref_to_validation_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_reference_state()
     {
        let ref_to_validation_state = &test_state();

        let _result = ref_to_validation_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_name_of_validation_state_when_calling_reference_to_validation_state() {
        let ref_to_validation_state = &test_state();

        let expected_result = String::from("Validate CIK Format");

        let result = ref_to_validation_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_as_input_data_when_reference_validation_state_in_initial_state()
     {
        let ref_to_validation_state = &test_state();

        let expected_result = &test_input();

        let result = ref_to_validation_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_change_input_data_when_computing_output_data() {
        let mut validation_state = test_state();

        let expected_result = &validation_state.input_data().clone();

        validation_state
            .compute_output_data_async()
            .await
            .expect("Default test state should always compute output successfully");
        let result = validation_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_correct_output_data_when_computing_output_data() {
        let mut validation_state = test_state();
        let output_data = ValidateCikFormatOutput::new(BERKSHIRE_HATHAWAY_CIK_RAW);

        let expected_result = &output_data.expect("Output data has been created above");

        validation_state
            .compute_output_data_async()
            .await
            .expect("Default test state should always compute output successfully");
        let result = validation_state.output_data().unwrap();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_output_when_calling_sync_compute_outside_tokio_runtime() {
        let mut state = test_state();

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let mut state = test_state();

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "compute_output_data failed")]
    fn should_panic_when_calling_sync_compute_with_invalid_input() {
        let input = ValidateCikFormatInput {
            raw_cik: "INVALID".to_string(),
        };
        let context = test_context();
        let mut state = ValidateCikFormat::new(input, context);

        state.compute_output_data();
    }
}
