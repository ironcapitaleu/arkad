//! # Parse Company Facts State
//!
//! This module provides the [`ParseCompanyFacts`] state and related types for parsing and
//! validating SEC Company Facts JSON data as part of the SEC filings transformation workflow.
//!
//! ## Overview
//! The [`ParseCompanyFacts`] state is responsible for taking raw SEC Company Facts JSON
//! and resolving XBRL concepts into typed [`CompanyData`].
//! It validates that all required concepts are present and produces structured financial data
//! suitable for downstream processing.
//!
//! ## Components
//! - [`context`]: Defines the context and updater types for the parsing process.
//! - [`data`]: Contains input and output data structures for the parsing state.
//! - [`ParseCompanyFactsContext`]: Context data type for the state.
//! - [`ParseCompanyFactsInput`]: Input data type holding the raw JSON response.
//! - [`ParseCompanyFactsOutput`]: Output data type containing the parsed company data.
//!
//! ## Usage
//! This state is typically used in the transform phase of the SEC state machine ETL pipeline,
//! after extracting the raw JSON from the SEC API. It resolves XBRL concepts from the JSON
//! into strongly-typed financial data structures.
//!
//! ## See Also
//! - [`crate::implementations::states::transform`]: Parent module for transformation-related states.
//! - [`crate::shared::financial`]: Financial data types used by this state.
//! - [`crate::traits::state_machine::state::State`]: State trait implemented by [`ParseCompanyFacts`].
//!
//! ## Testing
//! This module includes comprehensive unit tests covering state behavior, trait compliance, and error handling.

use std::collections::HashMap;
use std::fmt;

use async_trait::async_trait;
use chrono::NaiveDate;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::error::state_machine::state::IncompleteCompanyFacts;
use crate::shared::cik::Cik;
use crate::shared::financial::accession_number::AccessionNumber;
use crate::shared::financial::company_data::CompanyData;
use crate::shared::financial::company_fact::CompanyFact;
use crate::shared::financial::concept_definition::ConceptDefinition;
use crate::shared::financial::concept_definition::constants::{
    COMPANY_INFO_NAMESPACE, OPTIONAL_CONCEPTS, REQUIRED_CONCEPTS, REQUIRED_FACTS_NAMESPACE,
    SHARES_OUTSTANDING,
};
use crate::shared::financial::entity_name::EntityName;
use crate::shared::financial::filing_source::FilingSource;
use crate::shared::financial::fiscal_period::FiscalPeriod;
use crate::shared::financial::fiscal_year::FiscalYear;
use crate::shared::financial::form::Form;
use crate::shared::financial::frame::Frame;
use crate::shared::financial::observation::Observation;
use crate::shared::financial::period::Period;
use crate::shared::financial::unit::Unit;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::ParseCompanyFactsContext;
pub use data::ParseCompanyFactsInput;
pub use data::ParseCompanyFactsOutput;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
/// State that parses SEC Company Facts JSON into structured financial data.
///
/// The state takes a raw SEC Company Facts JSON response as input, validates
/// the presence of required XBRL concepts, resolves them into strongly-typed
/// [`CompanyData`], and produces the result as output.
///
/// # Behavior
/// - Extracts the CIK, entity name, and facts from the top-level JSON.
/// - Iterates over required and optional concept definitions.
/// - For each concept, tries XBRL key aliases in priority order.
/// - Extracts observations from the matching concept's unit data.
/// - Fails if any required concept is missing from the response.
///
/// # Output
/// The parsed [`CompanyData`] is stored internally after calling
/// [`State::compute_output_data_async`].
///
/// # Example
/// ```rust
/// use sec::implementations::states::transform::parse_company_facts::*;
/// use sec::shared::response::implementations::sec_response::body_digest::BodyDigest;
///
/// let json = serde_json::json!({});
/// let digest = BodyDigest::from_body_text(&json.to_string());
/// let input = ParseCompanyFactsInput::new(json, digest);
/// let context = ParseCompanyFactsContext::default();
/// let state = ParseCompanyFacts::new(input, context);
/// ```
pub struct ParseCompanyFacts {
    input: ParseCompanyFactsInput,
    context: ParseCompanyFactsContext,
    output: Option<ParseCompanyFactsOutput>,
}

