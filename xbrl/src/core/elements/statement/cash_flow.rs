//! # Cash Flow Concepts
//!
//! Provides [`CashFlowConcept`], its placements, and [`GROUPS`], the groups whose parent is a
//! cash flow concept.

use crate::core::elements::IncomeStatementConcept;
use crate::core::elements::Placed;
use crate::core::elements::roll_up::{Group, GroupKind, Placement, Weight};

const NET_CHANGE_IN_CASH: &str = "net-change-in-cash";
const OPERATING_CASH_FLOW: &str = "operating-cash-flow";
const INVESTING_CASH_FLOW: &str = "investing-cash-flow";
const FINANCING_CASH_FLOW: &str = "financing-cash-flow";

innermost_concepts! {
    /// A standard concept of the cash flow statement. Each value is a duration.
    pub enum CashFlowConcept in Statement(StatementConcept::CashFlow) {
        /// Net cash from operating activities.
        OperatingCashFlow => "Operating Cash Flow",
        /// Net cash from investing activities.
        InvestingCashFlow => "Investing Cash Flow",
        /// Net cash from financing activities.
        FinancingCashFlow => "Financing Cash Flow",
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

impl Placed for CashFlowConcept {
    fn placement(self) -> Placement {
        match self {
            Self::OperatingCashFlow | Self::InvestingCashFlow | Self::FinancingCashFlow => {
                Placement::child(NET_CHANGE_IN_CASH, Weight::Plus)
            }
            Self::DepreciationAmortization | Self::ShareBasedCompensation => {
                Placement::child(OPERATING_CASH_FLOW, Weight::Plus)
            }
            Self::CapitalExpenditures => Placement::child(INVESTING_CASH_FLOW, Weight::Minus),
            Self::DividendsPaid | Self::ShareRepurchases => {
                Placement::child(FINANCING_CASH_FLOW, Weight::Minus)
            }
        }
    }
}

/// The groups whose parent is a cash flow concept, from the top of
/// the statement down.
///
/// `operating-cash-flow` starts from `NetIncome` of the income statement, because the indirect
/// method starts from net income.
pub const GROUPS: &[Group] = &[
    Group {
        name: OPERATING_CASH_FLOW,
        parent: CashFlowConcept::OperatingCashFlow.element(),
        kind: GroupKind::Step {
            start: IncomeStatementConcept::NetIncome.element(),
        },
        closed: false,
    },
    Group {
        name: INVESTING_CASH_FLOW,
        parent: CashFlowConcept::InvestingCashFlow.element(),
        kind: GroupKind::Breakdown,
        closed: false,
    },
    Group {
        name: FINANCING_CASH_FLOW,
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
            (
                CashFlowConcept::OperatingCashFlow,
                Placement::child("net-change-in-cash", Weight::Plus),
            ),
            (
                CashFlowConcept::InvestingCashFlow,
                Placement::child("net-change-in-cash", Weight::Plus),
            ),
            (
                CashFlowConcept::FinancingCashFlow,
                Placement::child("net-change-in-cash", Weight::Plus),
            ),
            (
                CashFlowConcept::DepreciationAmortization,
                Placement::child("operating-cash-flow", Weight::Plus),
            ),
            (
                CashFlowConcept::ShareBasedCompensation,
                Placement::child("operating-cash-flow", Weight::Plus),
            ),
            (
                CashFlowConcept::CapitalExpenditures,
                Placement::child("investing-cash-flow", Weight::Minus),
            ),
            (
                CashFlowConcept::DividendsPaid,
                Placement::child("financing-cash-flow", Weight::Minus),
            ),
            (
                CashFlowConcept::ShareRepurchases,
                Placement::child("financing-cash-flow", Weight::Minus),
            ),
        ];

        let result: Vec<(CashFlowConcept, Placement)> = concepts
            .iter()
            .map(|concept| (*concept, concept.placement()))
            .collect();

        assert_eq!(result, expected_result);
    }
}
