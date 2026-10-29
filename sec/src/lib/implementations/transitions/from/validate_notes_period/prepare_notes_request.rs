//! # Transition: `ValidateNotesPeriod` → `PrepareNotesRequest`
//!
//! Implements the [`TryFrom`] conversion producing a [`PrepareNotesRequest`] from a
//! [`ValidateNotesPeriod`].
//!
//! It moves the validated [`NotesPeriod`](crate::shared::notes_period::NotesPeriod) from the output
//! of the source state, and the `execution_id` and the storage repository from its context.
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::prepare_notes_request::constants::STATE_NAME as PREPARE_NOTES_REQUEST;
use crate::implementations::states::extract::prepare_notes_request::{
    PrepareNotesRequest, PrepareNotesRequestContext, PrepareNotesRequestInput,
};
use crate::implementations::states::extract::validate_notes_period::ValidateNotesPeriod;
use crate::implementations::states::extract::validate_notes_period::constants::STATE_NAME as VALIDATE_NOTES_PERIOD;

impl<R> TryFrom<ValidateNotesPeriod<R>> for PrepareNotesRequest<R> {
    type Error = TransitionError;

    /// Builds a [`PrepareNotesRequest`] from a computed [`ValidateNotesPeriod`].
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    fn try_from(state: ValidateNotesPeriod<R>) -> Result<Self, TransitionError> {
        let (_input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(VALIDATE_NOTES_PERIOD, PREPARE_NOTES_REQUEST)
        })?;

        let new_input = PrepareNotesRequestInput::new(output_data.notes_period);
        let new_context = PrepareNotesRequestContext::new(context.execution_id, context.repository);

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use storage::WriteError;
    use uuid::Uuid;

    use super::*;
    use crate::implementations::states::extract::validate_notes_period::{
        ValidateNotesPeriodContext, ValidateNotesPeriodInput,
    };
    use crate::shared::notes_period::NotesPeriod;
    use crate::tests::fixtures::sample_write_repository::FakeNotesRepository;
    use crate::traits::state_machine::state::State;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);

    fn rejecting_repository() -> FakeNotesRepository {
        FakeNotesRepository::rejecting(WriteError::conflicting_write("Sample conflict"))
    }

    fn validate_notes_period() -> ValidateNotesPeriod<FakeNotesRepository> {
        ValidateNotesPeriod::new(
            ValidateNotesPeriodInput::new("2024q1"),
            ValidateNotesPeriodContext::new(SAMPLE_EXECUTION_ID, rejecting_repository()),
        )
    }

    #[tokio::test]
    async fn should_move_period_execution_id_and_repository_when_source_has_output() {
        let mut state = validate_notes_period();
        state.compute_output_data_async().await.expect(
            "Given the hardcoded published quarter '2024q1', validation should always succeed",
        );

        let period = NotesPeriod::new("2024q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed");
        let expected_result = Ok(PrepareNotesRequest::new(
            PrepareNotesRequestInput::new(period),
            PrepareNotesRequestContext::new(SAMPLE_EXECUTION_ID, rejecting_repository()),
        ));

        let result = PrepareNotesRequest::try_from(state);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = validate_notes_period();

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(VALIDATE_NOTES_PERIOD, PREPARE_NOTES_REQUEST),
        ));

        let result = PrepareNotesRequest::try_from(state);

        assert_eq!(result, expected_result);
    }
}