impl ParseCompanyFacts {
    /// Creates a new [`ParseCompanyFacts`] state with the given input and context.
    #[must_use]
    pub const fn new(input: ParseCompanyFactsInput, context: ParseCompanyFactsContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its components. Used for state transitions.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        ParseCompanyFactsInput,
        Option<ParseCompanyFactsOutput>,
        ParseCompanyFactsContext,
    ) {
        (self.input, self.output, self.context)
    }
}

/// Attempts to parse a date string in `YYYY-MM-DD` format into a [`NaiveDate`].
fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Builds the [`Period`] for an observation entry from the SEC JSON.
///
/// SEC data uses `"end"` for all entries and optionally `"start"` for duration measurements.
/// If `"start"` is present, it is a duration; otherwise, it is an instant.
fn build_period(entry: &serde_json::Value) -> Option<Period> {
    let end = parse_date(entry.get("end")?.as_str()?)?;
    if let Some(start_val) = entry.get("start") {
        let start = parse_date(start_val.as_str()?)?;
        Some(Period::Duration { start, end })
    } else {
        Some(Period::Instant { date: end })
    }
}

/// Builds a [`FilingSource`] from an SEC observation JSON entry.
fn build_filing_source(entry: &serde_json::Value) -> Option<FilingSource> {
    let accn = AccessionNumber::new(entry.get("accn")?.as_str()?);
    let form = Form::from_sec_str(entry.get("form")?.as_str()?)?;
    let fy = FiscalYear::try_from(entry.get("fy")?.as_u64()?).ok()?;
    let fp = FiscalPeriod::from_sec_str(entry.get("fp")?.as_str()?)?;
    let filed = parse_date(entry.get("filed")?.as_str()?)?;
    let end = parse_date(entry.get("end")?.as_str()?)?;

    Some(FilingSource::new(accn, form, fy, fp, filed, end))
}

/// Parses a single observation entry from the SEC JSON unit array.
fn parse_observation(entry: &serde_json::Value, unit: Unit) -> Option<Observation> {
    let value = entry.get("val")?.as_i64()?;
    let period = build_period(entry)?;
    let frame = entry
        .get("frame")
        .and_then(serde_json::Value::as_str)
        .and_then(Frame::parse);
    let filing = build_filing_source(entry)?;

    Some(Observation::new(value, unit, period, frame, filing))
}

/// Determines the JSON namespace key for a given concept definition.
///
/// The `SHARES_OUTSTANDING` concept uses the `dei` namespace; all others use `us-gaap`.
fn namespace_for_concept(concept: &ConceptDefinition) -> &'static str {
    if concept.canonical_name() == SHARES_OUTSTANDING {
        COMPANY_INFO_NAMESPACE
    } else {
        REQUIRED_FACTS_NAMESPACE
    }
}

/// Attempts to resolve a [`ConceptDefinition`] from the SEC facts JSON.
///
/// Returns `Some((CompanyFact, &'static ConceptDefinition))` if the concept was found,
/// or `None` if none of the XBRL key aliases matched.
fn resolve_concept(
    facts: &serde_json::Value,
    concept: &'static ConceptDefinition,
) -> Option<CompanyFact> {
    let namespace = namespace_for_concept(concept);
    let namespace_data = facts.get(namespace)?;

    for &xbrl_key in concept.xbrl_keys() {
        if let Some(concept_data) = namespace_data.get(xbrl_key) {
            let company_label = concept_data
                .get("label")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| concept.canonical_name())
                .to_string();

            let unit_key = concept.expected_unit().to_string();
            let units_data = concept_data.get("units")?;
            let unit_array = units_data.get(&unit_key)?.as_array()?;

            let observations: Vec<Observation> = unit_array
                .iter()
                .filter_map(|entry| parse_observation(entry, concept.expected_unit()))
                .collect();

            return Some(CompanyFact::new(
                company_label,
                xbrl_key.to_string(),
                observations,
            ));
        }
    }

    None
}

#[async_trait]
impl State for ParseCompanyFacts {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let response = &self.input.response;

        // Validate top-level structure
        let obj = response
            .as_object()
            .ok_or(StateError::FailedOutputComputation)?;

        let cik_value = obj.get("cik").ok_or(StateError::FailedOutputComputation)?;

        let cik_str = if let Some(n) = cik_value.as_u64() {
            n.to_string()
        } else if let Some(s) = cik_value.as_str() {
            s.to_string()
        } else {
            return Err(StateError::FailedOutputComputation);
        };

        let cik = Cik::new(&cik_str).map_err(|_| StateError::FailedOutputComputation)?;

