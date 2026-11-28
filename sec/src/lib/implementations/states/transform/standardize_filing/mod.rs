//! # Standardize Filing State
//!
//! Provides the [`StandardizeFiling`] state, which turns the as-is facts of one source of one filing
//! into a [`StandardizedFiling`].
//!
//! The state runs three rules of `xbrl` in order: the dedup of copy ranks, the resolution of
//! aliases with the registry of the context, and the cadence filter with the filing. It keeps the
//! candidates that a conflict blocks, and the `Conflict` findings of the resolution. It returns no
//! error, since the rules return none.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//!
//! ## See Also
//!
//! - [`crate::shared::parsed_filing::ParsedFiling`]: The input of this state.
//! - [`crate::shared::standardized_filing::StandardizedFiling`]: The output of this state.
//! - [`crate::shared::filing_context::FilingContext`]: The context of this state.

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use serde::ser::{SerializeStruct, Serializer};
use state_maschine::prelude::State as SMState;
use xbrl::core::alias::resolve_aliases;
use xbrl::core::cadence_filter::keep_by_cadence;
use xbrl::core::dedup::more_precise;

use crate::error::State as StateError;
use crate::shared::filing_context::FilingContext;
use crate::shared::parsed_filing::ParsedFiling;
use crate::shared::standardized_filing::StandardizedFiling;
use crate::traits::state_machine::state::State;

pub mod constants;

pub use constants::STATE_NAME;

/// Turns the as-is facts of one source of one filing into a [`StandardizedFiling`].
///
/// The output holds the input unchanged, the candidates with the ones a conflict blocks, and one
/// `Conflict` finding per value key whose copy ranks disagree.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StandardizeFiling {
    input: ParsedFiling,
    context: FilingContext,
    output: Option<StandardizedFiling>,
}

