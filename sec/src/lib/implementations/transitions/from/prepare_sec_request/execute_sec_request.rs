//! # Transition: `PrepareSecRequest` → `ExecuteSecRequest`
//!
//! This module implements the state transition from [`PrepareSecRequest`] to [`ExecuteSecRequest`].
//! The transition extracts the prepared SEC request and HTTP client from the source state and
//! initializes the next state for executing the HTTP request.
//!
//! ## Transition Flow
//! 1. Extracts output data (client and request) from the source [`PrepareSecRequest`] state
//! 2. Converts the source context to [`ExecuteSecRequestContext`]
//! 3. Converts the output to [`ExecuteSecRequestInput`]
//! 4. Constructs and returns a new [`ExecuteSecRequest`] state
//!
//! ## Error Handling
//! Returns a [`TransitionError`] if the source state lacks required output data.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::{
    ExecuteSecRequest, ExecuteSecRequestContext, ExecuteSecRequestInput,
};
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestOutput,
};

use state_maschine::prelude::State;

impl From<PrepareSecRequestContext> for ExecuteSecRequestContext {
    fn from(context: PrepareSecRequestContext) -> Self {
        Self::new(context.cik)
    }
}

impl From<PrepareSecRequestOutput> for ExecuteSecRequestInput {
    fn from(output_data: PrepareSecRequestOutput) -> Self {
        Self::new(output_data.client, output_data.request)
    }
}

impl TryFrom<PrepareSecRequest> for ExecuteSecRequest {
    type Error = TransitionError;

    fn try_from(state: PrepareSecRequest) -> Result<Self, TransitionError> {
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
        let new_context: ExecuteSecRequestContext = state_context.into();
        let new_input: ExecuteSecRequestInput = output_data.into();

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::sec_client::SecClient;
    use crate::shared::sec_request::SecRequest;

    #[test]
    fn should_convert_context_when_valid_context() {
        let cik = Cik::new("0001234567").expect("Hardcoded valid CIK string should always parse successfully");
        let context = PrepareSecRequestContext::new(cik.clone());

        let expected_result = ExecuteSecRequestContext::new(cik);

        let result: ExecuteSecRequestContext = context.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_output_to_input_when_valid_output() {
        let client = SecClient::default();
        let request = SecRequest::default();
        let output = PrepareSecRequestOutput::new(client.clone(), request.clone())
            .expect("Valid client and request should always create output successfully");

        let expected_result = ExecuteSecRequestInput::new(client, request);

        let result: ExecuteSecRequestInput = output.into();

        assert_eq!(result, expected_result);
    }
}
