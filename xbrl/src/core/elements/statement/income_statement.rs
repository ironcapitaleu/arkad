//! # Income Statement Concepts
//!
//! Provides [`IncomeStatementConcept`] and its placements. Provides [`IncomeStatementGroup`] and
//! [`GROUPS`], the groups whose parent is an income statement concept.
//!
//! The income statement is a cascade of steps from revenue to comprehensive income. Each subtotal
//! is the start of the next step.

use crate::core::elements::Placed;
use crate::core::elements::roll_up::{Group, GroupKind, Placement, Weight};

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
        /// Total operating expenses: the costs of operations other than the cost of revenue.
        OperatingExpenses => "Operating Expenses",
        /// Selling, general, and administrative expense.
        SellingGeneralAdmin => "SG&A",
        /// Research and development expense.
        ResearchDevelopment => "R&D",
        /// Operating income (loss).
        OperatingIncome => "Operating Income",
        /// Interest expense.
        InterestExpense => "Interest Expense",
        /// Income (loss) from continuing operations before income taxes, including the portion of
        /// the noncontrolling interest.
        PretaxIncome => "Pretax Income",
        /// Income tax expense (benefit) of continuing operations.
        IncomeTaxExpense => "Income Tax Expense",
        /// Net income (loss), including the portion of the noncontrolling interest.
        NetIncome => "Net Income",
        /// The portion of net income (loss) attributable to the parent.
        NetIncomeAttributableToParent => "Net Income Attributable to Parent",
        /// The portion of net income (loss) attributable to the noncontrolling interest.
        NetIncomeAttributableToNoncontrollingInterest =>
            "Net Income Attributable to Noncontrolling Interest",
        /// Other comprehensive income (unrealized gains/losses, FX, pensions), including the
        /// portion of the noncontrolling interest.
        OtherComprehensiveIncome => "Other Comprehensive Income",
        /// Comprehensive income (net income + OCI), including the portion of the noncontrolling
        /// interest.
        ComprehensiveIncome => "Comprehensive Income",
    }
}

group_names! {
    /// A group whose parent is an income statement concept.
    pub enum IncomeStatementGroup in IncomeStatement {
        /// The group whose parent is [`IncomeStatementConcept::GrossProfit`].
        GrossProfit => "gross-profit",
        /// The group whose parent is [`IncomeStatementConcept::OperatingIncome`].
        OperatingIncome => "operating-income",
        /// The group whose parent is [`IncomeStatementConcept::OperatingExpenses`].
        OperatingExpenses => "operating-expenses",
        /// The group whose parent is [`IncomeStatementConcept::PretaxIncome`].
        PretaxIncome => "pretax-income",
        /// The group whose parent is [`IncomeStatementConcept::NetIncome`]: the step from
        /// pretax income.
        NetIncome => "net-income",
        /// The group whose parent is [`IncomeStatementConcept::NetIncome`]: the split between
        /// the parent and the noncontrolling interest.
        NetIncomeAttribution => "net-income-attribution",
        /// The group whose parent is [`IncomeStatementConcept::ComprehensiveIncome`].
        ComprehensiveIncome => "comprehensive-income",
    }
}

impl Placed for IncomeStatementConcept {
    fn placement(self) -> Placement {
        match self {
            Self::Revenue
            | Self::GrossProfit
            | Self::OperatingIncome
            | Self::PretaxIncome
            | Self::NetIncome
            | Self::ComprehensiveIncome => Placement::Top,
            Self::CostOfRevenue => {
                Placement::child(IncomeStatementGroup::GrossProfit.name(), Weight::Minus)
            }
            Self::OperatingExpenses => {
                Placement::child(IncomeStatementGroup::OperatingIncome.name(), Weight::Minus)
            }
            Self::SellingGeneralAdmin | Self::ResearchDevelopment => {
                Placement::child(IncomeStatementGroup::OperatingExpenses.name(), Weight::Plus)
            }
            Self::InterestExpense => {
                Placement::child(IncomeStatementGroup::PretaxIncome.name(), Weight::Minus)
            }
            Self::IncomeTaxExpense => {
                Placement::child(IncomeStatementGroup::NetIncome.name(), Weight::Minus)
            }
            Self::NetIncomeAttributableToParent => Placement::child(
                IncomeStatementGroup::NetIncomeAttribution.name(),
                Weight::Plus,
            ),
            Self::NetIncomeAttributableToNoncontrollingInterest => Placement::child_zero_if_absent(
                IncomeStatementGroup::NetIncomeAttribution.name(),
                Weight::Plus,
            ),
            Self::OtherComprehensiveIncome => Placement::child(
                IncomeStatementGroup::ComprehensiveIncome.name(),
                Weight::Plus,
            ),
        }
    }
}