        let entity_name_str = obj
            .get("entityName")
            .and_then(serde_json::Value::as_str)
            .ok_or(StateError::FailedOutputComputation)?;

        let entity_name = EntityName::new(entity_name_str);

        let facts = obj
            .get("facts")
            .ok_or(StateError::FailedOutputComputation)?;

        // Resolve required and optional concepts
        let mut resolved_facts: HashMap<&'static ConceptDefinition, CompanyFact> = HashMap::new();
        let mut missing_fields: Vec<String> = Vec::new();

        for concept in REQUIRED_CONCEPTS {
            if let Some(fact) = resolve_concept(facts, concept) {
                resolved_facts.insert(concept, fact);
            } else {
                missing_fields.push(concept.canonical_name().to_string());
            }
        }

        if !missing_fields.is_empty() {
            let error = IncompleteCompanyFacts::new(self.state_name().to_string(), missing_fields);
            return Err(error.into());
        }

        for concept in OPTIONAL_CONCEPTS {
            if let Some(fact) = resolve_concept(facts, concept) {
                resolved_facts.insert(concept, fact);
            }
        }

        let company_data = CompanyData::new(cik, entity_name, resolved_facts);
        self.output = Some(ParseCompanyFactsOutput::new(company_data));

        Ok(())
    }
}

impl SMState for ParseCompanyFacts {
    type InputData = ParseCompanyFactsInput;
    type OutputData = ParseCompanyFactsOutput;
    type Context = ParseCompanyFactsContext;

    fn state_name(&self) -> impl ToString {
        STATE_NAME
    }

    /// Parses SEC Company Facts JSON into structured financial data.
    /// Panics unconditionally — SEC states are async-only.
    fn compute_output_data(&mut self) {
        unimplemented!(
            "SEC states are async-only. \
             Call compute_output_data_async instead"
        )
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

impl fmt::Display for ParseCompanyFacts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::response::implementations::sec_response::body_digest::BodyDigest;
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};
    use tokio;

    fn test_input() -> ParseCompanyFactsInput {
        let json = serde_json::json!({});
        let digest = BodyDigest::from_body_text(&json.to_string());
        ParseCompanyFactsInput::new(json, digest)
    }

    fn get_baseline_parse_state() -> ParseCompanyFacts {
        let input = test_input();
        let cik =
            Cik::new("0001067983").expect("Hardcoded CIK should always be valid");
        let context = ParseCompanyFactsContext::new(cik);
        ParseCompanyFacts::new(input, context)
    }

