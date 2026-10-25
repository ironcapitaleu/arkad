//! # Transition: `PrepareSecRequest` → `ExecuteSecRequest`
//!
//! Implements the [`TryFrom`] conversion producing an [`ExecuteSecRequest`] from a
//! [`PrepareSecRequest`].
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

impl<R> From<PrepareSecRequestContext<R>> for ExecuteSecRequestContext<R> {
    fn from(context: PrepareSecRequestContext<R>) -> Self {
        Self::new(
            context.cik,
            context.execution_id,
            context.user_agent,
            context.repository,
        )
    }
}

impl From<PrepareSecRequestOutput> for ExecuteSecRequestInput {
    fn from(output_data: PrepareSecRequestOutput) -> Self {
        Self::new(output_data.client, output_data.request)
    }
}

impl<R> TryFrom<PrepareSecRequest<R>> for ExecuteSecRequest<R> {
    type Error = TransitionError;

    fn try_from(state: PrepareSecRequest<R>) -> Result<Self, TransitionError> {
        let (_input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(PREPARE_SEC_REQUEST, EXECUTE_SEC_REQUEST)
        })?;

        let new_context: ExecuteSecRequestContext<R> = context.into();
        let new_input: ExecuteSecRequestInput = output_data.into();

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use storage::WriteError;
    use uuid::Uuid;

    use super::*;
    use crate::implementations::states::extract::prepare_sec_request::PrepareSecRequestInput;
    use crate::shared::cik::Cik;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::request::implementations::sec_request::SecRequest;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_run::{SAMPLE_EXECUTION_ID, sample_user_agent};
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;
    use crate::traits::state_machine::state::State;

    #[test]
    fn should_convert_context_when_valid_context() {
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let context = PrepareSecRequestContext::new(
            cik.clone(),
            SAMPLE_EXECUTION_ID,
            sample_user_agent(),
            FakeWriteRepository::new(),
        );

        let expected_result = ExecuteSecRequestContext::new(
            cik,
            SAMPLE_EXECUTION_ID,
            sample_user_agent(),
            FakeWriteRepository::new(),
        );

        let result = ExecuteSecRequestContext::from(context);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_output_to_input_when_valid_output() {
        let client = SecClient::default();
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
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
        let context = PrepareSecRequestContext::new(
            cik.clone(),
            SAMPLE_EXECUTION_ID,
            sample_user_agent(),
            FakeWriteRepository::new(),
        );
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
        let context = PrepareSecRequestContext::new(
            cik,
            SAMPLE_EXECUTION_ID,
            sample_user_agent(),
            FakeWriteRepository::new(),
        );
        let state = PrepareSecRequest::new(input, context);

        let expected_result = true;

        let result = ExecuteSecRequest::try_from(state).is_err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_carry_execution_id_when_converting_context() {
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let context = PrepareSecRequestContext::new(
            cik,
            Uuid::from_u128(42),
            sample_user_agent(),
            FakeWriteRepository::new(),
        );

        let expected_result = Uuid::from_u128(42);

        let result = ExecuteSecRequestContext::from(context).execution_id();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_carry_user_agent_when_converting_context() {
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let other_user_agent = UserAgent::new("Other Company other@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        let context = PrepareSecRequestContext::new(
            cik,
            SAMPLE_EXECUTION_ID,
            other_user_agent.clone(),
            FakeWriteRepository::new(),
        );

        let expected_result = other_user_agent;

        let result = ExecuteSecRequestContext::from(context).user_agent().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_carry_repository_when_converting_context() {
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let context = PrepareSecRequestContext::new(
            cik,
            SAMPLE_EXECUTION_ID,
            sample_user_agent(),
            FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict")),
        );

        let expected_result =
            FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict"));

        let result = ExecuteSecRequestContext::from(context).repository().clone();

        assert_eq!(result, expected_result);
    }
}
