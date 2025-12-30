//! # Transition: `ExecuteSecRequest` → `ValidateSecResponse`
//!
//! This module implements the state transition from [`ExecuteSecRequest`] to [`ValidateSecResponse`].
//! The transition extracts the HTTP response from the executed request and initializes the next state
//! for response validation.
//!
//! ## Transition Flow
//! 1. Extracts output data (HTTP response) from the source [`ExecuteSecRequest`] state
//! 2. Converts the source context to [`ValidateSecResponseContext`]
//! 3. Converts the output to [`ValidateSecResponseInput`]
//! 4. Constructs and returns a new [`ValidateSecResponse`] state
//!
//! ## Error Handling
//! Returns a [`TransitionError`] if the source state lacks required output data.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::{
    ExecuteSecRequest, ExecuteSecRequestContext, ExecuteSecRequestOutput,
};
use crate::implementations::states::extract::validate_sec_response::{
    ValidateSecResponse, ValidateSecResponseContext, ValidateSecResponseInput,
};

use state_maschine::prelude::State;

impl From<ExecuteSecRequestContext> for ValidateSecResponseContext {
    fn from(context: ExecuteSecRequestContext) -> Self {
        Self::new(context.cik)
    }
}

impl From<ExecuteSecRequestOutput> for ValidateSecResponseInput {
    fn from(output_data: ExecuteSecRequestOutput) -> Self {
        Self::new(output_data.response)
    }
}

impl TryFrom<ExecuteSecRequest> for ValidateSecResponse {
    type Error = TransitionError;

    fn try_from(state: ExecuteSecRequest) -> Result<Self, TransitionError> {
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

        let state_context = state.context_data().clone();
        let new_context: ValidateSecResponseContext = state_context.into();
        let new_input: ValidateSecResponseInput = output_data.into();

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::sec_response::SecResponse;

    #[test]
    fn should_convert_context_when_valid_context() {
        let cik = Cik::new("0001234567").expect("Hardcoded valid CIK string should always parse successfully");
        let context = ExecuteSecRequestContext::new(cik.clone());

        let expected_result = ValidateSecResponseContext::new(cik);

        let result: ValidateSecResponseContext = context.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_output_to_input_when_valid_output() {
        let response = SecResponse::default();
        let output = ExecuteSecRequestOutput::new(response.clone())
            .expect("Valid HTTP response should always create output successfully");

        let expected_result = ValidateSecResponseInput::new(response);

        let result: ValidateSecResponseInput = output.into();

        assert_eq!(result, expected_result);
    }
}
