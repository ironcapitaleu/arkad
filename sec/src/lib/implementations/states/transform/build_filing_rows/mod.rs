//! # Build Filing Rows State
//!
//! Provides the [`BuildFilingRows`] state, which turns a [`StandardizedFiling`] into the
//! [`FilingRows`] of the four datasets of a transform unit.
//!
//! The state adds the run columns of the context to each row, and the ingestion provenance of the
//! raw document to each row of `as_is_facts` and `filings`. It computes the entity bucket of each
//! candidate, and turns the parser findings and the `Conflict` findings into rows with
//! `step = filing`. It then builds the record batch of each dataset once, so a value that does not
//! fit its column fails the filing here and not the write of the unit.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//!
//! ## See Also
//!
//! - [`crate::shared::standardized_filing::StandardizedFiling`]: The input of this state.
//! - [`crate::shared::filing_rows::FilingRows`]: The output of this state.
//! - [`crate::shared::filing_context::FilingContext`]: The context of this state.

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use serde::ser::{SerializeStruct, Serializer};
use state_maschine::prelude::State as SMState;

use crate::datasets::{
    ArrowRow, ArrowRowError, AsIsFactRow, CandidateRow, FilingRow, FindingRow, Provenance,
    RunColumns, SicSource,
};
use crate::error::State as StateError;
use crate::error::state_machine::state::failed_row_build::FailedRowBuild;
use crate::shared::entity_bucket::entity_bucket;
use crate::shared::filing_context::FilingContext;
use crate::shared::filing_rows::FilingRows;
use crate::shared::standardized_filing::StandardizedFiling;
use crate::traits::state_machine::state::State;

pub mod constants;

pub use constants::STATE_NAME;

/// Turns the as-is output, the candidates, and the findings of one source of one filing into the
/// [`FilingRows`] of the four datasets.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildFilingRows {
    input: StandardizedFiling,
    context: FilingContext,
    output: Option<FilingRows>,
}

impl BuildFilingRows {
    /// Creates a new [`BuildFilingRows`] state from its input and context, with no output computed
    /// yet.
    #[must_use]
    pub const fn new(input: StandardizedFiling, context: FilingContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    #[must_use]
    pub fn into_parts(self) -> (StandardizedFiling, Option<FilingRows>, FilingContext) {
        (self.input, self.output, self.context)
    }

    /// Returns the rows of the filing, before the check of their Arrow form.
    fn rows(&self) -> FilingRows {
        let run = RunColumns {
            transform_version: self.context.transform_version,
            execution_id: self.context.execution_id,
            loaded_at: self.context.loaded_at,
        };
        let parsed = &self.input.parsed;
        let source = &parsed.source;
        let provenance = Provenance {
            source_kind: source.kind,
            source_key: source.key.as_path().display().to_string(),
            source_sha256: source.sha256.clone(),
            ingestion_mode: source.ingestion_mode,
            run,
        };
        let entity = &parsed.filing.entity;
        FilingRows {
            filing_id: parsed.filing.id.clone(),
            as_is_facts: parsed
                .facts
                .iter()
                .map(|fact| AsIsFactRow {
                    fact: fact.clone(),
                    provenance: provenance.clone(),
                })
                .collect(),
            filing: FilingRow {
                filing: parsed.filing.clone(),
                sic_source: parsed.industry_source.as_ref().map(|industry| SicSource {
                    source_key: industry.source.key.as_path().display().to_string(),
                    source_sha256: industry.source.sha256.clone(),
                    retrieved_at: industry.retrieved_at,
                }),
                provenance,
            },
            candidates: self
                .input
                .candidates
                .iter()
                .map(|candidate| CandidateRow {
                    entity_bucket: entity_bucket(&candidate.entity),
                    candidate: candidate.clone(),
                    run,
                })
                .collect(),
            findings: parsed
                .findings
                .iter()
                .chain(&self.input.conflicts)
                .filter_map(|finding| FindingRow::from_filing_finding(finding, entity, run))
                .collect(),
            counts: parsed.counts.clone(),
        }
    }
}

/// Builds the record batch of each dataset of `rows`, and drops it.
///
/// # Errors
///
/// Returns the [`ArrowRowError`] of the first dataset whose rows do not fit their columns.
fn check_arrow_form(rows: &FilingRows) -> Result<(), ArrowRowError> {
    AsIsFactRow::to_batch(&rows.as_is_facts)?;
    FilingRow::to_batch(std::slice::from_ref(&rows.filing))?;
    CandidateRow::to_batch(&rows.candidates)?;
    FindingRow::to_batch(&rows.findings)?;
    Ok(())
}

#[async_trait]
impl State for BuildFilingRows {
    /// Builds the rows of the filing, checks their Arrow form, and stores the [`FilingRows`] as
    /// output.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::FailedRowBuild`] if a value does not fit its column, for example a
    /// decimal with more than 38 digits.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let rows = self.rows();
        check_arrow_form(&rows)
            .map_err(|error| FailedRowBuild::new(STATE_NAME, rows.filing_id.to_string(), error))?;
        self.output = Some(rows);
        Ok(())
    }
}

