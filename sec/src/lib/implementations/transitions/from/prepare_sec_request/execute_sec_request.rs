//! # Transition: `PrepareSecRequest` → `ExecuteSecRequest`
//!
//! Implements the [`TryFrom`] conversion producing an [`ExecuteCompanyFactsRequest`] from a
//! [`PrepareCompanyFactsRequest`] and an SEC client.
//!
//! It moves the prepared request from the output of the source state into the input of the next
//! state, and carries the CIK and the repository across in the context. The source state holds no
//! client, so the caller passes the one from the state machine context. The `From` impl converts
//! the context. The `TryFrom` is fallible because the source output may be absent.
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output to carry forward.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::constants::STATE_NAME as EXECUTE_SEC_REQUEST;
use crate::implementations::states::extract::execute_sec_request::{
    ExecuteCompanyFactsRequest, ExecuteSecRequestContext, ExecuteSecRequestInput,
};
use crate::implementations::states::extract::prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareCompanyFactsRequest, PrepareSecRequestContext,
};
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::request::implementations::sec_request::CompanyFacts;

impl<R> From<PrepareSecRequestContext<CompanyFacts, R>>
    for ExecuteSecRequestContext<CompanyFacts, R>
{
    fn from(context: PrepareSecRequestContext<CompanyFacts, R>) -> Self {
        Self::new(context.request_input, context.repository)
    }
}

impl<R, C: SecClientTrait> TryFrom<(PrepareCompanyFactsRequest<R>, C)>
    for ExecuteCompanyFactsRequest<R, C>
{
    type Error = TransitionError;

    /// Builds an [`ExecuteCompanyFactsRequest`] from a computed [`PrepareCompanyFactsRequest`] and
    /// the client that sends the request.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    fn try_from(
        (state, sec_client): (PrepareCompanyFactsRequest<R>, C),
    ) -> Result<Self, TransitionError> {
        let (_input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(PREPARE_SEC_REQUEST, EXECUTE_SEC_REQUEST)
        })?;

        let new_context: ExecuteSecRequestContext<CompanyFacts, R> = context.into();
        let new_input = ExecuteSecRequestInput::new(sec_client, output_data.request);

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::State as SMState;
    use storage::WriteError;

    use super::*;
    use crate::implementations::states::extract::prepare_sec_request::PrepareSecRequestInput;
    use crate::shared::cik::Cik;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::request::implementations::sec_request::SecRequest;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;
    use crate::traits::state_machine::state::State;

    #[test]
    fn should_convert_context_when_valid_context() {
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let context = PrepareSecRequestContext::new(cik.clone(), FakeWriteRepository::new());

        let expected_result = ExecuteSecRequestContext::new(cik, FakeWriteRepository::new());

        let result = ExecuteSecRequestContext::from(context);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_move_request_and_client_into_input_when_prepare_sec_request_has_output() {
        let client = SecClient::default();
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let request = SecRequest::builder()
            .all_company_facts()
            .cik(cik.clone())
            .build();
        let mut state = PrepareCompanyFactsRequest::new(
            PrepareSecRequestInput::new(cik.clone()),
            PrepareSecRequestContext::new(cik, FakeWriteRepository::new()),
        );
        state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");

        let expected_result = ExecuteSecRequestInput::new(client.clone(), request);

        let result = ExecuteCompanyFactsRequest::try_from((state, client))
            .expect("Given a computed state, the transition should always succeed")
            .input_data()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_transition_to_execute_sec_request_when_prepare_sec_request_has_output() {
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let input = PrepareSecRequestInput::new(cik.clone());
        let context = PrepareSecRequestContext::new(cik.clone(), FakeWriteRepository::new());
        let mut state = PrepareCompanyFactsRequest::new(input, context);
        state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");

        let expected_result = true;

        let result = ExecuteCompanyFactsRequest::try_from((state, SecClient::default())).is_ok();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_prepare_sec_request_has_no_output() {
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let input = PrepareSecRequestInput::new(cik.clone());
        let context = PrepareSecRequestContext::new(cik, FakeWriteRepository::new());
        let state = PrepareCompanyFactsRequest::new(input, context);

        let expected_result = true;

        let result = ExecuteCompanyFactsRequest::try_from((state, SecClient::default())).is_err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_carry_repository_when_converting_context() {
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let context = PrepareSecRequestContext::new(
            cik,
            FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict")),
        );

        let expected_result =
            FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict"));

        let result = ExecuteSecRequestContext::from(context).repository().clone();

        assert_eq!(result, expected_result);
    }
}
