//! # Transition: `PrepareSecRequest` → `ExecuteSecRequest`
//!
//! Implements the [`TryFrom`] conversion producing an [`ExecuteSecRequest`] from a
//! [`PrepareSecRequest`] and an SEC client, for every request kind `K`.
//!
//! It moves the prepared request from the output of the source state into the input of the next
//! state, and carries the request input and the repository across in the context. The source state
//! holds no client, so the caller passes the one from the state machine context. The `From` impl
//! converts the context. The `TryFrom` is fallible because the source output may be absent.
//!
//! Both states have the same kind `K`. A prepared Company Facts request cannot become the
//! execution of a Notes file request, so this does not compile:
//!
//! ```compile_fail,E0277
//! use sec::implementations::states::extract::execute_sec_request::ExecuteSecRequest;
//! use sec::implementations::states::extract::prepare_sec_request::PrepareCompanyFactsRequest;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use sec::shared::request::implementations::sec_request::NotesFile;
//!
//! fn execute_as_notes_file<R>(state: PrepareCompanyFactsRequest<R>) {
//!     let _next = ExecuteSecRequest::<NotesFile, R, SecClient>::try_from((state, SecClient::default()));
//! }
//! ```
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
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext,
};
use crate::shared::http_client::RespondsWith;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::request::implementations::sec_request::RequestKind;

impl<K: RequestKind, R> From<PrepareSecRequestContext<K, R>> for ExecuteSecRequestContext<K, R> {
    fn from(context: PrepareSecRequestContext<K, R>) -> Self {
        Self::new(context.request_input, context.repository)
    }
}

impl<K: RespondsWith, R, C: SecClientTrait> TryFrom<(PrepareSecRequest<K, R>, C)>
    for ExecuteSecRequest<K, R, C>
{
    type Error = TransitionError;

    /// Builds an [`ExecuteSecRequest`] from a computed [`PrepareSecRequest`] of the same kind and
    /// the client that sends the request.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    fn try_from(
        (state, sec_client): (PrepareSecRequest<K, R>, C),
    ) -> Result<Self, TransitionError> {
        let (_input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(K::PREPARE_STATE_NAME, EXECUTE_SEC_REQUEST)
        })?;

        let new_context: ExecuteSecRequestContext<K, R> = context.into();
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
    use crate::implementations::states::extract::execute_sec_request::ExecuteCompanyFactsRequest;
    use crate::implementations::states::extract::prepare_sec_request::{
        PrepareCompanyFactsRequest, PrepareSecRequestInput,
    };
    use crate::shared::cik::Cik;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::SecRequest as SecRequestTrait;
    use crate::shared::request::implementations::sec_request::{
        CompanyFacts, NotesFile, SecRequest,
    };
    use crate::tests::fixtures::sample_write_repository::{
        FakeNotesRepository, FakeWriteRepository,
    };
    use crate::traits::state_machine::state::State;

    #[test]
    fn should_convert_context_when_valid_context() {
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let context = PrepareSecRequestContext::<CompanyFacts, _>::new(
            cik.clone(),
            FakeWriteRepository::new(),
        );

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
        let context = PrepareSecRequestContext::<CompanyFacts, _>::new(
            cik,
            FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict")),
        );

        let expected_result =
            FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict"));

        let result = ExecuteSecRequestContext::from(context).repository().clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_move_notes_file_request_into_input_when_prepare_sec_request_is_of_notes_file_kind()
     {
        let client = SecClient::default();
        let period = NotesPeriod::new("2024q1")
            .expect("Given a published period, NotesPeriod creation should always succeed");
        let mut state = PrepareSecRequest::builder()
            .notes_file()
            .period(period)
            .repository(FakeNotesRepository::new())
            .build();
        state
            .compute_output_data_async()
            .await
            .expect("Request preparation should always succeed for a valid period");

        let expected_result =
            ExecuteSecRequestInput::<NotesFile, _>::new(client.clone(), SecRequest::new(period));

        let result = ExecuteSecRequest::try_from((state, client))
            .expect("Given a computed state, the transition should always succeed")
            .input_data()
            .clone();

        assert_eq!(result, expected_result);
    }
}
