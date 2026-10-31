//! # Transition: `ValidateIndexDate` → `PrepareSecRequest<DailyIndex>`
//!
//! Implements the [`TryFrom`] conversion producing a [`PrepareSecRequest`] of the [`DailyIndex`]
//! kind from a [`ValidateIndexDate`].
//!
//! It moves the validated [`IndexDate`](crate::shared::index_date::IndexDate) from the output of
//! the source state into the input and the context of the next state.
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use crate::implementations::states::extract::validate_index_date::ValidateIndexDate;
use crate::implementations::states::extract::validate_index_date::constants::STATE_NAME as VALIDATE_INDEX_DATE;
use crate::shared::request::implementations::sec_request::DailyIndex;

impl TryFrom<ValidateIndexDate> for PrepareSecRequest<DailyIndex> {
    type Error = TransitionError;

    /// Builds a [`PrepareSecRequest`] of the [`DailyIndex`] kind from a computed
    /// [`ValidateIndexDate`].
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    fn try_from(state: ValidateIndexDate) -> Result<Self, TransitionError> {
        let (_input, output, _context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(VALIDATE_INDEX_DATE, PREPARE_SEC_REQUEST)
        })?;

        Ok(Self::new(
            PrepareSecRequestInput::new(output_data.index_date),
            PrepareSecRequestContext::new(output_data.index_date),
        ))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::implementations::states::extract::validate_index_date::{
        ValidateIndexDateContext, ValidateIndexDateInput,
    };
    use crate::shared::index_date::IndexDate;
    use crate::traits::state_machine::state::State;

    fn validate_index_date(raw_date: &str) -> ValidateIndexDate {
        ValidateIndexDate::new(
            ValidateIndexDateInput::new(raw_date),
            ValidateIndexDateContext::new(Uuid::nil()),
        )
    }

    #[tokio::test]
    async fn should_move_index_date_when_source_has_output() {
        let mut state = validate_index_date("2026-10-27");
        state.compute_output_data_async().await.expect(
            "Given the hardcoded past weekday '2026-10-27', validation should always succeed",
        );
        let index_date = IndexDate::new("2026-10-27")
            .expect("Given a hardcoded past date, validation should always succeed");

        let expected_result = Ok(PrepareSecRequest::new(
            PrepareSecRequestInput::new(index_date),
            PrepareSecRequestContext::new(index_date),
        ));

        let result = PrepareSecRequest::<DailyIndex>::try_from(state);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = validate_index_date("2026-10-27");

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(VALIDATE_INDEX_DATE, PREPARE_SEC_REQUEST),
        ));

        let result = PrepareSecRequest::<DailyIndex>::try_from(state);

        assert_eq!(result, expected_result);
    }
}
