//! # Parse Instance Filing State
//!
//! Provides the [`ParseInstanceFiling`] state, which parses the main instance of one filing into a
//! [`ParsedFiling`].
//!
//! The state decodes the stored bytes as UTF-8, calls the instance parser of `xbrl`, and counts
//! the fact elements of the instance. It sets the industry of the company from the stored
//! `submissions` file, since the instance has no industry code. It fails if the bytes are not
//! UTF-8, if the text is not XML, or if a cover fact is missing or breaks a rule.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//!
//! ## See Also
//!
//! - [`crate::shared::filing_input::InstanceFilingInput`]: The input of this state.
//! - [`crate::shared::parsed_filing::ParsedFiling`]: The output of this state.
//! - [`crate::shared::filing_context::FilingContext`]: The context of this state.

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use serde::ser::{SerializeStruct, Serializer};
use state_maschine::prelude::State as SMState;
use xbrl::adapter::edgar::company_facts::SEC_FILING_SOURCE;
use xbrl::adapter::edgar::instance::{self, InstanceFiling};
use xbrl::core::filing::FilingId;

use crate::error::State as StateError;
use crate::error::state_machine::state::failed_instance_parse::FailedInstanceParse;
use crate::error::state_machine::state::invalid_instance_text::InvalidInstanceText;
use crate::shared::filing_context::FilingContext;
use crate::shared::filing_input::InstanceFilingInput;
use crate::shared::parsed_filing::ParsedFiling;
use crate::shared::source_counts::SourceCounts;
use crate::traits::state_machine::state::State;

pub mod constants;

pub use constants::STATE_NAME;

/// Parses the main instance of one filing into a [`ParsedFiling`].
///
/// A successful output holds the facts, the filing, and the findings of the parser, the industry
/// of the stored `submissions` file, and the fact elements of the instance with the elements the
/// parser dropped.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParseInstanceFiling {
    input: InstanceFilingInput,
    context: FilingContext,
    output: Option<ParsedFiling>,
}

