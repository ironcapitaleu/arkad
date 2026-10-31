//! # Transition: `ValidateCikFormat` → `PrepareSecRequest`
//!
//! Implements the [`TryFrom`] conversion producing a [`PrepareSecRequest`] from a
//! [`ValidateCikFormat`], for every request kind `K` whose input is a [`Cik`]. These are
//! [`CompanyFacts`](crate::shared::request::implementations::sec_request::CompanyFacts) and
//! [`Submissions`](crate::shared::request::implementations::sec_request::Submissions).
//!
//! It takes the validated [`Cik`] from the source state's output and the repository from its
//! context, and seeds the successor's input and context with them. The conversion is fallible
//! because the source state may not have computed its output yet.
//!
//! A kind with another input cannot follow [`ValidateCikFormat`], so this does not compile:
//!
//! ```compile_fail,E0277
//! use sec::implementations::states::extract::prepare_sec_request::PrepareNotesRequest;
//! use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
//!
//! fn prepare_as_notes_file<R>(state: ValidateCikFormat<R>) {
//!     let _next = PrepareNotesRequest::<R>::try_from(state);
//! }
//! ```
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output to carry forward.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use crate::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use crate::implementations::states::extract::validate_cik_format::constants::STATE_NAME as VALIDATE_CIK_FORMAT;
use crate::shared::cik::Cik;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::request::implementations::sec_request::RequestKind;

impl<K, R, C> TryFrom<ValidateCikFormat<R, C>> for PrepareSecRequest<K, R>
where
    K: RequestKind<Input = Cik>,
    C: SecClientTrait,
{
    type Error = TransitionError;

    fn try_from(state: ValidateCikFormat<R, C>) -> Result<Self, TransitionError> {
        let (_input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(VALIDATE_CIK_FORMAT, PREPARE_SEC_REQUEST)
        })?;

        let new_context =
            PrepareSecRequestContext::new(output_data.validated_cik.clone(), context.repository);
        let new_input = PrepareSecRequestInput::new(output_data.validated_cik);

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::State as SMState;
    use storage::WriteError;

    use super::*;
    use crate::implementations::states::extract::prepare_sec_request::PrepareCompanyFactsRequest;
    use crate::implementations::states::extract::validate_cik_format::{
        ValidateCikFormat, ValidateCikFormatContext, ValidateCikFormatInput,
    };
    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::request::implementations::sec_request::Submissions;
    use crate::tests::fixtures::sample_write_repository::{
        FakeSubmissionsRepository, FakeWriteRepository,
    };
    use crate::traits::state_machine::state::State;

    #[tokio::test]
    async fn should_transition_to_prepare_sec_request_when_validate_cik_format_has_output() {
        let cik_string = "0001234567";
        let sec_client = SecClient::default();
        let input = ValidateCikFormatInput {
            raw_cik: cik_string.into(),
        };
        let context =
            ValidateCikFormatContext::new(cik_string, sec_client, FakeWriteRepository::new());
        let mut state = ValidateCikFormat::new(input, context);
        state
            .compute_output_data_async()
            .await
            .expect("Hardcoded valid CIK should always compute successfully");

        let expected_cik = Cik::new(cik_string)
            .expect("Hardcoded valid CIK string should always parse successfully");
        let expected_context =
            PrepareSecRequestContext::new(expected_cik.clone(), FakeWriteRepository::new());
        let expected_input = PrepareSecRequestInput::new(expected_cik);
        let expected_result = PrepareCompanyFactsRequest::new(expected_input, expected_context);

        let result = PrepareCompanyFactsRequest::try_from(state)
            .expect("State with computed output should always transition successfully");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_validate_cik_format_has_no_output() {
        let sec_client = SecClient::default();
        let input = ValidateCikFormatInput::new(BERKSHIRE_HATHAWAY_CIK_RAW);
        let context = ValidateCikFormatContext::new(
            BERKSHIRE_HATHAWAY_CIK_RAW,
            sec_client,
            FakeWriteRepository::new(),
        );
        let state = ValidateCikFormat::new(input, context);

        let expected_result = true;

        let result = PrepareCompanyFactsRequest::try_from(state).is_err();

        assert_eq!(result, expected_result);
    }

    async fn computed_validate_cik_format(
        repository: FakeWriteRepository,
    ) -> ValidateCikFormat<FakeWriteRepository> {
        let input = ValidateCikFormatInput::new(BERKSHIRE_HATHAWAY_CIK_RAW);
        let context = ValidateCikFormatContext::new(
            BERKSHIRE_HATHAWAY_CIK_RAW,
            SecClient::default(),
            repository,
        );
        let mut state = ValidateCikFormat::new(input, context);
        state
            .compute_output_data_async()
            .await
            .expect("Given a hardcoded valid CIK, validation should always succeed");
        state
    }

    #[tokio::test]
    async fn should_carry_repository_when_validate_cik_format_has_output() {
        let state = computed_validate_cik_format(FakeWriteRepository::rejecting(
            WriteError::conflicting_write("Sample conflict"),
        ))
        .await;

        let expected_result =
            FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict"));

        let result = PrepareCompanyFactsRequest::try_from(state)
            .expect("Given a state with output, the transition should always succeed")
            .context_data()
            .repository()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_transition_to_prepare_sec_request_of_submissions_kind_when_validate_cik_format_has_output()
     {
        let repository =
            FakeSubmissionsRepository::rejecting(WriteError::conflicting_write("Sample conflict"));
        let input = ValidateCikFormatInput::new(BERKSHIRE_HATHAWAY_CIK_RAW);
        let context = ValidateCikFormatContext::new(
            BERKSHIRE_HATHAWAY_CIK_RAW,
            SecClient::default(),
            repository.clone(),
        );
        let mut state = ValidateCikFormat::new(input, context);
        state
            .compute_output_data_async()
            .await
            .expect("Given a hardcoded valid CIK, validation should always succeed");

        let expected_cik = Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW)
            .expect("Given a hardcoded valid CIK, parsing should always succeed");
        let expected_result = PrepareSecRequest::<Submissions, _>::new(
            PrepareSecRequestInput::new(expected_cik.clone()),
            PrepareSecRequestContext::new(expected_cik, repository),
        );

        let result = PrepareSecRequest::<Submissions, _>::try_from(state)
            .expect("Given a state with output, the transition should always succeed");

        assert_eq!(result, expected_result);
    }
}
