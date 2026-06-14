//! # Transition: `ValidateCikFormat` → `PrepareSecRequest`
//!
//! Implements `TryFrom<`[`ValidateCikFormat`]`>` for [`PrepareSecRequest`], the first extract-phase
//! transition.
//!
//! It takes the validated [`Cik`](crate::shared::cik::Cik) from the source state's output and the
//! shared client from its context, and seeds the next state's input and context with them. The
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

        let new_context = PrepareSecRequestContext::new(output_data.validated_cik.clone());
        let new_input = PrepareSecRequestInput::new(output_data.validated_cik, context.sec_client);

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::implementations::states::extract::validate_cik_format::{
        ValidateCikFormat, ValidateCikFormatContext, ValidateCikFormatInput,
    };
    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::traits::state_machine::state::State;

    #[tokio::test]
    async fn should_transition_to_prepare_sec_request_when_validate_cik_format_has_output() {
        let cik_string = "0001234567";
        let sec_client = SecClient::default();
        let input = ValidateCikFormatInput {
            raw_cik: cik_string.into(),
        };
        let context = ValidateCikFormatContext::new(cik_string, sec_client.clone());
        let mut state = ValidateCikFormat::new(input, context);
        state
            .compute_output_data_async()
            .await
            .expect("Hardcoded valid CIK should always compute successfully");

        let expected_cik = Cik::new(cik_string)
            .expect("Hardcoded valid CIK string should always parse successfully");
        let expected_context = PrepareSecRequestContext::new(expected_cik.clone());
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
        let context = ValidateCikFormatContext::new(BERKSHIRE_HATHAWAY_CIK_RAW, sec_client);
        let state = ValidateCikFormat::new(input, context);

        let expected_result = true;

        let result = PrepareSecRequest::try_from(state).is_err();

        assert_eq!(result, expected_result);
    }
}
