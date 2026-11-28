//! # Filing Machine
//!
//! Provides [`run_filing`], which runs the states of the filing machine on one filing, and its
//! result, [`FilingOutcome`].
//!
//! The machine runs a parse state for the source of the input, then `StandardizeFiling`, then
//! `BuildFilingRows`. It moves from one state to the next by the `TryFrom` transitions, with no
//! event stream, so no state data enters event JSON. Every error of a state or a transition
//! rejects the filing: the machine does no I/O, so a second try gives the same error.
//!
//! ## See Also
//!
//! - [`crate::shared::filing_input::FilingInput`]: The input of the machine.
//! - [`crate::shared::filing_rows::FilingRows`]: The rows of a written filing.

use state_maschine::prelude::State as SMState;

use crate::error::StateMachine as StateMachineError;
use crate::implementations::states::transform::build_filing_rows::BuildFilingRows;
use crate::implementations::states::transform::parse_instance_filing::ParseInstanceFiling;
use crate::implementations::states::transform::parse_notes_filing::ParseNotesFiling;
use crate::implementations::states::transform::standardize_filing::StandardizeFiling;
use crate::shared::filing_context::FilingContext;
use crate::shared::filing_input::FilingInput;
use crate::shared::filing_rows::FilingRows;
use crate::shared::source_counts::SourceCounts;
use crate::traits::state_machine::state::State;

/// The result of the filing machine for one filing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilingOutcome {
    /// Every state succeeded. The rows go to the writer. They are boxed, because they are larger
    /// than a rejected filing.
    Written(Box<FilingRows>),
    /// A state or a transition failed. The filing writes no row.
    Rejected(RejectedFiling),
}

/// A filing that the filing machine rejected, with the error and the counts of its source rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedFiling {
    /// The accession number of the filing, for example `0000320193-26-000001`.
    pub filing: String,
    /// The error of the state or the transition that failed.
    pub error: StateMachineError,
    /// Every source row of the filing, dropped under `RejectedFiling`.
    pub counts: SourceCounts,
}

/// Runs the filing machine on one filing.
///
/// No state waits on I/O, so the future completes in one run of a current-thread executor. The
/// worker of the task runner owns that executor.
///
/// The counts of a rejected filing hold the source rows that the machine knows when it fails: the
/// `num` and `txt` rows of a Notes filing, the fact elements of an instance whose parse succeeded,
/// and none for an instance whose parse failed.
pub async fn run_filing(input: FilingInput, context: &FilingContext) -> FilingOutcome {
    let filing = input.accession_number().to_owned();
    let reject = |error, rows_in| {
        FilingOutcome::Rejected(RejectedFiling {
            filing,
            error,
            counts: SourceCounts::rejected(rows_in),
        })
    };
    let rows_before_parse = match &input {
        FilingInput::Notes(notes) => (notes.rows.num.len() + notes.rows.txt.len()) as u64,
        FilingInput::Instance(_) => 0,
    };
    let standardize = match parse(input, *context).await {
        Ok(state) => state,
        Err(error) => return reject(error, rows_before_parse),
    };
    let rows_in = standardize.input_data().counts.rows_in();
    match standardize_and_build(standardize).await {
        Ok(rows) => FilingOutcome::Written(Box::new(rows)),
        Err(error) => reject(error, rows_in),
    }
}

/// Runs the parse state of the source and moves its output into `StandardizeFiling`.
///
/// # Errors
///
/// Returns the [`StateMachineError`] of the parse state or of the transition.
async fn parse(
    input: FilingInput,
    context: FilingContext,
) -> Result<StandardizeFiling, StateMachineError> {
    Ok(match input {
        FilingInput::Notes(notes) => {
            let mut parse = ParseNotesFiling::new(notes, context);
            parse.compute_output_data_async().await?;
            StandardizeFiling::try_from(parse)?
        }
        FilingInput::Instance(instance) => {
            let mut parse = ParseInstanceFiling::new(instance, context);
            parse.compute_output_data_async().await?;
            StandardizeFiling::try_from(parse)?
        }
    })
}