impl SMState for BuildFilingRows {
    type InputData = StandardizedFiling;
    type OutputData = FilingRows;
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

impl fmt::Display for BuildFilingRows {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{STATE_NAME}` State Summary\n\
             ---------------------------\n\
             Input Data:\n\tFiling: {}\n\
             Output Data:\n\t{}",
            self.input.parsed.filing.id.id(),
            self.output.as_ref().map_or_else(
                || "None".to_owned(),
                |output| format!("As-is facts: {}", output.as_is_facts.len())
            )
        )
    }
}

/// Writes the accession number and the number of rows of each dataset of the output. It writes no
/// row.
impl Serialize for BuildFilingRows {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("BuildFilingRows", 4)?;
        state.serialize_field("accession_number", self.input.parsed.filing.id.id())?;
        state.serialize_field(
            "as_is_facts_count",
            &self.output.as_ref().map(|output| output.as_is_facts.len()),
        )?;
        state.serialize_field(
            "candidates_count",
            &self.output.as_ref().map(|output| output.candidates.len()),
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
    use bigdecimal::BigDecimal;
    use jiff::Timestamp;
    use pretty_assertions::assert_eq;
    use storage::Sha256Digest;
    use storage::implementations::repository::filesystem::IngestionMode;
    use uuid::Uuid;
    use xbrl::core::concept::NamespaceFamily;
    use xbrl::core::fact::FactValue;
    use xbrl::core::finding::Finding;
    use xbrl::tests::fixtures::data::notes::notes_fixture_rows;

    use super::*;
    use crate::datasets::{FindingKind, SourceKind};
    use crate::implementations::states::transform::parse_notes_filing::ParseNotesFiling;
    use crate::implementations::states::transform::standardize_filing::StandardizeFiling;
    use crate::shared::filing_input::{NotesFilingInput, StoredIndustry};
    use crate::tests::fixtures::sample_filing_context::us_gaap_context;
    use crate::tests::fixtures::sample_source_document::{instance_source, notes_source};

    async fn fixture_standardized_filing() -> StandardizedFiling {
        let input = NotesFilingInput {
            rows: notes_fixture_rows(),
            source: notes_source(),
        };
        let mut parse = ParseNotesFiling::new(input, us_gaap_context());
        parse
            .compute_output_data_async()
            .await
            .expect("Given the valid sub row of a real filing, the parse should always succeed");
        let mut standardize = StandardizeFiling::try_from(parse)
            .expect("Given a computed parse state, the transition should always succeed");
        standardize
            .compute_output_data_async()
            .await
            .expect("Given any as-is facts, the standardization should always succeed");
        standardize
            .into_parts()
            .1
            .expect("Given a computed state, the output should always be present")
    }

    async fn output_of(input: StandardizedFiling) -> FilingRows {
        let mut state = BuildFilingRows::new(input, us_gaap_context());
        state
            .compute_output_data_async()
            .await
            .expect("Given the rows of a real filing, the row build should always succeed");
        state
            .into_parts()
            .1
            .expect("Given a computed state, the output should always be present")
    }

    #[tokio::test]
    async fn should_return_state_name_when_created() {
        let state = BuildFilingRows::new(fixture_standardized_filing().await, us_gaap_context());

        let expected_result = "Build Filing Rows";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_give_one_row_per_fact_and_candidate_when_building_the_fixture_filing() {
        let input = fixture_standardized_filing().await;

        let expected_result = (613, 88, 0);

        let output = output_of(input).await;
        let result = (
            output.as_is_facts.len(),
            output.candidates.len(),
            output.findings.len(),
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_add_the_provenance_of_the_notes_file_and_the_context_to_the_filing_row() {
        let input = fixture_standardized_filing().await;

        let expected_result = Provenance {
            source_kind: SourceKind::Notes,
            source_key: "sec/notes/2025q3_notes.zip".to_owned(),
            source_sha256: Sha256Digest::of(b"notes"),
            ingestion_mode: IngestionMode::Backfill,
            run: RunColumns {
                transform_version: 1,
                execution_id: Uuid::nil(),
                loaded_at: Timestamp::UNIX_EPOCH,
            },
        };

        let result = output_of(input).await.filing.provenance;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_give_every_candidate_the_bucket_of_the_fixture_entity() {
        let input = fixture_standardized_filing().await;

        // The reference implementation of XXH3-64 (the `xxhash` package of Python) hashes
        // `http://www.sec.gov/CIK:0002082247` to 2481830158971776855, which is 23 modulo 64.
        let expected_result = vec![23];

