//! # Balance Sheet
//!
//! Provides [`BalanceSheet`], the typed view of a [`Statement`] of the balance sheet.
//!
//! `line` takes a [`BalanceSheetConcept`], so a concept of another statement does not compile:
//!
//! ```rust,compile_fail
//! use xbrl::core::elements::IncomeStatementConcept;
//! use xbrl::core::statement::BalanceSheet;
//!
//! fn revenue(balance_sheet: &BalanceSheet) {
//!     let _line = balance_sheet.line(IncomeStatementConcept::Revenue);
//! }
//! ```

use super::{FinancialStatement, Statement, StatementKind, StatementLine};
use crate::core::elements::BalanceSheetConcept;

statement_view! {
    /// A [`Statement`] of the balance sheet. It holds no data of its own.
    pub struct BalanceSheet for BalanceSheet(BalanceSheetConcept);
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::IncomeStatementConcept;
    use crate::core::elements::statement::balance_sheet::GROUPS;
    use crate::core::elements::{CanonicalElementKind, GroupKind};
    use crate::core::statement::IncomeStatement;
    use crate::core::statement::tests::{fact, number};

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_balance_sheet() {
        implements_auto_traits::<BalanceSheet>();
    }

    fn balance_sheet() -> BalanceSheet {
        let facts = [
            fact(BalanceSheetConcept::Assets, "100"),
            fact(BalanceSheetConcept::Goodwill, "20"),
        ];
        let statement = Statement::build(StatementKind::BalanceSheet, &facts)
            .expect("Given two instants of one entity, the build should always succeed");
        BalanceSheet::try_from(statement)
            .expect("Given a statement of the balance sheet, the view should always accept it")
    }

    #[test]
    fn should_give_the_value_of_the_line_when_the_concept_has_a_fact() {
        let balance_sheet = balance_sheet();

        let expected_result = Some(number("20"));

        let result = balance_sheet
            .line(BalanceSheetConcept::Goodwill)
            .and_then(StatementLine::value)
            .cloned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_line_when_the_parent_has_no_value() {
        let balance_sheet = balance_sheet();

        let expected_result = None;

        let result = balance_sheet.line(BalanceSheetConcept::NonCurrentAssets);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_back_the_statement_when_it_has_another_kind() {
        let facts = [fact(IncomeStatementConcept::Revenue, "100")];
        let statement = Statement::build(StatementKind::IncomeStatement, &facts)
            .expect("Given one duration of revenue, the build should always succeed");

        let expected_result = Err(statement.clone());

        let result = BalanceSheet::try_from(statement);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_back_the_financial_statement_when_it_holds_another_kind() {
        let facts = [fact(IncomeStatementConcept::Revenue, "100")];
        let statement = Statement::build(StatementKind::IncomeStatement, &facts)
            .expect("Given one duration of revenue, the build should always succeed");
        let income_statement = IncomeStatement::try_from(statement)
            .expect("Given a statement of the income statement, the view should always accept it");
        let financial_statement = FinancialStatement::from(income_statement);

        let expected_result = Err(financial_statement.clone());

        let result = BalanceSheet::try_from(financial_statement);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_view_out_when_the_financial_statement_holds_a_balance_sheet() {
        let balance_sheet = balance_sheet();
        let financial_statement = FinancialStatement::from(balance_sheet.clone());

        let expected_result = Ok(balance_sheet);

        let result = BalanceSheet::try_from(financial_statement);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_view_of_its_kind_when_wrapping_a_statement_in_a_financial_statement() {
        let facts = [fact(BalanceSheetConcept::Assets, "100")];
        let statement = Statement::build(StatementKind::BalanceSheet, &facts)
            .expect("Given one fact of the statement, the build should always succeed");

        let expected_result = Ok(BalanceSheet(statement.clone()));

        let result = BalanceSheet::try_from(FinancialStatement::from(statement));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_find_no_start_from_another_statement_in_the_groups_of_the_balance_sheet() {
        let groups = GROUPS;

        let expected_result: Vec<CanonicalElementKind> = Vec::new();

        let result: Vec<CanonicalElementKind> = groups
            .iter()
            .filter_map(|group| match group.kind {
                GroupKind::Step { start } if BalanceSheetConcept::try_from(start).is_err() => {
                    Some(start)
                }
                _ => None,
            })
            .collect();

        assert_eq!(result, expected_result);
    }
}