    /// Creates a minimal valid SEC Company Facts JSON for testing.
    #[allow(clippy::too_many_lines)]
    fn get_baseline_company_facts_json() -> serde_json::Value {
        serde_json::json!({
            "cik": 320_193,
            "entityName": "Apple Inc.",
            "facts": {
                "us-gaap": {
                    "Revenues": {
                        "label": "Net Sales",
                        "description": "Total revenue",
                        "units": {
                            "USD": [
                                {
                                    "start": "2022-10-01",
                                    "end": "2023-09-30",
                                    "val": 383_285_000_000_i64,
                                    "accn": "0000320193-23-000106",
                                    "fy": 2023,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2023-11-03",
                                    "frame": "CY2023"
                                }
                            ]
                        }
                    },
                    "OperatingIncomeLoss": {
                        "label": "Operating Income",
                        "description": "Operating income",
                        "units": {
                            "USD": [
                                {
                                    "start": "2022-10-01",
                                    "end": "2023-09-30",
                                    "val": 114_301_000_000_i64,
                                    "accn": "0000320193-23-000106",
                                    "fy": 2023,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2023-11-03",
                                    "frame": "CY2023"
                                }
                            ]
                        }
                    },
                    "IncomeTaxExpenseBenefit": {
                        "label": "Income Tax Expense",
                        "description": "Income tax",
                        "units": {
                            "USD": [
                                {
                                    "start": "2022-10-01",
                                    "end": "2023-09-30",
                                    "val": 16_741_000_000_i64,
                                    "accn": "0000320193-23-000106",
                                    "fy": 2023,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2023-11-03",
                                    "frame": "CY2023"
                                }
                            ]
                        }
                    },
                    "NetIncomeLoss": {
                        "label": "Net Income",
                        "description": "Net income",
                        "units": {
                            "USD": [
                                {
                                    "start": "2022-10-01",
                                    "end": "2023-09-30",
                                    "val": 96_995_000_000_i64,
                                    "accn": "0000320193-23-000106",
                                    "fy": 2023,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2023-11-03",
                                    "frame": "CY2023"
                                }
                            ]
                        }
                    },
                    "Assets": {
                        "label": "Total Assets",
                        "description": "Total assets",
                        "units": {
                            "USD": [
                                {
                                    "end": "2023-09-30",
                                    "val": 352_583_000_000_i64,
                                    "accn": "0000320193-23-000106",
                                    "fy": 2023,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2023-11-03",
                                    "frame": "CY2023Q3I"
                                }
                            ]
                        }
                    },
                    "Liabilities": {
                        "label": "Total Liabilities",
                        "description": "Total liabilities",
                        "units": {
                            "USD": [
                                {
                                    "end": "2023-09-30",
                                    "val": 290_437_000_000_i64,
                                    "accn": "0000320193-23-000106",
                                    "fy": 2023,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2023-11-03",
                                    "frame": "CY2023Q3I"
                                }
                            ]
                        }
                    },
                    "StockholdersEquity": {
                        "label": "Stockholders Equity",
                        "description": "Total equity",
                        "units": {
                            "USD": [
                                {
                                    "end": "2023-09-30",
                                    "val": 62_146_000_000_i64,
                                    "accn": "0000320193-23-000106",
                                    "fy": 2023,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2023-11-03",
                                    "frame": "CY2023Q3I"
                                }
                            ]
                        }
                    },
                    "NetCashProvidedByUsedInOperatingActivities": {
                        "label": "Cash from Operations",
                        "description": "Operating cash flow",
                        "units": {
                            "USD": [
                                {
                                    "start": "2022-10-01",
                                    "end": "2023-09-30",
                                    "val": 110_543_000_000_i64,
                                    "accn": "0000320193-23-000106",
                                    "fy": 2023,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2023-11-03",
                                    "frame": "CY2023"
                                }
                            ]
                        }
                    },
                    "NetCashProvidedByUsedInInvestingActivities": {
                        "label": "Cash from Investing",
                        "description": "Investing cash flow",
                        "units": {
                            "USD": [
                                {
                                    "start": "2022-10-01",
                                    "end": "2023-09-30",
                                    "val": -3_548_000_000_i64,
                                    "accn": "0000320193-23-000106",
                                    "fy": 2023,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2023-11-03",
                                    "frame": "CY2023"
                                }
                            ]
                        }
                    },
                    "NetCashProvidedByUsedInFinancingActivities": {
                        "label": "Cash from Financing",
                        "description": "Financing cash flow",
                        "units": {
                            "USD": [
                                {
                                    "start": "2022-10-01",
                                    "end": "2023-09-30",
                                    "val": -108_488_000_000_i64,
                                    "accn": "0000320193-23-000106",
                                    "fy": 2023,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2023-11-03",
                                    "frame": "CY2023"
                                }
                            ]
                        }
                    }
                },
                "dei": {
                    "EntityCommonStockSharesOutstanding": {
                        "label": "Shares Outstanding",
                        "description": "Common shares outstanding",
                        "units": {
                            "shares": [
                                {
                                    "end": "2023-10-20",
                                    "val": 15_550_061_000_i64,
                                    "accn": "0000320193-23-000106",
                                    "fy": 2023,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2023-11-03"
                                }
                            ]
                        }
                    }
                }
            }
        })
    }

