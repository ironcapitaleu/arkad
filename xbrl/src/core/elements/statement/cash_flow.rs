//! # Cash Flow Concepts
//!
//! Provides [`CashFlowConcept`].

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

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_cash_flow_concept() {
        implements_auto_traits::<CashFlowConcept>();
    }
}
