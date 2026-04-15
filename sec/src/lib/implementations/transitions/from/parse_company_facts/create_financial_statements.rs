//! # Transition: `ParseCompanyFacts` → `CreateFinancialStatements`
//!
//! This module implements the state transition from [`ParseCompanyFacts`] to [`CreateFinancialStatements`].
//! The transition extracts the parsed company data from the source state and initializes the next
//! state for creating financial statements.
//!
//! ## Transition Flow
//! 1. Extracts output data (company data) from the source [`ParseCompanyFacts`] state
//! 2. Converts the source context to [`CreateFinancialStatementsContext`]
//! 3. Converts the output to [`CreateFinancialStatementsInput`]
//! 4. Constructs and returns a new [`CreateFinancialStatements`] state
//!
//! ## Error Handling
//! Returns a [`TransitionError`] if the source state lacks required output data.

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
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::financial::company_data::CompanyData;
    use crate::shared::financial::entity_name::EntityName;

    fn test_cik() -> Cik {
        Cik::new("0001234567").expect("Hardcoded valid CIK string should always parse successfully")
    }

    fn test_company_data() -> CompanyData {
        CompanyData::new(test_cik(), EntityName::new("Test Corp."), HashMap::new())
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
    fn should_return_error_when_parse_company_facts_has_no_output() {
        let state = ParseCompanyFacts::default();

        let expected_result = true;

        let result = CreateFinancialStatements::try_from(state).is_err();

        assert_eq!(result, expected_result);
    }
}
