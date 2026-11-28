//! # Transition: `ParseInstanceFiling` → `StandardizeFiling`
//!
//! Implements the [`TryFrom`] conversion that produces a [`StandardizeFiling`] from a
//! [`ParseInstanceFiling`].
//!
//! It moves the [`ParsedFiling`](crate::shared::parsed_filing::ParsedFiling) of the source state
//! into the input of the next state, and keeps the
//! [`FilingContext`](crate::shared::filing_context::FilingContext). The conversion fails if the
//! source state has no output.
//!
//! # Errors
//!
//! Returns [`TransitionError::MissingOutput`] when the source state has no computed output.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::transform::parse_instance_filing::ParseInstanceFiling;
use crate::implementations::states::transform::parse_instance_filing::constants::STATE_NAME as PARSE_INSTANCE_FILING;
use crate::implementations::states::transform::standardize_filing::StandardizeFiling;
use crate::implementations::states::transform::standardize_filing::constants::STATE_NAME as STANDARDIZE_FILING;

impl TryFrom<ParseInstanceFiling> for StandardizeFiling {
    type Error = TransitionError;

    fn try_from(state: ParseInstanceFiling) -> Result<Self, TransitionError> {
        let (_input, output, context) = state.into_parts();
        let parsed = output.ok_or_else(|| {
            transition::MissingOutput::new(PARSE_INSTANCE_FILING, STANDARDIZE_FILING)
        })?;
        Ok(Self::new(parsed, context))
    }
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::State as SMState;
    use xbrl::tests::fixtures::data::instance::INSTANCE_DOCUMENT;

    use super::*;
    use crate::error::state_machine::transition::MissingOutput;
    use crate::shared::accession_number::AccessionNumber;
    use crate::shared::cik::Cik;
    use crate::shared::filing_input::InstanceFilingInput;
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::form::{Form, ParsedForm};
    use crate::tests::fixtures::sample_filing_context::us_gaap_context;
    use crate::tests::fixtures::sample_source_document::instance_source;
    use crate::traits::state_machine::state::State;

    fn parse_state() -> ParseInstanceFiling {
        let input = InstanceFilingInput {
            filing: FilingRef::new(
                Cik::new("1048911")
                    .expect("Given a hardcoded valid CIK, parsing should always succeed"),
                AccessionNumber::new("0001628280-26-068246").expect(
                    "Given a hardcoded accession number of the right shape, creation should always succeed",
                ),
                ParsedForm::new(Form::TenQ, false),
                date(2026, 10, 29),
            ),
            document: INSTANCE_DOCUMENT.as_bytes().to_vec(),
            source: instance_source(),
            industry: None,
        };
        ParseInstanceFiling::new(input, us_gaap_context())
    }

    async fn computed_parse_state() -> ParseInstanceFiling {
        let mut state = parse_state();
        state
            .compute_output_data_async()
            .await
            .expect("Given the instance of a real filing, the parse should always succeed");
        state
    }

    #[test]
    fn should_return_missing_output_when_parse_instance_filing_has_no_output() {
        let state = parse_state();

        let expected_result = Some(TransitionError::MissingOutput(MissingOutput::new(
            "Parse Instance Filing",
            "Standardize Filing",
        )));

        let result = StandardizeFiling::try_from(state).err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_move_the_parsed_filing_into_the_input_when_parse_instance_filing_has_output() {
        let state = computed_parse_state().await;

        let expected_result = 613;

        let next = StandardizeFiling::try_from(state)
            .expect("Given a computed parse state, the transition should always succeed");
        let result = next.input_data().facts.len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_give_candidates_when_the_fixture_filing_runs_from_parse_instance_filing() {
        let state = computed_parse_state().await;
        let mut next = StandardizeFiling::try_from(state)
            .expect("Given a computed parse state, the transition should always succeed");

        let expected_result = 88;

        next.compute_output_data_async()
            .await
            .expect("Given any as-is facts, the standardization should always succeed");
        let result = next
            .output_data()
            .expect("Given a computed state, the output should always be present")
            .candidates
            .len();

        assert_eq!(result, expected_result);
    }
}