/// The groups whose parent is an income statement concept.
///
/// The list follows the order of the first row that names each group in the table of the income
/// statement. `docs/adr/0013-order-of-the-groups.md` records the order.
pub const GROUPS: &[Group] = &[
    Group {
        name: IncomeStatementGroup::GrossProfit.name(),
        parent: IncomeStatementConcept::GrossProfit.element(),
        kind: GroupKind::Step {
            start: IncomeStatementConcept::Revenue.element(),
        },
        closed: true,
    },
    Group {
        name: IncomeStatementGroup::OperatingIncome.name(),
        parent: IncomeStatementConcept::OperatingIncome.element(),
        kind: GroupKind::Step {
            start: IncomeStatementConcept::GrossProfit.element(),
        },
        closed: false,
    },
    Group {
        name: IncomeStatementGroup::OperatingExpenses.name(),
        parent: IncomeStatementConcept::OperatingExpenses.element(),
        kind: GroupKind::Breakdown,
        closed: false,
    },
    Group {
        name: IncomeStatementGroup::PretaxIncome.name(),
        parent: IncomeStatementConcept::PretaxIncome.element(),
        kind: GroupKind::Step {
            start: IncomeStatementConcept::OperatingIncome.element(),
        },
        closed: false,
    },
    Group {
        name: IncomeStatementGroup::NetIncome.name(),
        parent: IncomeStatementConcept::NetIncome.element(),
        kind: GroupKind::Step {
            start: IncomeStatementConcept::PretaxIncome.element(),
        },
        closed: false,
    },
    Group {
        name: IncomeStatementGroup::NetIncomeAttribution.name(),
        parent: IncomeStatementConcept::NetIncome.element(),
        kind: GroupKind::Breakdown,
        closed: true,
    },
    Group {
        name: IncomeStatementGroup::ComprehensiveIncome.name(),
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
                Placement::child(IncomeStatementGroup::GrossProfit.name(), Weight::Minus),
            ),
            (IncomeStatementConcept::GrossProfit, Placement::Top),
            (
                IncomeStatementConcept::OperatingExpenses,
                Placement::child(IncomeStatementGroup::OperatingIncome.name(), Weight::Minus),
            ),
            (
                IncomeStatementConcept::SellingGeneralAdmin,
                Placement::child(IncomeStatementGroup::OperatingExpenses.name(), Weight::Plus),
            ),
            (
                IncomeStatementConcept::ResearchDevelopment,
                Placement::child(IncomeStatementGroup::OperatingExpenses.name(), Weight::Plus),
            ),
            (IncomeStatementConcept::OperatingIncome, Placement::Top),
            (
                IncomeStatementConcept::InterestExpense,
                Placement::child(IncomeStatementGroup::PretaxIncome.name(), Weight::Minus),
            ),
            (IncomeStatementConcept::PretaxIncome, Placement::Top),
            (
                IncomeStatementConcept::IncomeTaxExpense,
                Placement::child(IncomeStatementGroup::NetIncome.name(), Weight::Minus),
            ),
            (IncomeStatementConcept::NetIncome, Placement::Top),
            (
                IncomeStatementConcept::NetIncomeAttributableToParent,
                Placement::child(
                    IncomeStatementGroup::NetIncomeAttribution.name(),
                    Weight::Plus,
                ),
            ),
            (
                IncomeStatementConcept::NetIncomeAttributableToNoncontrollingInterest,
                Placement::child_zero_if_absent(
                    IncomeStatementGroup::NetIncomeAttribution.name(),
                    Weight::Plus,
                ),
            ),
            (
                IncomeStatementConcept::OtherComprehensiveIncome,
                Placement::child(
                    IncomeStatementGroup::ComprehensiveIncome.name(),
                    Weight::Plus,
                ),
            ),
            (IncomeStatementConcept::ComprehensiveIncome, Placement::Top),
        ];

        let result: Vec<(IncomeStatementConcept, Placement)> = concepts
            .iter()
            .map(|concept| (*concept, concept.placement()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_kebab_case_name_for_every_income_statement_group() {
        let groups = IncomeStatementGroup::ALL;

        let expected_result = vec![
            "gross-profit",
            "operating-income",
            "operating-expenses",
            "pretax-income",
            "net-income",
            "net-income-attribution",
            "comprehensive-income",
        ];

        let result: Vec<&str> = groups.iter().map(|group| group.as_str()).collect();

        assert_eq!(result, expected_result);
    }
}