/// Runs `StandardizeFiling` and `BuildFilingRows`, and returns the rows of the filing.
///
/// # Errors
///
/// Returns the [`StateMachineError`] of the first state or transition that fails.
async fn standardize_and_build(
    mut standardize: StandardizeFiling,
) -> Result<FilingRows, StateMachineError> {
    standardize.compute_output_data_async().await?;
    let mut build = BuildFilingRows::try_from(standardize)?;
    build.compute_output_data_async().await?;
    let (_input, output, _context) = build.into_parts();
    Ok(output.expect("Given a computed state, the output should always be present"))
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use jiff::civil::date;
    use pretty_assertions::assert_eq;
    use xbrl::adapter::edgar::notes::NotesFiling;
    use xbrl::tests::fixtures::data::instance::INSTANCE_DOCUMENT;
    use xbrl::tests::fixtures::data::notes::notes_fixture_rows;

    use super::*;
    use crate::shared::accession_number::AccessionNumber;
    use crate::shared::cik::Cik;
    use crate::shared::filing_input::{InstanceFilingInput, NotesFilingInput};
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::form::{Form, ParsedForm};
    use crate::tests::fixtures::sample_filing_context::us_gaap_context;
    use crate::tests::fixtures::sample_source_document::{instance_source, notes_source};

    fn notes_input(rows: NotesFiling) -> FilingInput {
        FilingInput::Notes(NotesFilingInput {
            rows,
            source: notes_source(),
        })
    }

    fn instance_input(document: &[u8]) -> FilingInput {
        FilingInput::Instance(InstanceFilingInput {
            filing: FilingRef::new(
                Cik::new("1048911")
                    .expect("Given a hardcoded valid CIK, parsing should always succeed"),
                AccessionNumber::new("0001628280-26-068246").expect(
                    "Given a hardcoded accession number of the right shape, creation should always succeed",
                ),
                ParsedForm::new(Form::TenQ, false),
                date(2026, 10, 29),
            ),
            document: document.to_vec(),
            source: instance_source(),
            industry: None,
        })
    }

    /// A Notes filing whose `sub` row breaks a rule: its CIK is not a number. It keeps the 678
    /// `num` and `txt` rows of the fixture.
    fn broken_sub_rows() -> NotesFiling {
        let mut rows = notes_fixture_rows();
        rows.sub.cik = "abc".to_owned();
        rows
    }

    /// Returns the as-is fact rows of a written filing, or `None` for a rejected filing.
    fn written_facts(outcome: &FilingOutcome) -> Option<usize> {
        match outcome {
            FilingOutcome::Written(rows) => Some(rows.as_is_facts.len()),
            FilingOutcome::Rejected(_) => None,
        }
    }

    /// Returns the error text and the counts of a rejected filing, or `None` for a written filing.
    fn rejection(outcome: FilingOutcome) -> Option<(String, String, SourceCounts)> {
        match outcome {
            FilingOutcome::Written(_) => None,
            FilingOutcome::Rejected(rejected) => {
                Some((rejected.filing, rejected.error.to_string(), rejected.counts))
            }
        }
    }

    #[tokio::test]
    async fn should_write_every_as_is_fact_for_the_notes_fixture_filing() {
        let input = notes_input(notes_fixture_rows());

        let expected_result = Some(613);

        let result = written_facts(&run_filing(input, &us_gaap_context()).await);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_every_as_is_fact_for_the_instance_fixture_filing() {
        let input = instance_input(INSTANCE_DOCUMENT.as_bytes());

        let expected_result = Some(613);

        let result = written_facts(&run_filing(input, &us_gaap_context()).await);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_reject_the_filing_with_every_source_row_when_the_sub_row_is_broken() {
        let input = notes_input(broken_sub_rows());

        let expected_result = Some((
            "0001628280-26-068246".to_owned(),
            "[FailedNotesParse] Failed to parse the Notes filing, Caused by: [InvalidSubmissionRow] Invalid sub row of the Notes filing '0001628280-26-068246', Caused by: [InvalidValue] Invalid 'SubRow', Reason: 'The value is not a number', Input: 'abc'".to_owned(),
            SourceCounts::rejected(678),
        ));

        let result = rejection(run_filing(input, &us_gaap_context()).await);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_reject_the_filing_with_no_source_row_when_the_instance_is_not_xml() {
        let input = instance_input(b"<xbrl>");

        let expected_result = Some((
            "0001628280-26-068246".to_owned(),
            "[FailedInstanceParse] Failed to parse the instance filing, Caused by: [InvalidXml] Failed to parse the instance document of the filing 'sec:0001628280-26-068246', Reason: 'the root node was opened but never closed'".to_owned(),
            SourceCounts::rejected(0),
        ));

        let result = rejection(run_filing(input, &us_gaap_context()).await);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_reject_every_fact_element_when_the_row_build_of_an_instance_fails() {
        let document = INSTANCE_DOCUMENT.replace(
            r#"id="f-31" unitRef="usd">388000000<"#,
            r#"id="f-31" unitRef="usd">1000000000000000000000000000000<"#,
        );
        let input = instance_input(document.as_bytes());

        let expected_result = Some((
            "0001628280-26-068246".to_owned(),
            "[FailedRowBuild] Failed to build the rows of the filing 'sec:0001628280-26-068246', Caused by: [UnexpectedColumnValue] Row '30' of column 'value_numeric' in dataset 'as_is_facts' holds an unexpected value, Reason: ''1000000000000000000000000000000' has more than 38 digits'".to_owned(),
            SourceCounts::rejected(678),
        ));

        let result = rejection(run_filing(input, &us_gaap_context()).await);

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filing_outcome() {
        implements_auto_traits::<FilingOutcome>();
    }

    const fn implements_value_traits<T: Debug + Clone + PartialEq + Eq>() {}
    #[test]
    const fn should_implement_value_traits_for_filing_outcome() {
        implements_value_traits::<FilingOutcome>();
    }
}
