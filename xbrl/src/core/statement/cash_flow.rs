//! # Cash Flow Statement
//!
//! Provides [`CashFlowStatement`], the typed view of a [`Statement`] of the cash flow statement.

use super::{FinancialStatement, Statement, StatementKind, StatementLine};
use crate::core::elements::{CanonicalElementKind, CashFlowConcept, IncomeStatementConcept};

statement_view! {
    /// A [`Statement`] of the cash flow statement. It holds no data of its own.
    pub struct CashFlowStatement for CashFlow(CashFlowConcept);
}

impl CashFlowStatement {
    /// The starts of the steps of the cash flow statement that belong to another statement, in
    /// the order of the groups. Each start has an accessor, such as
    /// [`CashFlowStatement::net_income`].
    pub const STARTS: &'static [CanonicalElementKind] =
        &[IncomeStatementConcept::NetIncome.element()];

    /// Returns the line of net income, the start of operating cash flow.
    ///
    /// The statement always has this line. If net income has no fact, the line has no value.
    #[must_use]
    pub fn net_income(&self) -> Option<&StatementLine> {
        self.0
            .standard_line(IncomeStatementConcept::NetIncome.element())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::GroupKind;
    use crate::core::elements::statement::cash_flow::GROUPS;
    use crate::core::statement::tests::{fact, number};

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_cash_flow_statement() {
        implements_auto_traits::<CashFlowStatement>();
    }

    #[test]
    fn should_find_net_income_as_the_only_start_from_another_statement_of_the_cash_flow_groups() {
        let groups = GROUPS;

        let expected_result = vec![IncomeStatementConcept::NetIncome.element()];

        let result: Vec<CanonicalElementKind> = groups
            .iter()
            .filter_map(|group| match group.kind {
                GroupKind::Step { start } if CashFlowConcept::try_from(start).is_err() => {
                    Some(start)
                }
                _ => None,
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_net_income_in_starts_for_the_cash_flow_statement() {
        let starts = CashFlowStatement::STARTS;

        let expected_result = vec![IncomeStatementConcept::NetIncome.element()];

        let result = starts.to_vec();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_line_with_no_value_for_net_income_when_the_start_has_no_fact() {
        let facts = [fact(CashFlowConcept::OperatingCashFlow, "390")];
        let statement = Statement::build(StatementKind::CashFlow, &facts)
            .expect("Given one duration of operating cash flow, the build should always succeed");
        let cash_flow_statement = CashFlowStatement::try_from(statement).expect(
            "Given a statement of the cash flow statement, the view should always accept it",
        );

        let expected_result = Some(None);

        let result = cash_flow_statement
            .net_income()
            .map(|line| line.value().cloned());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_value_of_net_income_when_the_start_has_a_fact() {
        let facts = [
            fact(IncomeStatementConcept::NetIncome, "230"),
            fact(CashFlowConcept::OperatingCashFlow, "390"),
        ];
        let statement = Statement::build(StatementKind::CashFlow, &facts)
            .expect("Given two durations of one entity, the build should always succeed");
        let cash_flow_statement = CashFlowStatement::try_from(statement).expect(
            "Given a statement of the cash flow statement, the view should always accept it",
        );

        let expected_result = Some(number("230"));

        let result = cash_flow_statement
            .net_income()
            .and_then(StatementLine::value)
            .cloned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_view_of_its_kind_when_wrapping_a_statement_in_a_financial_statement() {
        let facts = [fact(CashFlowConcept::OperatingCashFlow, "390")];
        let statement = Statement::build(StatementKind::CashFlow, &facts)
            .expect("Given one duration of operating cash flow, the build should always succeed");

        let expected_result = Ok(CashFlowStatement(statement.clone()));

        let result = CashFlowStatement::try_from(FinancialStatement::from(statement));

        assert_eq!(result, expected_result);
    }
}
