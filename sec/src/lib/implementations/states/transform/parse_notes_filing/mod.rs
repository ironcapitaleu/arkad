//! # Parse Notes Filing State
//!
//! Provides the [`ParseNotesFiling`] state, which parses the rows of one Notes filing into a
//! [`ParsedFiling`].
//!
//! The state calls the Notes parser of `xbrl` and counts the source rows of the filing. It fails
//! if the `sub` row of the filing breaks a rule of a core type.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//!
//! ## See Also
//!
//! - [`crate::shared::filing_input::NotesFilingInput`]: The input of this state.
//! - [`crate::shared::parsed_filing::ParsedFiling`]: The output of this state.
//! - [`crate::shared::filing_context::FilingContext`]: The context of this state.

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use serde::ser::{SerializeStruct, Serializer};
use state_maschine::prelude::State as SMState;
use xbrl::adapter::edgar::notes;

use crate::error::State as StateError;
use crate::error::state_machine::state::failed_notes_parse::FailedNotesParse;
use crate::shared::filing_context::FilingContext;
use crate::shared::filing_input::NotesFilingInput;
use crate::shared::parsed_filing::ParsedFiling;
use crate::shared::source_counts::SourceCounts;
use crate::traits::state_machine::state::State;

pub mod constants;

pub use constants::STATE_NAME;

/// Parses the rows of one Notes filing into a [`ParsedFiling`].
///
/// A successful output holds the facts, the filing, and the findings of the parser, and the
/// source rows of the filing with the rows the parser dropped.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParseNotesFiling {
    input: NotesFilingInput,
    context: FilingContext,
    output: Option<ParsedFiling>,
}

impl ParseNotesFiling {
    /// Creates a new [`ParseNotesFiling`] state from its input and context, with no output
    /// computed yet.
    #[must_use]
    pub const fn new(input: NotesFilingInput, context: FilingContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    #[must_use]
    pub fn into_parts(self) -> (NotesFilingInput, Option<ParsedFiling>, FilingContext) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl State for ParseNotesFiling {
    /// Parses the rows of the filing and stores the [`ParsedFiling`] as output.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::FailedNotesParse`] if the parser rejects the `sub` row.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let parsed = notes::parse(&self.input.rows, self.context.registry.registry())
            .map_err(|error| FailedNotesParse::new(STATE_NAME, error))?;
        let counts = SourceCounts::new(parsed.rows_in, parsed.duplicate_rows, &parsed.findings);
        self.output = Some(ParsedFiling {
            facts: parsed.facts,
            filing: parsed.filing,
            findings: parsed.findings,
            source: self.input.source.clone(),
            industry_source: None,
            counts,
        });
        Ok(())
    }
}

impl SMState for ParseNotesFiling {
    type InputData = NotesFilingInput;
    type OutputData = ParsedFiling;
    type Context = FilingContext;

    fn state_name(&self) -> impl ToString {
        STATE_NAME
    }