impl StandardizeFiling {
    /// Creates a new [`StandardizeFiling`] state from its input and context, with no output
    /// computed yet.
    #[must_use]
    pub const fn new(input: ParsedFiling, context: FilingContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    #[must_use]
    pub fn into_parts(self) -> (ParsedFiling, Option<StandardizedFiling>, FilingContext) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl State for StandardizeFiling {
    /// Runs the dedup, the resolution of aliases, and the cadence filter, and stores the
    /// [`StandardizedFiling`] as output.
    ///
    /// # Errors
    ///
    /// Never returns an error.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let copies = more_precise(&self.input.facts);
        let resolved = resolve_aliases(&copies, self.context.registry.registry());
        let kept = keep_by_cadence(resolved, &self.input.filing);
        self.output = Some(StandardizedFiling {
            parsed: self.input.clone(),
            candidates: kept.candidates,
            conflicts: kept.findings,
        });
        Ok(())
    }
}

impl SMState for StandardizeFiling {
    type InputData = ParsedFiling;
    type OutputData = StandardizedFiling;
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

impl fmt::Display for StandardizeFiling {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{STATE_NAME}` State Summary\n\
             ---------------------------\n\
             Input Data:\n\tFiling: {}\n\
             Output Data:\n\t{}",
            self.input.filing.id.id(),
            self.output.as_ref().map_or_else(
                || "None".to_owned(),
                |output| format!("Candidates: {}", output.candidates.len())
            )
        )
    }
}

/// Writes the accession number, the number of candidates, and the number of conflicts of the
/// output. It writes no fact and no candidate.
impl Serialize for StandardizeFiling {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("StandardizeFiling", 3)?;
        state.serialize_field("accession_number", self.input.filing.id.id())?;
        state.serialize_field(
            "candidates_count",
            &self.output.as_ref().map(|output| output.candidates.len()),
        )?;
        state.serialize_field(
            "conflicts_count",
            &self.output.as_ref().map(|output| output.conflicts.len()),
        )?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use bigdecimal::BigDecimal;
    use pretty_assertions::assert_eq;
    use xbrl::core::candidate::CandidateValue;
    use xbrl::core::concept::ConceptName;
    use xbrl::core::fact::FactValue;
    use xbrl::core::fiscal_period::FiscalPeriod;
    use xbrl::tests::fixtures::data::notes::notes_fixture_rows;

    use super::*;
    use crate::implementations::states::transform::parse_notes_filing::ParseNotesFiling;
    use crate::shared::filing_input::NotesFilingInput;
    use crate::tests::fixtures::sample_filing_context::us_gaap_context;
    use crate::tests::fixtures::sample_source_document::notes_source;

    async fn fixture_parsed_filing() -> ParsedFiling {
        let input = NotesFilingInput {
            rows: notes_fixture_rows(),
            source: notes_source(),
        };
        let mut state = ParseNotesFiling::new(input, us_gaap_context());
        state
            .compute_output_data_async()
            .await
            .expect("Given the valid sub row of a real filing, the parse should always succeed");
        state
            .into_parts()
            .1
            .expect("Given a computed state, the output should always be present")
    }

    /// Adds a copy of rank 1 with another value to the as-is fact of the first candidate.
    async fn parsed_filing_with_conflict() -> ParsedFiling {
        let mut parsed = fixture_parsed_filing().await;
        let standardized = output_of(parsed.clone()).await;
        let key = standardized.candidates[0].inputs[0].clone();
        let mut copy = parsed
            .facts
            .iter()
            .find(|fact| fact.key() == key)
            .expect("Given the input key of a candidate, the filing should always hold the fact")
            .clone();
        let FactValue::Numeric(value) = &copy.value else {
            panic!("Given the input of a candidate, the fact should always hold a numeric value")
        };
        copy.value = FactValue::Numeric(value + BigDecimal::from(1_000_000_000));
        copy.copy_rank = 1;
        parsed.facts.push(copy);
        parsed
    }

    /// Adds a debt maturity bucket to the 10-Q fixture filing, on the period of report and the
    /// value of its long-term debt without dimensions.
    async fn parsed_filing_with_debt_maturity_bucket() -> ParsedFiling {
        let mut parsed = fixture_parsed_filing().await;
        let report = parsed.filing.period_of_report;
        let mut bucket = parsed
            .facts
            .iter()
            .find(|fact| {
                fact.concept.local_name()
                    == "LongTermDebtAndCapitalLeaseObligationsIncludingCurrentMaturities"
                    && fact.context.dimensions.is_empty()
                    && Some(fact.context.period.end()) == report
            })
            .expect(
                "Given the fixture 10-Q, the filing should always report its long-term debt at the period of report",
            )
            .clone();
        bucket.concept = ConceptName::new(
            bucket.concept.family().clone(),
            "LongTermDebtMaturitiesRepaymentsOfPrincipalInNextTwelveMonths",
        )
        .expect(
            "Given a hardcoded local name without whitespace, the creation should always succeed",
        );
        parsed.facts.push(bucket);
        parsed
    }

    async fn output_of(input: ParsedFiling) -> StandardizedFiling {
        let mut state = StandardizeFiling::new(input, us_gaap_context());
        state
            .compute_output_data_async()
            .await
            .expect("Given any as-is facts, the standardization should always succeed");
        state
            .into_parts()
            .1
            .expect("Given a computed state, the output should always be present")
    }

    #[tokio::test]
    async fn should_return_state_name_when_created() {
        let state = StandardizeFiling::new(fixture_parsed_filing().await, us_gaap_context());

        let expected_result = "Standardize Filing";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_facts_of_the_input_when_standardizing_the_fixture_filing() {
        let input = fixture_parsed_filing().await;

        let expected_result = input.facts.clone();

        let result = output_of(input).await.parsed.facts;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_give_candidates_when_standardizing_the_fixture_filing() {
        let input = fixture_parsed_filing().await;

        let expected_result = 88;

        let result = output_of(input).await.candidates.len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_drop_the_debt_maturity_bucket_when_the_filing_reports_no_fiscal_year() {
        let input = parsed_filing_with_debt_maturity_bucket().await;

        let expected_result = 88;

        let result = output_of(input).await.candidates.len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_debt_maturity_bucket_when_the_filing_reports_a_fiscal_year() {
        let mut input = parsed_filing_with_debt_maturity_bucket().await;
        input.filing.fiscal_period = Some(
            FiscalPeriod::new("FY")
                .expect("Given a hardcoded fiscal period, the creation should always succeed"),
        );

        let expected_result = 89;

        let result = output_of(input).await.candidates.len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_give_no_conflict_when_the_copy_ranks_of_the_fixture_filing_agree() {
        let input = fixture_parsed_filing().await;

        let expected_result = 0;

        let result = output_of(input).await.conflicts.len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_give_one_conflict_when_two_copy_ranks_disagree() {
        let input = parsed_filing_with_conflict().await;

        let expected_result = 1;

        let result = output_of(input).await.conflicts.len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_blocked_candidate_when_two_copy_ranks_disagree() {
        let input = parsed_filing_with_conflict().await;

        let expected_result = 1;

        let result = output_of(input)
            .await
            .candidates
            .iter()
            .filter(|candidate| candidate.value == CandidateValue::Blocked)
            .count();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_serialize_counts_instead_of_candidates_when_computed() {
        let mut state = StandardizeFiling::new(fixture_parsed_filing().await, us_gaap_context());
        state
            .compute_output_data_async()
            .await
            .expect("Given any as-is facts, the standardization should always succeed");

        let expected_result = serde_json::json!({
            "accession_number": "0001628280-26-068246",
            "candidates_count": 88,
            "conflicts_count": 0,
        });

        let result = serde_json::to_value(&state).expect(
            "Given a state of strings and numbers, the serialization should always succeed",
        );

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_standardize_filing() {
        implements_auto_traits::<StandardizeFiling>();
    }

    const fn implements_state<T: State + Serialize>() {}
    #[test]
    const fn should_implement_state_and_serialize_for_standardize_filing() {
        implements_state::<StandardizeFiling>();
    }
}
