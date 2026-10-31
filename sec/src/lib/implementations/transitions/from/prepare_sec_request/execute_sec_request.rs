//! # Transition: `PrepareSecRequest` → `ExecuteSecRequest`
//!
//! Implements the [`TryFrom`] conversion producing an [`ExecuteSecRequest`] from a
//! [`PrepareSecRequest`] and an SEC client, for every request kind `K`.
//!
//! It moves the prepared request from the output of the source state into the input of the next
//! state. The context takes the request input from the source input, which built the request. The
//! source state holds no client, so the caller passes the one from the state machine context. The
//! `TryFrom` is fallible because the source output may be absent.
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
//! fn execute_as_notes_file(state: PrepareCompanyFactsRequest) {
//!     let _next = ExecuteSecRequest::<NotesFile, SecClient>::try_from((state, SecClient::default()));
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
use crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
use crate::implementations::states::extract::prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
use crate::shared::http_client::RespondsWith;
use crate::shared::http_client::SecClient as SecClientTrait;

impl<K: RespondsWith, C: SecClientTrait> TryFrom<(PrepareSecRequest<K>, C)>
    for ExecuteSecRequest<K, C>
{
    type Error = TransitionError;

    /// Builds an [`ExecuteSecRequest`] from a computed [`PrepareSecRequest`] of the same kind and
    /// the client that sends the request.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    fn try_from((state, sec_client): (PrepareSecRequest<K>, C)) -> Result<Self, TransitionError> {
        let (input, output, _context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(PREPARE_SEC_REQUEST, EXECUTE_SEC_REQUEST)
        })?;

        // The request comes from the input, so the context takes its input from there too.
        let new_context = ExecuteSecRequestContext::new(input.request_input);
        let new_input = ExecuteSecRequestInput::new(sec_client, output_data.request);

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::State as SMState;

    use super::*;
    use crate::implementations::states::extract::execute_sec_request::ExecuteCompanyFactsRequest;
    use crate::implementations::states::extract::prepare_sec_request::{
        PrepareCompanyFactsRequest, PrepareSecRequestContext, PrepareSecRequestInput,
    };
    use crate::shared::cik::Cik;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::SecRequest as SecRequestTrait;
    use crate::shared::request::implementations::sec_request::{
        NotesFile, SecRequest, Submissions,
    };
    use crate::traits::state_machine::state::State;

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
            PrepareSecRequestContext::new(cik),
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
        let client = SecClient::default();
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let input = PrepareSecRequestInput::new(cik.clone());
        let context = PrepareSecRequestContext::new(cik.clone());
        let mut state = PrepareCompanyFactsRequest::new(input, context);
        state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");

        let expected_result = true;

        let result = ExecuteCompanyFactsRequest::try_from((state, client)).is_ok();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_cik_from_input_when_input_and_context_name_different_ciks() {
        let client = SecClient::default();
        let input_cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let context_cik = Cik::new("0000320193")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let mut state = PrepareCompanyFactsRequest::new(
            PrepareSecRequestInput::new(input_cik.clone()),
            PrepareSecRequestContext::new(context_cik),
        );
        state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");

        let expected_result = input_cik;

        let result = ExecuteCompanyFactsRequest::try_from((state, client))
            .expect("Given a computed state, the transition should always succeed")
            .context_data()
            .request_input
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_prepare_sec_request_has_no_output() {
        let client = SecClient::default();
        let cik = Cik::new("0001234567")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let input = PrepareSecRequestInput::new(cik.clone());
        let context = PrepareSecRequestContext::new(cik);
        let state = PrepareCompanyFactsRequest::new(input, context);

        let expected_result = true;

        let result = ExecuteCompanyFactsRequest::try_from((state, client)).is_err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_named_after_notes_file_kind_when_prepare_notes_request_has_no_output()
     {
        let client = SecClient::default();
        let period = NotesPeriod::new("2024q1")
            .expect("Given a published period, NotesPeriod creation should always succeed");
        let state = PrepareSecRequest::builder()
            .notes_file()
            .period(period)
            .build();

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(PREPARE_SEC_REQUEST, EXECUTE_SEC_REQUEST),
        ));

        let result =
            ExecuteSecRequest::<NotesFile, SecClient>::try_from((state, client)).map(|_| ());

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

    #[tokio::test]
    async fn should_move_submissions_request_into_input_when_prepare_sec_request_is_of_submissions_kind()
     {
        let client = SecClient::default();
        let cik = Cik::new("0001067983")
            .expect("Hardcoded valid CIK string should always parse successfully");
        let input = PrepareSecRequestInput::new(cik.clone());
        let context = PrepareSecRequestContext::new(cik.clone());
        let mut state = PrepareSecRequest::<Submissions>::new(input, context);
        state
            .compute_output_data_async()
            .await
            .expect("Request preparation should always succeed for a valid CIK");

        let expected_result = ExecuteSecRequestInput::<Submissions, _>::new(
            client.clone(),
            SecRequest::builder().submissions().cik(cik).build(),
        );

        let result = ExecuteSecRequest::try_from((state, client))
            .expect("Given a computed state, the transition should always succeed")
            .input_data()
            .clone();

        assert_eq!(result, expected_result);
    }
}
