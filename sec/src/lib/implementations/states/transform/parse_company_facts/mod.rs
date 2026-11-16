//! # Parse Company Facts State
//!
//! Provides the [`ParseCompanyFacts`] state, which parses the SEC Company Facts JSON into
//! [`CompanyData`].
//!
//! The state calls the Company Facts parser of `xbrl` and keeps every fact of every form,
//! amendment, unit, and namespace. It fails if the facts are about another company than the CIK
//! of its context, or if a required standard concept has no `us-gaap` fact.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`ParseCompanyFactsContext`] carried alongside the state.
//! - [`data`]: The [`ParseCompanyFactsInput`] and [`ParseCompanyFactsOutput`] data types.
//!
//! ## See Also
//!
//! - [`crate::implementations::states::transform`]: Parent module describing the transform flow.
//! - [`crate::shared::company_data`]: The [`CompanyData`] this state produces.
//! - [`crate::traits::state_machine::state::State`]: The trait implemented by [`ParseCompanyFacts`].

use std::collections::HashSet;
use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;
use xbrl::adapter::edgar::company_facts;
use xbrl::core::entity_name::EntityName;
use xbrl::core::fact::AsIsFact;
use xbrl::us_gaap::mappings::{PART_CONCEPTS, PartDefinition, REQUIRED_CONCEPTS};

use crate::error::State as StateError;
use crate::error::state_machine::state::failed_company_facts_parse::FailedCompanyFactsParse;
use crate::error::state_machine::state::incomplete_company_facts::{
    IncompleteCompanyFacts, MissingFields,
};
use crate::error::state_machine::state::unexpected_cik::UnexpectedCik;
use crate::shared::company_data::CompanyData;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use constants::US_GAAP_FAMILY;
pub use context::ParseCompanyFactsContext;
pub use data::ParseCompanyFactsInput;
pub use data::ParseCompanyFactsOutput;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Parses the Company Facts JSON into [`CompanyData`].
///
/// A successful output holds every fact of the body, is about the CIK of the context, and has a
/// `us-gaap` fact for each required standard concept.
pub struct ParseCompanyFacts {
    input: ParseCompanyFactsInput,
    context: ParseCompanyFactsContext,
    output: Option<ParseCompanyFactsOutput>,
}

impl ParseCompanyFacts {
    /// Creates a new [`ParseCompanyFacts`] state from its input and context, with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::transform::parse_company_facts::*;
    /// use sec::shared::cik::Cik;
    /// use state_maschine::prelude::State as SMState;
    /// use storage::shared::Sha256Digest;
    ///
    /// let json = serde_json::json!({});
    /// let digest = Sha256Digest::of(json.to_string());
    /// let cik = Cik::new("0001067983").expect("A hardcoded valid CIK should always parse");
    /// let input = ParseCompanyFactsInput::new(json, digest);
    /// let context = ParseCompanyFactsContext::new(cik);
    /// let state = ParseCompanyFacts::new(input, context);
    ///
    /// let expected_result = None;
    ///
    /// let result = state.output_data();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(input: ParseCompanyFactsInput, context: ParseCompanyFactsContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
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

/// Returns the labels of the required standard concepts that no `us-gaap` fact reports.
///
/// A standard concept is reported if a `us-gaap` fact has a local name in its `xbrl_keys` or in
/// the parts of its [`PART_CONCEPTS`] entry.
fn missing_required_concepts(facts: &[AsIsFact]) -> Vec<String> {
    let reported: HashSet<&str> = facts
        .iter()
        .filter(|fact| fact.concept.family().as_str() == US_GAAP_FAMILY)
        .map(|fact| fact.concept.local_name())
        .collect();

    REQUIRED_CONCEPTS
        .iter()
        .filter(|definition| {
            let parts = PART_CONCEPTS
                .iter()
                .filter(|part| part.canonical_element() == definition.canonical_element())
                .flat_map(PartDefinition::parts);
            !definition
                .xbrl_keys()
                .iter()
                .chain(parts)
                .any(|name| reported.contains(name))
        })
        .map(|definition| definition.canonical_element().to_string())
        .collect()
}

#[async_trait]
impl State for ParseCompanyFacts {
    /// Parses the Company Facts JSON into [`CompanyData`] and stores it as output.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::FailedCompanyFactsParse`] when the parser rejects the JSON,
    /// [`StateError::UnexpectedCik`] when a fact is about another CIK than the context, and
    /// [`StateError::IncompleteCompanyFacts`] when a required standard concept has no `us-gaap`
    /// fact.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let state_name = self.state_name().to_string();
        let response = &self.input.response;
        let cik = self.context.cik();

