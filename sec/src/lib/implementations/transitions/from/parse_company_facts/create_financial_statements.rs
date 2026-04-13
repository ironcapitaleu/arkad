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
use crate::implementations::states::transform::parse_company_facts::{
    ParseCompanyFacts, ParseCompanyFactsContext, ParseCompanyFactsOutput,
};

use state_maschine::prelude::State;

impl From<ParseCompanyFactsOutput> for CreateFinancialStatementsInput {
    fn from(output_data: ParseCompanyFactsOutput) -> Self {
        Self::new(output_data.company_data().clone())
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
        let output_data = match state.output_data() {
            Some(data) => data.clone(),
            None => {
                return Err(transition::MissingOutput::new(
                    state.state_name().to_string(),
                    CREATE_FINANCIAL_STATEMENTS,
                )
                .into());
            }
        };

        let state_context = state.context_data().clone();
        let new_context: CreateFinancialStatementsContext = state_context.into();
        let new_input: CreateFinancialStatementsInput = output_data.into();

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

        let result: CreateFinancialStatementsContext = context.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_output_to_input_when_valid_output() {
        let company_data = test_company_data();
        let output = ParseCompanyFactsOutput::new(company_data.clone());

        let expected_result = CreateFinancialStatementsInput::new(company_data);

        let result: CreateFinancialStatementsInput = output.into();

        assert_eq!(result, expected_result);
    }
}
