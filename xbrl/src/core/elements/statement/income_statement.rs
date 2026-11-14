//! # Income Statement Concepts
//!
//! Provides [`IncomeStatementConcept`].
//!
//! The income statement is a cascade of steps from revenue to comprehensive income. Each subtotal
//! is the start of the next step.

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

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_income_statement_concept() {
        implements_auto_traits::<IncomeStatementConcept>();
    }
}