    /// Blocking wrapper around [`compute_output_data_async`](crate::traits::state_machine::state::State::compute_output_data_async).
    ///
    /// # Panics
    ///
    /// - Panics if the async computation returns an error.
    /// - Panics if called inside a `current_thread` runtime, since `block_in_place` needs a
    ///   multi-thread runtime.
    fn compute_output_data(&mut self) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(self.compute_output_data_async()))
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.compute_output_data_async())
        };

        if let Err(e) = result {
            panic!("compute_output_data failed: {e}")
        }
    }

    fn context_data(&self) -> &Self::Context {
        &self.context
    }

    fn input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl fmt::Display for ParseNotesFiling {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{STATE_NAME}` State Summary\n\
             ---------------------------\n\
             Input Data:\n\tFiling: {}\n\
             Output Data:\n\t{}",
            self.input.adsh(),
            self.output.as_ref().map_or_else(
                || "None".to_owned(),
                |output| format!("Facts: {}", output.facts.len())
            )
        )
    }
}

/// Writes the `adsh` and the counts of the output, not the rows, so that no row of a filing
/// enters event JSON (ADR 0048).
impl Serialize for ParseNotesFiling {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ParseNotesFiling", 3)?;
        state.serialize_field("adsh", self.input.adsh())?;
        state.serialize_field(
            "facts_count",
            &self.output.as_ref().map(|output| output.facts.len()),
        )?;
        state.serialize_field(
            "findings_count",
            &self.output.as_ref().map(|output| output.findings.len()),
        )?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use pretty_assertions::assert_eq;
    use xbrl::adapter::edgar::notes::NotesFiling;
    use xbrl::core::invalid_value::{InvalidValue, InvalidValueReason};
    use xbrl::error::parsing::ParseErrorKind;
    use xbrl::tests::fixtures::data::notes::notes_fixture_rows;

    use super::*;
    use crate::shared::source_counts::DropReason;
    use crate::tests::fixtures::sample_filing_context::fixture_context;
    use crate::tests::fixtures::sample_source_document::notes_source;

    fn state_of(rows: NotesFiling) -> ParseNotesFiling {
        let input = NotesFilingInput {
            rows,
            source: notes_source(),
        };
        ParseNotesFiling::new(input, fixture_context())
    }

    async fn output_of(rows: NotesFiling) -> ParsedFiling {
        let mut state = state_of(rows);
        state
            .compute_output_data_async()
            .await
            .expect("Given the valid sub row of a real filing, the parse should always succeed");
        state
            .into_parts()
            .1
            .expect("Given a computed state, the output should always be present")
    }

    #[test]
    fn should_return_state_name_when_created() {
        let state = state_of(notes_fixture_rows());

        let expected_result = "Parse Notes Filing";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_count_rows_in_and_dropped_rows_when_parsing_the_fixture_filing() {
        let rows = notes_fixture_rows();

        let expected_result = (678, BTreeMap::from([(DropReason::DuplicateRow, 65)]));

        let counts = output_of(rows).await.counts;
        let result = (counts.rows_in(), counts.dropped().clone());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_facts_of_the_parser_when_parsing_the_fixture_filing() {
        let rows = notes_fixture_rows();

        let expected_result = 613;

        let result = output_of(rows).await.facts.len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_set_the_notes_file_as_source_when_parsing_the_fixture_filing() {
        let rows = notes_fixture_rows();

        let expected_result = notes_source();

        let result = output_of(rows).await.source;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_notes_parse_when_the_cik_of_the_sub_row_is_not_a_number() {
        let mut rows = notes_fixture_rows();
        rows.sub.cik = "abc".to_owned();
        let mut state = state_of(rows);

        let expected_result = Err(StateError::FailedNotesParse(FailedNotesParse::new(
            "Parse Notes Filing",
            ParseErrorKind::InvalidSubmissionRow {
                adsh: "0001628280-26-068246".to_owned(),
                source: InvalidValue::new("SubRow", InvalidValueReason::NotANumber, "abc"),
            },
        )));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_serialize_counts_instead_of_rows_when_computed() {
        let mut state = state_of(notes_fixture_rows());
        state
            .compute_output_data_async()
            .await
            .expect("Given the valid sub row of a real filing, the parse should always succeed");

        let expected_result = serde_json::json!({
            "adsh": "0001628280-26-068246",
            "facts_count": 613,
            "findings_count": 0,
        });

        let result = serde_json::to_value(&state).expect(
            "Given a state of strings and numbers, the serialization should always succeed",
        );

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_parse_notes_filing() {
        implements_auto_traits::<ParseNotesFiling>();
    }

    const fn implements_state<T: State + Serialize>() {}
    #[test]
    const fn should_implement_state_and_serialize_for_parse_notes_filing() {
        implements_state::<ParseNotesFiling>();
    }
}
