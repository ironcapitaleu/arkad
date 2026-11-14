//! # Balance Sheet Concepts
//!
//! Provides [`BalanceSheetConcept`].

innermost_concepts! {
    /// A standard concept of the balance sheet. Each value is an instant.
    pub enum BalanceSheetConcept in Statement(StatementConcept::BalanceSheet) {
        /// Total assets.
        Assets => "Assets",
        /// Current assets subtotal.
        CurrentAssets => "Current Assets",
        /// Non-current assets subtotal.
        NonCurrentAssets => "Non-Current Assets",
        /// Cash and cash equivalents.
        CashAndEquivalents => "Cash and Equivalents",
        /// Short-term investments.
        ShortTermInvestments => "Short-Term Investments",
        /// Accounts receivable (net).
        AccountsReceivable => "Accounts Receivable",
        /// Inventory (net).
        Inventory => "Inventory",
        /// Property, plant, and equipment (net).
        PropertyPlantEquipment => "Property, Plant & Equipment",
        /// Goodwill.
        Goodwill => "Goodwill",
        /// Intangible assets (net, excluding goodwill).
        IntangibleAssets => "Intangible Assets",
        /// Total liabilities.
        Liabilities => "Liabilities",
        /// Stockholders' equity, including the portion of the noncontrolling interest.
        Equity => "Equity",
        /// Current liabilities subtotal.
        CurrentLiabilities => "Current Liabilities",
        /// Non-current liabilities subtotal.
        NonCurrentLiabilities => "Non-Current Liabilities",
        /// Long-term debt.
        LongTermDebt => "Long-Term Debt",
        /// Retained earnings (accumulated deficit).
        RetainedEarnings => "Retained Earnings",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_balance_sheet_concept() {
        implements_auto_traits::<BalanceSheetConcept>();
    }
}