impl ParseInstanceFiling {
    /// Creates a new [`ParseInstanceFiling`] state from its input and context, with no output
    /// computed yet.
    #[must_use]
    pub const fn new(input: InstanceFilingInput, context: FilingContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    #[must_use]
    pub fn into_parts(self) -> (InstanceFilingInput, Option<ParsedFiling>, FilingContext) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl State for ParseInstanceFiling {
    /// Parses the instance and stores the [`ParsedFiling`] as output.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::InvalidInstanceText`] if the bytes are not UTF-8, and
    /// [`StateError::FailedInstanceParse`] if the parser rejects the text.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let reference = &self.input.filing;
        let accession_number = reference.accession_number().value();
        let document = std::str::from_utf8(&self.input.document).map_err(|error| {
            InvalidInstanceText::new(STATE_NAME, accession_number, error.to_string())
        })?;
        let input = InstanceFiling {
            document,
            id: FilingId::new(SEC_FILING_SOURCE, accession_number).expect(
                "Given an accession number of digits and dashes, the creation of a filing ID should always succeed",
            ),
            filed: reference.filed_date(),
        };
        let parsed = instance::parse(&input, self.context.registry.registry())
            .map_err(|error| FailedInstanceParse::new(STATE_NAME, error))?;
        let counts = SourceCounts::new(parsed.rows_in, parsed.duplicate_rows, &parsed.findings);
        let mut filing = parsed.filing;
        filing.company.industry = self
            .input
            .industry
            .as_ref()
            .map(|industry| industry.code.clone());
        self.output = Some(ParsedFiling {
            facts: parsed.facts,
            filing,
            findings: parsed.findings,
            source: self.input.source.clone(),
            industry_source: self.input.industry.clone(),
            counts,
        });
        Ok(())
    }
}

impl SMState for ParseInstanceFiling {
    type InputData = InstanceFilingInput;
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

impl fmt::Display for ParseInstanceFiling {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{STATE_NAME}` State Summary\n\
             ---------------------------\n\
             Input Data:\n\tFiling: {}\n\
             Output Data:\n\t{}",
            self.input.filing.accession_number().value(),
            self.output.as_ref().map_or_else(
                || "None".to_owned(),
                |output| format!("Facts: {}", output.facts.len())
            )
        )
    }
}

/// Writes the accession number and the counts of the output, not the facts, so that no row of a
/// filing enters event JSON (ADR 0048).
impl Serialize for ParseInstanceFiling {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ParseInstanceFiling", 3)?;
        state.serialize_field(
            "accession_number",
            self.input.filing.accession_number().value(),
        )?;
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

    use jiff::Timestamp;
    use jiff::civil::date;
    use pretty_assertions::assert_eq;
    use xbrl::core::filing::IndustryCode;
    use xbrl::error::parsing::ParseErrorKind;
    use xbrl::tests::fixtures::data::instance::INSTANCE_DOCUMENT;

    use super::*;
    use crate::shared::accession_number::AccessionNumber;
    use crate::shared::cik::Cik;
    use crate::shared::filing_input::StoredIndustry;
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::form::{Form, ParsedForm};
    use crate::shared::source_counts::DropReason;
    use crate::tests::fixtures::sample_filing_context::fixture_context;
    use crate::tests::fixtures::sample_source_document::{instance_source, notes_source};

    fn stored_industry() -> StoredIndustry {
        StoredIndustry {
            code: IndustryCode::new(IndustryCode::SIC_SCHEME, "4513")
                .expect("Given a hardcoded scheme and code, the creation should always succeed"),
            source: notes_source(),
            retrieved_at: Timestamp::UNIX_EPOCH,
        }
    }

    fn state_of(document: &[u8]) -> ParseInstanceFiling {
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
            document: document.to_vec(),
            source: instance_source(),
            industry: Some(stored_industry()),
        };
        ParseInstanceFiling::new(input, fixture_context())
    }

    async fn output_of(document: &[u8]) -> ParsedFiling {
        let mut state = state_of(document);
        state
            .compute_output_data_async()
            .await
            .expect("Given the instance of a real filing, the parse should always succeed");
        state
            .into_parts()
            .1
            .expect("Given a computed state, the output should always be present")
    }

    #[test]
    fn should_return_state_name_when_created() {
        let state = state_of(INSTANCE_DOCUMENT.as_bytes());

        let expected_result = "Parse Instance Filing";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_count_rows_in_and_dropped_rows_when_parsing_the_fixture_instance() {
        let document = INSTANCE_DOCUMENT.as_bytes();

        let expected_result = (678, BTreeMap::from([(DropReason::DuplicateRow, 65)]));

        let counts = output_of(document).await.counts;
        let result = (counts.rows_in(), counts.dropped().clone());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_facts_of_the_parser_when_parsing_the_fixture_instance() {
        let document = INSTANCE_DOCUMENT.as_bytes();

        let expected_result = 613;

        let result = output_of(document).await.facts.len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_set_the_industry_of_the_stored_submissions_when_parsing_the_fixture_instance() {
        let document = INSTANCE_DOCUMENT.as_bytes();

        let expected_result = (
            Some(stored_industry().code),
            Some(stored_industry()),
            instance_source(),
        );

        let output = output_of(document).await;
        let result = (
            output.filing.company.industry,
            output.industry_source,
            output.source,
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_instance_text_when_the_bytes_are_not_utf8() {
        let mut state = state_of(b"<xbrl>\xff</xbrl>");

        let expected_result = Err(StateError::InvalidInstanceText(InvalidInstanceText::new(
            "Parse Instance Filing",
            "0001628280-26-068246",
            "invalid utf-8 sequence of 1 bytes from index 6",
        )));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_instance_parse_when_the_text_is_not_xml() {
        let mut state = state_of(b"<xbrl>");

        let expected_result = Err(StateError::FailedInstanceParse(FailedInstanceParse::new(
            "Parse Instance Filing",
            ParseErrorKind::InvalidXml {
                filing: FilingId::new("sec", "0001628280-26-068246").expect(
                    "Given a hardcoded source and accession number, the creation should always succeed",
                ),
                reason: "the root node was opened but never closed".to_owned(),
            },
        )));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_parse_instance_filing() {
        implements_auto_traits::<ParseInstanceFiling>();
    }

    const fn implements_state<T: State + Serialize>() {}
    #[test]
    const fn should_implement_state_and_serialize_for_parse_instance_filing() {
        implements_state::<ParseInstanceFiling>();
    }
}
