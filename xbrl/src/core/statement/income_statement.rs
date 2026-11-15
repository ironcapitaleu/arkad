//! # Income Statement
//!
//! Provides [`IncomeStatement`], the typed view of a [`Statement`] of the income statement.

use super::{FinancialStatement, Statement, StatementKind, StatementLine};
use crate::core::elements::IncomeStatementConcept;

statement_view! {
    /// A [`Statement`] of the income statement, with comprehensive income at its end. It holds no
    /// data of its own.
    pub struct IncomeStatement for IncomeStatement(IncomeStatementConcept);
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::statement::tests::{fact, number};

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_income_statement() {
        implements_auto_traits::<IncomeStatement>();
    }

    #[test]
    fn should_give_the_value_of_the_line_when_the_concept_has_a_fact() {
        let facts = [fact(IncomeStatementConcept::Revenue, "100")];
        let statement = Statement::build(StatementKind::IncomeStatement, &facts)
            .expect("Given one duration of revenue, the build should always succeed");
        let income_statement = IncomeStatement::try_from(statement)
            .expect("Given a statement of the income statement, the view should always accept it");

        let expected_result = Some(number("100"));

        let result = income_statement
            .line(IncomeStatementConcept::Revenue)
            .and_then(StatementLine::value)
            .cloned();

        assert_eq!(result, expected_result);
    }
}