    #[test]
    fn should_return_name_of_parse_state_when_in_parse_state() {
        let parse_state = get_baseline_parse_state();

        let expected_result = String::from("Parse Company Facts");

        let result = parse_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_input_data_when_in_initial_state() {
        let parse_state = get_baseline_parse_state();

        let expected_result = &test_input();

        let result = parse_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let parse_state = get_baseline_parse_state();

        let _result = parse_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let parse_state = get_baseline_parse_state();

        let expected_result = false;

        let result = parse_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_context_data_when_in_initial_state() {
        let parse_state = get_baseline_parse_state();

        let expected_result = &ParseCompanyFactsContext::default();

        let result = parse_state.context_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_produce_output_with_correct_entity_name_when_computing_valid_json() {
        let json = get_baseline_company_facts_json();
        let digest = BodyDigest::from_body_text(&json.to_string());
        let input = ParseCompanyFactsInput::new(json, digest);
        let context = ParseCompanyFactsContext::default();
        let mut parse_state = ParseCompanyFacts::new(input, context);

        let expected_result = "Apple Inc.";

        parse_state
            .compute_output_data_async()
            .await
            .expect("Valid complete JSON should always parse successfully");
        let result = parse_state
            .output_data()
            .expect("Output should be present after successful computation")
            .company_data()
            .entity_name()
            .value();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_resolve_all_required_concepts_when_computing_valid_json() {
        let json = get_baseline_company_facts_json();
        let digest = BodyDigest::from_body_text(&json.to_string());
        let input = ParseCompanyFactsInput::new(json, digest);
        let context = ParseCompanyFactsContext::default();
        let mut parse_state = ParseCompanyFacts::new(input, context);

        let expected_result = REQUIRED_CONCEPTS.len();

        parse_state
            .compute_output_data_async()
            .await
            .expect("Valid complete JSON should always parse successfully");
        let result = parse_state
            .output_data()
            .expect("Output should be present after successful computation")
            .company_data()
            .facts()
            .len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_error_when_required_concept_is_missing() {
        let json = serde_json::json!({
            "cik": 320_193,
            "entityName": "Apple Inc.",
            "facts": {
                "us-gaap": {},
                "dei": {}
            }
        });
        let digest = BodyDigest::from_body_text(&json.to_string());
        let input = ParseCompanyFactsInput::new(json, digest);
        let context = ParseCompanyFactsContext::default();
        let mut parse_state = ParseCompanyFacts::new(input, context);

        let expected_result = true;

        let result = parse_state.compute_output_data_async().await.is_err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_change_input_data_when_computing_output_data() {
        let json = get_baseline_company_facts_json();
        let digest = BodyDigest::from_body_text(&json.to_string());
        let input = ParseCompanyFactsInput::new(json, digest);
        let context = ParseCompanyFactsContext::default();
        let mut parse_state = ParseCompanyFacts::new(input, context);

        let expected_result = &parse_state.input_data().clone();

        parse_state
            .compute_output_data_async()
            .await
            .expect("Valid complete JSON should always parse successfully");
        let result = parse_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_error_when_response_is_not_an_object() {
        let json = serde_json::json!("not an object");
        let digest = BodyDigest::from_body_text(&json.to_string());
        let input = ParseCompanyFactsInput::new(json, digest);
        let context = ParseCompanyFactsContext::default();
        let mut parse_state = ParseCompanyFacts::new(input, context);

        let expected_result = true;

        let result = parse_state.compute_output_data_async().await.is_err();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_trait() {
        implements_auto_traits::<ParseCompanyFacts>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_trait() {
        implements_send::<ParseCompanyFacts>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_trait() {
        implements_sync::<ParseCompanyFacts>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_trait() {
        implements_send::<ParseCompanyFacts>();
        implements_sync::<ParseCompanyFacts>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_trait() {
        implements_sized::<ParseCompanyFacts>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_trait() {
        implements_hash::<ParseCompanyFacts>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_trait() {
        implements_partial_eq::<ParseCompanyFacts>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_trait() {
        implements_eq::<ParseCompanyFacts>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_trait() {
        implements_partial_ord::<ParseCompanyFacts>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_trait() {
        implements_ord::<ParseCompanyFacts>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_trait() {
        implements_debug::<ParseCompanyFacts>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_trait() {
        implements_clone::<ParseCompanyFacts>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_trait() {
        implements_unpin::<ParseCompanyFacts>();
    }

    #[test]
    fn should_return_context_data_when_called_with_state_reference() {
        let parse_state = &get_baseline_parse_state();
        let ref_to_parse_state = &get_baseline_parse_state();

        let expected_result = parse_state.context_data();

        let result = ref_to_parse_state.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_parse_state = &mut get_baseline_parse_state();

        let expected_result = false;

        let result = ref_to_parse_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_reference_state()
     {
        let ref_to_parse_state = &get_baseline_parse_state();

        let _result = ref_to_parse_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_name_of_parse_state_when_calling_reference_to_parse_state() {
        let ref_to_parse_state = &get_baseline_parse_state();

        let expected_result = String::from("Parse Company Facts");

        let result = ref_to_parse_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_input_data_when_reference_parse_state_in_initial_state() {
        let ref_to_parse_state = &get_baseline_parse_state();

        let expected_result = &test_input();

        let result = ref_to_parse_state.input_data();

        assert_eq!(result, expected_result);
    }
}
