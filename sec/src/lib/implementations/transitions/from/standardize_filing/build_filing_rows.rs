//! # Transition: `StandardizeFiling` → `BuildFilingRows`
//!
//! Implements the [`TryFrom`] conversion that produces a [`BuildFilingRows`] from a
//! [`StandardizeFiling`].
//!
//! It moves the [`StandardizedFiling`](crate::shared::standardized_filing::StandardizedFiling) of
//! the source state into the input of the next state, and keeps the
//! [`FilingContext`](crate::shared::filing_context::FilingContext). The conversion fails if the
//! source state has no output.
//!
//! # Errors
//!
//! Returns [`TransitionError::MissingOutput`] when the source state has no computed output.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::transform::build_filing_rows::BuildFilingRows;
use crate::implementations::states::transform::build_filing_rows::constants::STATE_NAME as BUILD_FILING_ROWS;
use crate::implementations::states::transform::standardize_filing::StandardizeFiling;
use crate::implementations::states::transform::standardize_filing::constants::STATE_NAME as STANDARDIZE_FILING;

impl TryFrom<StandardizeFiling> for BuildFilingRows {
    type Error = TransitionError;

    fn try_from(state: StandardizeFiling) -> Result<Self, TransitionError> {
        let (_input, output, context) = state.into_parts();
        let standardized = output
            .ok_or_else(|| transition::MissingOutput::new(STANDARDIZE_FILING, BUILD_FILING_ROWS))?;
        Ok(Self::new(standardized, context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::State as SMState;
    use xbrl::tests::fixtures::data::notes::notes_fixture_rows;

    use super::*;
    use crate::error::state_machine::transition::MissingOutput;
    use crate::implementations::states::transform::parse_notes_filing::ParseNotesFiling;
    use crate::shared::filing_input::NotesFilingInput;
    use crate::tests::fixtures::sample_filing_context::us_gaap_context;
    use crate::tests::fixtures::sample_source_document::notes_source;
    use crate::traits::state_machine::state::State;

    async fn standardize_state() -> StandardizeFiling {
        let input = NotesFilingInput {
            rows: notes_fixture_rows(),
            source: notes_source(),
        };
        let mut parse = ParseNotesFiling::new(input, us_gaap_context());
        parse
            .compute_output_data_async()
            .await
            .expect("Given the valid sub row of a real filing, the parse should always succeed");
        StandardizeFiling::try_from(parse)
            .expect("Given a computed parse state, the transition should always succeed")
    }

    #[tokio::test]
    async fn should_return_missing_output_when_standardize_filing_has_no_output() {
        let state = standardize_state().await;

        let expected_result = Some(TransitionError::MissingOutput(MissingOutput::new(
            "Standardize Filing",
            "Build Filing Rows",
        )));

        let result = BuildFilingRows::try_from(state).err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_move_the_candidates_into_the_input_when_standardize_filing_has_output() {
        let mut state = standardize_state().await;
        state
            .compute_output_data_async()
            .await
            .expect("Given any as-is facts, the standardization should always succeed");

        let expected_result = 88;

        let next = BuildFilingRows::try_from(state)
            .expect("Given a computed standardize state, the transition should always succeed");
        let result = next.input_data().candidates.len();

        assert_eq!(result, expected_result);
    }
}
