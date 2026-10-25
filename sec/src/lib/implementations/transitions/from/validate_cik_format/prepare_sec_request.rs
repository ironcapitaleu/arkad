//! # Transition: `ValidateCikFormat` → `PrepareSecRequest`
//!
//! Implements the [`TryFrom`] conversion producing a [`PrepareSecRequest`] from a
//! [`ValidateCikFormat`].
//!
//! It takes the validated [`Cik`](crate::shared::cik::Cik) from the source state's output and the
//! shared client from its context, and seeds the successor's input and context with them. The
//! conversion is fallible because the source state may not have computed its output yet.
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

impl TryFrom<ValidateCikFormat> for PrepareSecRequest {
    type Error = TransitionError;

    fn try_from(state: ValidateCikFormat) -> Result<Self, TransitionError> {
        let (_input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(VALIDATE_CIK_FORMAT, PREPARE_SEC_REQUEST)
        })?;

        let new_context =
            PrepareSecRequestContext::new(output_data.validated_cik.clone(), context.user_agent);
        let new_input = PrepareSecRequestInput::new(output_data.validated_cik, context.sec_client);

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::State as SMState;

    use super::*;
    use crate::implementations::states::extract::validate_cik_format::{
        ValidateCikFormat, ValidateCikFormatContext, ValidateCikFormatInput,
    };
    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_run::sample_user_agent;
    use crate::traits::state_machine::state::State;

    #[tokio::test]
    async fn should_transition_to_prepare_sec_request_when_validate_cik_format_has_output() {
        let cik_string = "0001234567";
        let sec_client = SecClient::default();
        let input = ValidateCikFormatInput {
            raw_cik: cik_string.into(),
        };
        let context =
            ValidateCikFormatContext::new(cik_string, sec_client.clone(), sample_user_agent());
        let mut state = ValidateCikFormat::new(input, context);
        state
            .compute_output_data_async()
            .await
            .expect("Hardcoded valid CIK should always compute successfully");

        let expected_cik = Cik::new(cik_string)
            .expect("Hardcoded valid CIK string should always parse successfully");
        let expected_context =
            PrepareSecRequestContext::new(expected_cik.clone(), sample_user_agent());
        let expected_input = PrepareSecRequestInput::new(expected_cik, sec_client);
        let expected_result = PrepareSecRequest::new(expected_input, expected_context);

        let result = PrepareSecRequest::try_from(state)
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
            sample_user_agent(),
        );
        let state = ValidateCikFormat::new(input, context);

        let expected_result = true;

        let result = PrepareSecRequest::try_from(state).is_err();

        assert_eq!(result, expected_result);
    }

    async fn computed_validate_cik_format(user_agent: UserAgent) -> ValidateCikFormat {
        let input = ValidateCikFormatInput::new(BERKSHIRE_HATHAWAY_CIK_RAW);
        let context = ValidateCikFormatContext::new(
            BERKSHIRE_HATHAWAY_CIK_RAW,
            SecClient::default(),
            user_agent,
        );
        let mut state = ValidateCikFormat::new(input, context);
        state
            .compute_output_data_async()
            .await
            .expect("Given a hardcoded valid CIK, validation should always succeed");
        state
    }

    #[tokio::test]
    async fn should_carry_user_agent_when_validate_cik_format_has_output() {
        let other_user_agent = UserAgent::new("Other Company other@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        let state = computed_validate_cik_format(other_user_agent.clone()).await;

        let expected_result = other_user_agent;

        let result = PrepareSecRequest::try_from(state)
            .expect("Given a state with output, the transition should always succeed")
            .context_data()
            .user_agent()
            .clone();

        assert_eq!(result, expected_result);
    }
}
