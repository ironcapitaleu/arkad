//! # Cash Flow Concepts
//!
//! Provides [`CashFlowConcept`] and its placements. Provides [`CashFlowGroup`] and [`GROUPS`],
//! the groups whose parent is a cash flow concept.

use crate::core::elements::IncomeStatementConcept;
use crate::core::elements::Placed;
use crate::core::elements::roll_up::{Group, GroupKind, Placement, Weight};

innermost_concepts! {
    /// A standard concept of the cash flow statement. Each value is a duration.
    pub enum CashFlowConcept in Statement(StatementConcept::CashFlow) {
        /// The net change in cash, cash equivalents, and restricted cash, including the effect of
        /// exchange rates.
        NetChangeInCash => "Net Change in Cash",
        /// Net cash from operating activities.
        OperatingCashFlow => "Operating Cash Flow",
        /// Net cash from investing activities.
        InvestingCashFlow => "Investing Cash Flow",
        /// Net cash from financing activities.
        FinancingCashFlow => "Financing Cash Flow",
        /// The effect of exchange rates on cash, cash equivalents, and restricted cash, including
        /// the cash of disposal groups and discontinued operations.
        ExchangeRateEffect => "Exchange Rate Effect",
        /// Depreciation, depletion, and amortization: the add-back of the cash flow statement
        /// (`DepreciationDepletionAndAmortization`).
        DepreciationAmortization => "Depreciation & Amortization",
        /// Share-based compensation expense.
        ShareBasedCompensation => "Share-Based Compensation",
        /// Capital expenditures: the payments for every productive asset, intangible assets and
        /// software included (`PaymentsToAcquireProductiveAssets`).
        CapitalExpenditures => "Capital Expenditures",
        /// Dividends paid: every dividend payment, including to preferred stockholders and to
        /// the noncontrolling interest, and capital distributions (`PaymentsOfDividends`).
        DividendsPaid => "Dividends Paid",
        /// Share repurchases: the payments for the repurchase of every class of equity
        /// (`PaymentsForRepurchaseOfEquity`).
        ShareRepurchases => "Share Repurchases",
    }
}

group_names! {
    /// A group whose parent is a cash flow concept.
    pub enum CashFlowGroup in CashFlow {
        /// The group whose parent is [`CashFlowConcept::NetChangeInCash`].
        NetChangeInCash => "net-change-in-cash",
        /// The group whose parent is [`CashFlowConcept::OperatingCashFlow`].
        OperatingCashFlow => "operating-cash-flow",
        /// The group whose parent is [`CashFlowConcept::InvestingCashFlow`].
        InvestingCashFlow => "investing-cash-flow",
        /// The group whose parent is [`CashFlowConcept::FinancingCashFlow`].
        FinancingCashFlow => "financing-cash-flow",
    }
}

impl Placed for CashFlowConcept {
    fn placement(self) -> Placement {
        match self {
            Self::NetChangeInCash => Placement::Top,
            Self::OperatingCashFlow | Self::InvestingCashFlow | Self::FinancingCashFlow => {
                Placement::child(CashFlowGroup::NetChangeInCash.name(), Weight::Plus)
            }
            Self::ExchangeRateEffect => {
                Placement::child_zero_if_absent(CashFlowGroup::NetChangeInCash.name(), Weight::Plus)
            }
            Self::DepreciationAmortization | Self::ShareBasedCompensation => {
                Placement::child(CashFlowGroup::OperatingCashFlow.name(), Weight::Plus)
            }
            Self::CapitalExpenditures => {
                Placement::child(CashFlowGroup::InvestingCashFlow.name(), Weight::Minus)
            }
            Self::DividendsPaid | Self::ShareRepurchases => {
                Placement::child(CashFlowGroup::FinancingCashFlow.name(), Weight::Minus)
            }
        }
    }
}

/// The groups whose parent is a cash flow concept.
///
/// The list follows the order of the first row that names each group in the table of the cash
/// flow statement. `docs/adr/0013-order-of-the-groups.md` records the order.
///
/// `operating-cash-flow` starts from `NetIncome` of the income statement, because the indirect
/// method starts from net income.
pub const GROUPS: &[Group] = &[
    Group {
        name: CashFlowGroup::NetChangeInCash.name(),
        parent: CashFlowConcept::NetChangeInCash.element(),
        kind: GroupKind::Breakdown,
        closed: true,
    },
    Group {
        name: CashFlowGroup::OperatingCashFlow.name(),
        parent: CashFlowConcept::OperatingCashFlow.element(),
        kind: GroupKind::Step {
            start: IncomeStatementConcept::NetIncome.element(),
        },
        closed: false,
    },
    Group {
        name: CashFlowGroup::InvestingCashFlow.name(),
        parent: CashFlowConcept::InvestingCashFlow.element(),
        kind: GroupKind::Breakdown,
        closed: false,
    },
    Group {
        name: CashFlowGroup::FinancingCashFlow.name(),
        parent: CashFlowConcept::FinancingCashFlow.element(),
        kind: GroupKind::Breakdown,
        closed: false,
    },
];

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::Concept;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_cash_flow_concept() {
        implements_auto_traits::<CashFlowConcept>();
    }

    #[test]
    fn should_place_every_cash_flow_concept_as_the_table_gives_it() {
        let concepts = CashFlowConcept::ALL;

        let expected_result = vec![
            (CashFlowConcept::NetChangeInCash, Placement::Top),
            (
                CashFlowConcept::OperatingCashFlow,
                Placement::child(CashFlowGroup::NetChangeInCash.name(), Weight::Plus),
            ),
            (
                CashFlowConcept::InvestingCashFlow,
                Placement::child(CashFlowGroup::NetChangeInCash.name(), Weight::Plus),
            ),
            (
                CashFlowConcept::FinancingCashFlow,
                Placement::child(CashFlowGroup::NetChangeInCash.name(), Weight::Plus),
            ),
            (
                CashFlowConcept::ExchangeRateEffect,
                Placement::child_zero_if_absent(
                    CashFlowGroup::NetChangeInCash.name(),
                    Weight::Plus,
                ),
            ),
            (
                CashFlowConcept::DepreciationAmortization,
                Placement::child(CashFlowGroup::OperatingCashFlow.name(), Weight::Plus),
            ),
            (
                CashFlowConcept::ShareBasedCompensation,
                Placement::child(CashFlowGroup::OperatingCashFlow.name(), Weight::Plus),
            ),
            (
                CashFlowConcept::CapitalExpenditures,
                Placement::child(CashFlowGroup::InvestingCashFlow.name(), Weight::Minus),
            ),
            (
                CashFlowConcept::DividendsPaid,
                Placement::child(CashFlowGroup::FinancingCashFlow.name(), Weight::Minus),
            ),
            (
                CashFlowConcept::ShareRepurchases,
                Placement::child(CashFlowGroup::FinancingCashFlow.name(), Weight::Minus),
            ),
        ];

        let result: Vec<(CashFlowConcept, Placement)> = concepts
            .iter()
            .map(|concept| (*concept, concept.placement()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_kebab_case_name_for_every_cash_flow_group() {
        let groups = CashFlowGroup::ALL;

        let expected_result = vec![
            "net-change-in-cash",
            "operating-cash-flow",
            "investing-cash-flow",
            "financing-cash-flow",
        ];

        let result: Vec<&str> = groups.iter().map(|group| group.as_str()).collect();

        assert_eq!(result, expected_result);
    }
}
