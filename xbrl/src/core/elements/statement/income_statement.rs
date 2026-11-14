//! # Income Statement Concepts
//!
//! Provides [`IncomeStatementConcept`], its placements, and [`GROUPS`], the groups whose parent
//! is an income statement concept.
//!
//! The income statement is a cascade of steps from revenue to comprehensive income. Each subtotal
//! is the start of the next step.

use crate::core::elements::Placed;
use crate::core::elements::roll_up::{Group, GroupKind, Placement, Weight};

const GROSS_PROFIT: &str = "gross-profit";
const OPERATING_EXPENSES: &str = "operating-expenses";
const PRETAX_INCOME: &str = "pretax-income";
const COMPREHENSIVE_INCOME: &str = "comprehensive-income";

innermost_concepts! {
    /// A standard concept of the income statement, comprehensive income included. Each value is
    /// a duration.
    pub enum IncomeStatementConcept in Statement(StatementConcept::IncomeStatement) {
        /// Total revenue.
        Revenue => "Revenue",
        /// Cost of goods sold / cost of revenue.
        CostOfRevenue => "Cost of Revenue",
        /// Gross profit.
        GrossProfit => "Gross Profit",
        /// Selling, general, and administrative expense.
        SellingGeneralAdmin => "SG&A",
        /// Research and development expense.
        ResearchDevelopment => "R&D",
        /// Operating income (loss).
        OperatingIncome => "Operating Income",
        /// Interest expense.
        InterestExpense => "Interest Expense",
        /// Net income (loss), including the portion of the noncontrolling interest.
        NetIncome => "Net Income",
        /// Other comprehensive income (unrealized gains/losses, FX, pensions), including the
        /// portion of the noncontrolling interest.
        OtherComprehensiveIncome => "Other Comprehensive Income",
        /// Comprehensive income (net income + OCI), including the portion of the noncontrolling
        /// interest.
        ComprehensiveIncome => "Comprehensive Income",
    }
}

impl Placed for IncomeStatementConcept {
    fn placement(self) -> Placement {
        match self {
            Self::Revenue
            | Self::GrossProfit
            | Self::OperatingIncome
            | Self::NetIncome
            | Self::ComprehensiveIncome => Placement::Top,
            Self::CostOfRevenue => Placement::child(GROSS_PROFIT, Weight::Minus),
            Self::SellingGeneralAdmin | Self::ResearchDevelopment => {
                Placement::child(OPERATING_EXPENSES, Weight::Plus)
            }
            Self::InterestExpense => Placement::child(PRETAX_INCOME, Weight::Minus),
            Self::OtherComprehensiveIncome => Placement::child(COMPREHENSIVE_INCOME, Weight::Plus),
        }
    }
}

/// The groups whose parent is an income statement concept, in the order of their parents in
/// [`Concept::ALL`](crate::core::elements::Concept::ALL).
pub const GROUPS: &[Group] = &[
    Group {
        name: GROSS_PROFIT,
        parent: IncomeStatementConcept::GrossProfit.element(),
        kind: GroupKind::Step {
            start: IncomeStatementConcept::Revenue.element(),
        },
        closed: true,
    },
    Group {
        name: COMPREHENSIVE_INCOME,
        parent: IncomeStatementConcept::ComprehensiveIncome.element(),
        kind: GroupKind::Step {
            start: IncomeStatementConcept::NetIncome.element(),
        },
        closed: true,
    },
];

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::Concept;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_income_statement_concept() {
        implements_auto_traits::<IncomeStatementConcept>();
    }

    #[test]
    fn should_place_every_income_statement_concept_as_the_table_gives_it() {
        let concepts = IncomeStatementConcept::ALL;

        let expected_result = vec![
            (IncomeStatementConcept::Revenue, Placement::Top),
            (
                IncomeStatementConcept::CostOfRevenue,
                Placement::child("gross-profit", Weight::Minus),
            ),
            (IncomeStatementConcept::GrossProfit, Placement::Top),
            (
                IncomeStatementConcept::SellingGeneralAdmin,
                Placement::child("operating-expenses", Weight::Plus),
            ),
            (
                IncomeStatementConcept::ResearchDevelopment,
                Placement::child("operating-expenses", Weight::Plus),
            ),
            (IncomeStatementConcept::OperatingIncome, Placement::Top),
            (
                IncomeStatementConcept::InterestExpense,
                Placement::child("pretax-income", Weight::Minus),
            ),
            (IncomeStatementConcept::NetIncome, Placement::Top),
            (
                IncomeStatementConcept::OtherComprehensiveIncome,
                Placement::child("comprehensive-income", Weight::Plus),
            ),
            (IncomeStatementConcept::ComprehensiveIncome, Placement::Top),
        ];

        let result: Vec<(IncomeStatementConcept, Placement)> = concepts
            .iter()
            .map(|concept| (*concept, concept.placement()))
            .collect();

        assert_eq!(result, expected_result);
    }
}