        let mut result: Vec<u8> = output_of(input)
            .await
            .candidates
            .iter()
            .map(|row| row.entity_bucket)
            .collect();
        result.dedup();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_give_a_finding_row_of_the_filing_entity_when_the_parser_gave_a_finding() {
        let mut input = fixture_standardized_filing().await;
        input.parsed.findings.push(Finding::UndeclaredFamily {
            filing: input.parsed.filing.id.clone(),
            family: NamespaceFamily::new("xyz/2027").expect(
                "Given a hardcoded family without whitespace, the creation should always succeed",
            ),
        });

        let expected_result = vec![(
            FindingKind::UndeclaredFamily,
            "http://www.sec.gov/CIK:0002082247".to_owned(),
        )];

        let result: Vec<(FindingKind, String)> = output_of(input)
            .await
            .findings
            .iter()
            .map(|row| {
                (
                    row.kind,
                    format!("{}:{}", row.entity.scheme(), row.entity.id()),
                )
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_the_submissions_file_as_sic_source_when_the_filing_has_one() {
        let mut input = fixture_standardized_filing().await;
        input.parsed.industry_source = Some(StoredIndustry {
            code: xbrl::core::filing::IndustryCode::new(
                xbrl::core::filing::IndustryCode::SIC_SCHEME,
                "4513",
            )
            .expect("Given a hardcoded scheme and code, the creation should always succeed"),
            source: instance_source(),
            retrieved_at: Timestamp::UNIX_EPOCH,
        });

        let expected_result = Some(SicSource {
            source_key: "sec/filings/320193/000032019326000001/instance.xml".to_owned(),
            source_sha256: Sha256Digest::of(b"instance"),
            retrieved_at: Timestamp::UNIX_EPOCH,
        });

        let result = output_of(input).await.filing.sic_source;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_row_build_when_a_fact_value_does_not_fit_its_column() {
        let mut input = fixture_standardized_filing().await;
        let index = input
            .parsed
            .facts
            .iter()
            .position(|fact| matches!(fact.value, FactValue::Numeric(_)))
            .expect("Given the fixture 10-Q, the filing should always hold a numeric fact");
        input.parsed.facts[index].value = FactValue::Numeric(BigDecimal::from(10_i128.pow(30)));
        let mut state = BuildFilingRows::new(input, us_gaap_context());

        let expected_result = Err(StateError::FailedRowBuild(FailedRowBuild::new(
            "Build Filing Rows",
            "sec:0001628280-26-068246",
            ArrowRowError::UnexpectedColumnValue {
                dataset: "as_is_facts",
                column: "value_numeric",
                row: index,
                reason: "'1000000000000000000000000000000' has more than 38 digits".to_owned(),
            },
        )));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_serialize_counts_instead_of_rows_when_computed() {
        let mut state =
            BuildFilingRows::new(fixture_standardized_filing().await, us_gaap_context());
        state
            .compute_output_data_async()
            .await
            .expect("Given the rows of a real filing, the row build should always succeed");

        let expected_result = serde_json::json!({
            "accession_number": "0001628280-26-068246",
            "as_is_facts_count": 613,
            "candidates_count": 88,
            "findings_count": 0,
        });

        let result = serde_json::to_value(&state).expect(
            "Given a state of strings and numbers, the serialization should always succeed",
        );

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_build_filing_rows() {
        implements_auto_traits::<BuildFilingRows>();
    }

    const fn implements_state<T: State + Serialize>() {}
    #[test]
    const fn should_implement_state_and_serialize_for_build_filing_rows() {
        implements_state::<BuildFilingRows>();
    }
}
