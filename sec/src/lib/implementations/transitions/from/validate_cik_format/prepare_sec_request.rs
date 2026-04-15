//! # Transition: `ValidateCikFormat` → `PrepareSecRequest`
//!
//! This module implements the state transition from [`ValidateCikFormat`] to [`PrepareSecRequest`].
//! The transition extracts validated CIK output data and uses it to initialize the next state's
//! input and context.
//!
//! ## Transition Flow
//! 1. Extracts output data from the source [`ValidateCikFormat`] state
//! 2. Converts the output to [`PrepareSecRequestContext`] and [`PrepareSecRequestInput`]
//! 3. Constructs and returns a new [`PrepareSecRequest`] state
//!
//! ## Error Handling
//! Returns a [`TransitionError`] if the source state lacks required output data.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use crate::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use crate::implementations::states::extract::validate_cik_format::constants::STATE_NAME as VALIDATE_CIK_FORMAT;
use crate::shared::user_agent::constants::DEFAULT_SEC_USER_AGENT;

impl TryFrom<ValidateCikFormat> for PrepareSecRequest {
    type Error = TransitionError;

    fn try_from(state: ValidateCikFormat) -> Result<Self, TransitionError> {
        let (_input, output, _context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(VALIDATE_CIK_FORMAT, PREPARE_SEC_REQUEST)
        })?;

        // Both context and input need the CIK -- clone it once
        let new_context = PrepareSecRequestContext::new(output_data.validated_cik.clone());
        let new_input = PrepareSecRequestInput::new(
            output_data.validated_cik,
            DEFAULT_SEC_USER_AGENT.to_string(),
        );

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tokio;

    use super::*;
    use crate::implementations::states::extract::validate_cik_format::{
        ValidateCikFormat, ValidateCikFormatContext, ValidateCikFormatInput,
    };
    use crate::shared::cik::Cik;
    use crate::traits::state_machine::state::State;

    #[tokio::test]
    async fn should_transition_to_prepare_sec_request_when_validate_cik_format_has_output() {
        let cik_string = "0001234567";
        let input = ValidateCikFormatInput {
            raw_cik: cik_string.into(),
        };
        let context = ValidateCikFormatContext::default();
        let mut state = ValidateCikFormat::new(input, context);
        state
            .compute_output_data_async()
            .await
            .expect("Hardcoded valid CIK should always compute successfully");

        let expected_cik = Cik::new(cik_string)
            .expect("Hardcoded valid CIK string should always parse successfully");
        let expected_context = PrepareSecRequestContext::new(expected_cik.clone());
        let expected_input =
            PrepareSecRequestInput::new(expected_cik, DEFAULT_SEC_USER_AGENT.to_string());
        let expected_result = PrepareSecRequest::new(expected_input, expected_context);

        let result = PrepareSecRequest::try_from(state)
            .expect("State with computed output should always transition successfully");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_validate_cik_format_has_no_output() {
        let state = ValidateCikFormat::default();

        let expected_result = true;

        let result = PrepareSecRequest::try_from(state).is_err();

        assert_eq!(result, expected_result);
    }
}