        let parsed = company_facts::parse(response)
            .map_err(|error| FailedCompanyFactsParse::new(&state_name, error))?;
        let entity_name = company_facts::extract_entity_name(response)
            .map_err(|error| FailedCompanyFactsParse::new(&state_name, error))?;

        let other_entity = parsed
            .facts
            .iter()
            .map(|fact| fact.context.entity.id())
            .find(|entity| *entity != cik.value());
        if let Some(found) = other_entity {
            return Err(UnexpectedCik::new(state_name, cik.clone(), found).into());
        }

        let missing = missing_required_concepts(&parsed.facts);
        if !missing.is_empty() {
            return Err(
                IncompleteCompanyFacts::new(state_name, MissingFields::new(missing)).into(),
            );
        }

        let company_data = CompanyData::new(
            cik.clone(),
            EntityName::new(entity_name),
            parsed.facts,
            parsed.filings,
        );
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

    /// Blocking wrapper around [`compute_output_data_async`](crate::traits::state_machine::state::State::compute_output_data_async).
    ///
    /// Detects whether a tokio runtime is available and runs the async computation
    /// synchronously. This allows SEC states to be used as regular `SMState` implementations.
    ///
    /// # Panics
    /// - Panics if the async computation returns an error.
    /// - Panics if called inside a `current_thread` runtime (`block_in_place`
    ///   requires a multi-thread runtime). All binaries and tests in this project
    ///   use `flavor = "multi_thread"`.
    fn compute_output_data(&mut self) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(self.compute_output_data_async()))
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.compute_output_data_async())
        };

        if let Err(e) = result {
            let state_err: StateError = e;
            panic!("compute_output_data failed: {state_err}")
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
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use storage::shared::Sha256Digest;
    use tokio;
    use xbrl::error::parsing::ParseErrorKind;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK;

    const APPLE_CIK: &str = "0000320193";

    fn test_input() -> ParseCompanyFactsInput {
        let json = serde_json::json!({});
        let digest = Sha256Digest::of(json.to_string());
        ParseCompanyFactsInput::new(json, digest)
    }

    fn test_context() -> ParseCompanyFactsContext {
        let cik = Cik::new(APPLE_CIK).expect("Hardcoded Apple CIK should always be valid");
        ParseCompanyFactsContext::new(cik)
    }

    fn input_of(json: serde_json::Value) -> ParseCompanyFactsInput {
        let digest = Sha256Digest::of(json.to_string());
        ParseCompanyFactsInput::new(json, digest)
    }

    fn get_baseline_parse_state() -> ParseCompanyFacts {
        let input = test_input();
        let context = test_context();
        ParseCompanyFacts::new(input, context)
    }

    /// Creates a minimal valid SEC Company Facts JSON for testing.
    ///
    /// It reports Net Income and Equity only by their parts, `NetIncomeLoss` and
    /// `StockholdersEquity`, so the required check passes only if it counts parts.
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

        let expected_result = &test_context();

        let result = parse_state.context_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_produce_output_with_correct_entity_name_when_computing_valid_json() {
        let json = get_baseline_company_facts_json();
        let digest = Sha256Digest::of(json.to_string());
        let input = ParseCompanyFactsInput::new(json, digest);
        let context = test_context();
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
            .as_str();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_every_fact_when_computing_valid_json() {
        let input = input_of(get_baseline_company_facts_json());
        let mut parse_state = ParseCompanyFacts::new(input, test_context());

        let expected_result = 11;

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
    async fn should_keep_one_filing_per_accession_number_when_computing_valid_json() {
        let input = input_of(get_baseline_company_facts_json());
        let mut parse_state = ParseCompanyFacts::new(input, test_context());

        let expected_result = 1;

        parse_state
            .compute_output_data_async()
            .await
            .expect("Valid complete JSON should always parse successfully");
        let result = parse_state
            .output_data()
            .expect("Output should be present after successful computation")
            .company_data()
            .filings()
            .len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_incomplete_company_facts_when_no_required_concept_is_reported() {
        let json = serde_json::json!({
            "cik": 320_193,
            "entityName": "Apple Inc.",
            "facts": {
                "us-gaap": {},
                "dei": {}
            }
        });
        let mut parse_state = ParseCompanyFacts::new(input_of(json), test_context());

        let expected_result = Some(StateError::IncompleteCompanyFacts(
            IncompleteCompanyFacts::new(
                "Parse Company Facts",
                MissingFields::new(vec![
                    "Revenue".to_string(),
                    "Operating Income".to_string(),
                    "Net Income".to_string(),
                    "Assets".to_string(),
                    "Liabilities".to_string(),
                    "Equity".to_string(),
                    "Operating Cash Flow".to_string(),
                    "Investing Cash Flow".to_string(),
                    "Financing Cash Flow".to_string(),
                ]),
            ),
        ));

        let result = parse_state.compute_output_data_async().await.err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_incomplete_company_facts_when_required_concept_is_outside_us_gaap() {
        let mut json = get_baseline_company_facts_json();
        let assets = json["facts"]["us-gaap"]
            .as_object_mut()
            .expect("The baseline JSON should always hold a us-gaap object")
            .remove("Assets")
            .expect("The baseline JSON should always hold an Assets concept");
        json["facts"]["dei"]["Assets"] = assets;
        let mut parse_state = ParseCompanyFacts::new(input_of(json), test_context());

        let expected_result = Some(StateError::IncompleteCompanyFacts(
            IncompleteCompanyFacts::new(
                "Parse Company Facts",
                MissingFields::new(vec!["Assets".to_string()]),
            ),
        ));

        let result = parse_state.compute_output_data_async().await.err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unexpected_cik_when_facts_are_about_another_company() {
        let input = input_of(get_baseline_company_facts_json());
        let berkshire = Cik::new(BERKSHIRE_HATHAWAY_CIK)
            .expect("Hardcoded Berkshire Hathaway CIK should always be valid");
        let mut parse_state =
            ParseCompanyFacts::new(input, ParseCompanyFactsContext::new(berkshire.clone()));

        let expected_result = Some(StateError::UnexpectedCik(UnexpectedCik::new(
            "Parse Company Facts",
            berkshire,
            APPLE_CIK,
        )));

        let result = parse_state.compute_output_data_async().await.err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_change_input_data_when_computing_output_data() {
        let input = input_of(get_baseline_company_facts_json());
        let mut parse_state = ParseCompanyFacts::new(input, test_context());

        let expected_result = &parse_state.input_data().clone();

        parse_state
            .compute_output_data_async()
            .await
            .expect("Valid complete JSON should always parse successfully");
        let result = parse_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_company_facts_parse_when_response_is_not_an_object() {
        let input = input_of(serde_json::json!("not an object"));
        let mut parse_state = ParseCompanyFacts::new(input, test_context());

        let expected_result = Some(StateError::FailedCompanyFactsParse(
            FailedCompanyFactsParse::new(
                "Parse Company Facts",
                ParseErrorKind::InvalidJson {
                    reason: "Expected a JSON object at the top level".to_string(),
                },
            ),
        ));

        let result = parse_state.compute_output_data_async().await.err();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_parse_company_facts() {
        implements_auto_traits::<ParseCompanyFacts>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_parse_company_facts() {
        implements_send::<ParseCompanyFacts>();
    }

    #[test]
    const fn should_implement_sync_for_parse_company_facts() {
        implements_sync::<ParseCompanyFacts>();
    }

    #[test]
    const fn should_be_thread_safe_for_parse_company_facts() {
        implements_send::<ParseCompanyFacts>();
        implements_sync::<ParseCompanyFacts>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_parse_company_facts() {
        implements_sized::<ParseCompanyFacts>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_parse_company_facts() {
        implements_hash::<ParseCompanyFacts>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_parse_company_facts() {
        implements_partial_eq::<ParseCompanyFacts>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_parse_company_facts() {
        implements_eq::<ParseCompanyFacts>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_parse_company_facts() {
        implements_partial_ord::<ParseCompanyFacts>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_parse_company_facts() {
        implements_ord::<ParseCompanyFacts>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_parse_company_facts() {
        implements_debug::<ParseCompanyFacts>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_parse_company_facts() {
        implements_clone::<ParseCompanyFacts>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_parse_company_facts() {
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

    #[test]
    fn should_produce_output_when_calling_sync_compute_outside_tokio_runtime() {
        let json = get_baseline_company_facts_json();
        let digest = Sha256Digest::of(json.to_string());
        let input = ParseCompanyFactsInput::new(json, digest);
        let context = test_context();
        let mut state = ParseCompanyFacts::new(input, context);

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let json = get_baseline_company_facts_json();
        let digest = Sha256Digest::of(json.to_string());
        let input = ParseCompanyFactsInput::new(json, digest);
        let context = test_context();
        let mut state = ParseCompanyFacts::new(input, context);

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "compute_output_data failed")]
    fn should_panic_when_calling_sync_compute_with_invalid_input() {
        let mut state = get_baseline_parse_state();

        state.compute_output_data();
    }
}
