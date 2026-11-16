//! # Transition: `ParseCompanyFacts` → `CreateFinancialStatements`
//!
//! Implements the [`TryFrom`] conversion producing a [`CreateFinancialStatements`] from a
//! [`ParseCompanyFacts`], the transform-phase transition.
//!
//! It moves the parsed [`CompanyData`](crate::shared::company_data::CompanyData) from the
//! source state's output into the next state's input, and carries the CIK across via context. The
//! supporting `From` impls perform the field-level conversions; the `TryFrom` is fallible because
//! the source output may be absent.
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output to carry forward.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::transform::create_financial_statements::constants::STATE_NAME as CREATE_FINANCIAL_STATEMENTS;
use crate::implementations::states::transform::create_financial_statements::{
    CreateFinancialStatements, CreateFinancialStatementsContext, CreateFinancialStatementsInput,
};
use crate::implementations::states::transform::parse_company_facts::constants::STATE_NAME as PARSE_COMPANY_FACTS;
use crate::implementations::states::transform::parse_company_facts::{
    ParseCompanyFacts, ParseCompanyFactsContext, ParseCompanyFactsOutput,
};

impl From<ParseCompanyFactsOutput> for CreateFinancialStatementsInput {
    fn from(output_data: ParseCompanyFactsOutput) -> Self {
        Self::new(output_data.company_data)
    }
}

impl From<ParseCompanyFactsContext> for CreateFinancialStatementsContext {
    fn from(context: ParseCompanyFactsContext) -> Self {
        Self::new(context.cik)
    }
}

impl TryFrom<ParseCompanyFacts> for CreateFinancialStatements {
    type Error = TransitionError;

    fn try_from(state: ParseCompanyFacts) -> Result<Self, TransitionError> {
        let (_input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(PARSE_COMPANY_FACTS, CREATE_FINANCIAL_STATEMENTS)
        })?;

        let new_input: CreateFinancialStatementsInput = output_data.into();
        let new_context: CreateFinancialStatementsContext = context.into();

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use storage::shared::Sha256Digest;
    use xbrl::adapter::edgar::company_facts;
    use xbrl::core::entity_name::EntityName;

    use super::*;
    use crate::error::state_machine::transition::MissingOutput;
    use crate::implementations::states::transform::parse_company_facts::ParseCompanyFactsInput;
    use crate::shared::cik::Cik;
    use crate::shared::company_data::CompanyData;

    fn get_baseline_parse_state_without_output() -> ParseCompanyFacts {
        let json: serde_json::Value = serde_json::json!({});
        let digest = Sha256Digest::of(json.to_string());
        let input = ParseCompanyFactsInput::new(json, digest);
        let cik = Cik::new("0001067983").expect("Hardcoded CIK should always be valid");
        let context = ParseCompanyFactsContext::new(cik);
        ParseCompanyFacts::new(input, context)
    }

    fn test_cik() -> Cik {
        Cik::new("0001234567").expect("Hardcoded valid CIK string should always parse successfully")
    }

    fn test_company_data() -> CompanyData {
        CompanyData::new(
            test_cik(),
            EntityName::new("Test Corp."),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn should_convert_context_when_valid_context() {
        let cik = test_cik();
        let context = ParseCompanyFactsContext::new(cik.clone());

        let expected_result = CreateFinancialStatementsContext::new(cik);

        let result = CreateFinancialStatementsContext::from(context);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_output_to_input_when_valid_output() {
        let company_data = test_company_data();
        let output = ParseCompanyFactsOutput::new(company_data.clone());

        let expected_result = CreateFinancialStatementsInput::new(company_data);

        let result = CreateFinancialStatementsInput::from(output);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_carry_the_facts_when_converting_output_to_input() {
        let json = serde_json::json!({
            "cik": 1_234_567,
            "entityName": "Test Corp.",
            "facts": {
                "us-gaap": {
                    "Assets": {
                        "units": {
                            "USD": [{
                                "end": "2023-12-31",
                                "val": 1_000,
                                "accn": "0001234567-24-000001",
                                "fy": 2023,
                                "fp": "FY",
                                "form": "10-K",
                                "filed": "2024-02-01"
                            }]
                        }
                    }
                }
            }
        });
        let parsed = company_facts::parse(&json).expect(
            "Given a hardcoded body with one valid data point, the parse should always succeed",
        );
        let company_data = CompanyData::new(
            test_cik(),
            EntityName::new("Test Corp."),
            parsed.facts,
            parsed.filings,
        );
        let output = ParseCompanyFactsOutput::new(company_data.clone());

        let expected_result = company_data.facts();

        let input = CreateFinancialStatementsInput::from(output);
        let result = input.company_data().facts();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_parse_company_facts_has_no_output() {
        let state = get_baseline_parse_state_without_output();

        let expected_result = Some(TransitionError::MissingOutput(MissingOutput::new(
            "Parse Company Facts",
            "Create Financial Statements",
        )));

        let result = CreateFinancialStatements::try_from(state).err();

        assert_eq!(result, expected_result);
    }
}
