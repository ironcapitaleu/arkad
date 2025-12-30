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
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use crate::implementations::states::extract::validate_cik_format::{
    ValidateCikFormat, ValidateCikFormatOutput,
};
use crate::shared::user_agent::constants::DEFAULT_SEC_USER_AGENT;

use state_maschine::prelude::State;

impl From<ValidateCikFormatOutput> for PrepareSecRequestContext {
    fn from(output_data: ValidateCikFormatOutput) -> Self {
        Self::new(output_data.validated_cik)
    }
}

impl From<ValidateCikFormatOutput> for PrepareSecRequestInput {
    fn from(output_data: ValidateCikFormatOutput) -> Self {
        Self::new(
            output_data.validated_cik,
            DEFAULT_SEC_USER_AGENT.to_string(),
        )
    }
}

impl TryFrom<ValidateCikFormat> for PrepareSecRequest {
    type Error = TransitionError;

    fn try_from(state: ValidateCikFormat) -> Result<Self, TransitionError> {
        let output_data = match state.output_data() {
            Some(data) => data.clone(),
            None => {
                return Err(transition::MissingOutput::new(
                    "Extract SuperState",
                    state.state_name().to_string(),
                )
                .into());
            }
        };

        let new_context: PrepareSecRequestContext = output_data.clone().into();
        let new_input: PrepareSecRequestInput = output_data.into();

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::cik::Cik;

    #[test]
    fn should_convert_output_to_context_when_valid_output() {
        let cik_string = "0001234567";
        let output = ValidateCikFormatOutput::new(cik_string)
            .expect("Hardcoded valid CIK string should always parse successfully");
        let expected_cik = Cik::new(cik_string)
            .expect("Hardcoded valid CIK string should always parse successfully");

        let expected_result = PrepareSecRequestContext::new(expected_cik);

        let result: PrepareSecRequestContext = output.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_output_to_input_when_valid_output() {
        let cik_string = "0001234567";
        let output = ValidateCikFormatOutput::new(cik_string)
            .expect("Hardcoded valid CIK string should always parse successfully");
        let expected_cik = Cik::new(cik_string)
            .expect("Hardcoded valid CIK string should always parse successfully");

        let expected_result =
            PrepareSecRequestInput::new(expected_cik, DEFAULT_SEC_USER_AGENT.to_string());

        let result: PrepareSecRequestInput = output.into();

        assert_eq!(result, expected_result);
    }
}
