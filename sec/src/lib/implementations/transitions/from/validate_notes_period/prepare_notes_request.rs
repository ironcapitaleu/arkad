//! # Transition: `ValidateNotesPeriod` → `PrepareNotesRequest`
//!
//! Implements the [`TryFrom`] conversion producing a [`PrepareNotesRequest`] from a
//! [`ValidateNotesPeriod`].
//!
//! It builds the state with the [`PrepareSecRequest`] builder from the validated
//! [`NotesPeriod`](crate::shared::notes_period::NotesPeriod) in the output of the source state. The
//! `execution_id` and the storage repository stay in the state machine context.
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareNotesRequest, PrepareSecRequest,
};
use crate::implementations::states::extract::validate_notes_period::ValidateNotesPeriod;
use crate::implementations::states::extract::validate_notes_period::constants::STATE_NAME as VALIDATE_NOTES_PERIOD;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawNotesArchive;

impl<R: RawRepository<RawNotesArchive>> TryFrom<ValidateNotesPeriod<R>> for PrepareNotesRequest {
    type Error = TransitionError;

    /// Builds a [`PrepareNotesRequest`] from a computed [`ValidateNotesPeriod`].
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    fn try_from(state: ValidateNotesPeriod<R>) -> Result<Self, TransitionError> {
        let (_input, output, _context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(VALIDATE_NOTES_PERIOD, PREPARE_SEC_REQUEST)
        })?;

        Ok(PrepareSecRequest::builder()
            .notes_file()
            .period(output_data.notes_period)
            .build())
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
    async fn should_move_period_when_source_has_output() {
        let mut state = validate_notes_period();
        state.compute_output_data_async().await.expect(
            "Given the hardcoded published quarter '2024q1', validation should always succeed",
        );

        let period = NotesPeriod::new("2024q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed");
        let expected_result = Ok(PrepareSecRequest::builder()
            .notes_file()
            .period(period)
            .build());

        let result = PrepareNotesRequest::try_from(state);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = validate_notes_period();

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(VALIDATE_NOTES_PERIOD, "Prepare SEC Request"),
        ));

        let result = PrepareNotesRequest::try_from(state);

        assert_eq!(result, expected_result);
    }
}
