//! # Transition: `PrepareSecRequest` → `ExecuteSecRequest`
//!
//! Implements the [`TryFrom`] conversion producing an [`ExecuteSecRequest`] from a
//! [`PrepareSecRequest`], the second extract-phase transition.
//!
//! It moves the prepared client and request from the source state's output into the next state's
//! input, and carries the CIK across via context. The supporting `From` impls perform the
//! field-level conversions; the `TryFrom` is fallible because the source output may be absent.
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output to carry forward.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::constants::STATE_NAME as EXECUTE_SEC_REQUEST;
use crate::implementations::states::extract::execute_sec_request::{
    ExecuteSecRequest, ExecuteSecRequestContext, ExecuteSecRequestInput,
};
use crate::implementations::states::extract::prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestOutput,
};

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
        let (_input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(PREPARE_SEC_REQUEST, EXECUTE_SEC_REQUEST)
        })?;

        let new_context: ExecuteSecRequestContext = context.into();
        let new_input: ExecuteSecRequestInput = output_data.into();

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::implementations::states::extract::prepare_sec_request::PrepareSecRequestInput;
    use crate::shared::cik::Cik;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::request::implementations::sec_request::SecRequest;
    use crate::traits::state_machine::state::State;

    #[test]
    fn should_convert_context_when_valid_context() {
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let context = PrepareSecRequestContext::new(cik.clone());

        let expected_result = ExecuteSecRequestContext::new(cik);

        let result = ExecuteSecRequestContext::from(context);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_output_to_input_when_valid_output() {
        let client = SecClient::default();
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let request = SecRequest::builder()
            .all_company_facts()
            .cik(cik.clone())
            .build();
        let output = PrepareSecRequestOutput::new(client.clone(), request.clone());

        let expected_result = ExecuteSecRequestInput::new(client, request);

        let result = ExecuteSecRequestInput::from(output);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_transition_to_execute_sec_request_when_prepare_sec_request_has_output() {
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let sec_client = SecClient::default();
        let input = PrepareSecRequestInput::new(cik.clone(), sec_client);
        let context = PrepareSecRequestContext::new(cik.clone());
        let mut state = PrepareSecRequest::new(input, context);
        state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");

        let expected_result = true;

        let result = ExecuteSecRequest::try_from(state).is_ok();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_prepare_sec_request_has_no_output() {
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let sec_client = SecClient::default();
        let input = PrepareSecRequestInput::new(cik.clone(), sec_client);
        let context = PrepareSecRequestContext::new(cik);
        let state = PrepareSecRequest::new(input, context);

        let expected_result = true;

        let result = ExecuteSecRequest::try_from(state).is_err();

        assert_eq!(result, expected_result);
    }
}
