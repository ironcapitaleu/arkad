//! # Canonical Elements
//!
//! The canonical set of financial elements derived from SFAC 6 (Level 1)
//! and key sub-elements from the US-GAAP taxonomy (Level 2).

use std::fmt;

/// A canonical financial element.
///
/// Level 1 elements are the SFAC 6 root elements connected by accounting invariants.
/// Level 2 elements are standard sub-items that break down the Level 1 totals.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum CanonicalElement {
    // --- SFAC 6 Level 1 (root invariants) ---
    /// Total assets.
    Assets,
    /// Total liabilities.
    Liabilities,
    /// Stockholders' equity.
    Equity,
    /// Total revenue.
    Revenue,
    /// Total expenses.
    Expenses,
    /// Gains (non-operating).
    Gains,
    /// Losses (non-operating).
    Losses,
    /// Net income (loss).
    NetIncome,
    /// Other comprehensive income (unrealized gains/losses, FX, pensions).
    OtherComprehensiveIncome,
    /// Comprehensive income (net income + OCI).
    ComprehensiveIncome,
    /// Net cash from operating activities.
    OperatingCashFlow,
    /// Net cash from investing activities.
    InvestingCashFlow,
    /// Net cash from financing activities.
    FinancingCashFlow,

    // --- Level 2: Balance Sheet sub-elements ---
    /// Current assets subtotal.
    CurrentAssets,
    /// Non-current assets subtotal.
    NonCurrentAssets,
    /// Cash and cash equivalents.
    CashAndEquivalents,
    /// Short-term investments.
    ShortTermInvestments,
    /// Accounts receivable (net).
    AccountsReceivable,
    /// Inventory (net).
    Inventory,
    /// Property, plant, and equipment (net).
    PropertyPlantEquipment,
    /// Goodwill.
    Goodwill,
    /// Intangible assets (net, excluding goodwill).
    IntangibleAssets,
    /// Current liabilities subtotal.
    CurrentLiabilities,
    /// Non-current liabilities subtotal.
    NonCurrentLiabilities,
    /// Long-term debt.
    LongTermDebt,
    /// Retained earnings (accumulated deficit).
    RetainedEarnings,

    // --- Level 2: Income Statement sub-elements ---
    /// Cost of goods sold / cost of revenue.
    CostOfRevenue,
    /// Gross profit.
    GrossProfit,
    /// Selling, general, and administrative expense.
    SellingGeneralAdmin,
    /// Research and development expense.
    ResearchDevelopment,
    /// Interest expense.
    InterestExpense,
    /// Operating income (loss).
    OperatingIncome,
    /// Earnings per share (basic).
    EarningsPerShareBasic,
    /// Earnings per share (diluted).
    EarningsPerShareDiluted,

    // --- Level 2: Cash Flow sub-elements ---
    /// Depreciation and amortization.
    DepreciationAmortization,
    /// Capital expenditures.
    CapitalExpenditures,
    /// Dividends paid.
    DividendsPaid,
    /// Share repurchases (buybacks).
    ShareRepurchases,
    /// Share-based compensation expense.
    ShareBasedCompensation,
}

impl CanonicalElement {
    /// Returns `true` if this element is a Level 1 (SFAC 6) root element.
    #[must_use]
    pub const fn is_level_one(self) -> bool {
        matches!(
            self,
            Self::Assets
                | Self::Liabilities
                | Self::Equity
                | Self::Revenue
                | Self::Expenses
                | Self::Gains
                | Self::Losses
                | Self::NetIncome
                | Self::OtherComprehensiveIncome
                | Self::ComprehensiveIncome
                | Self::OperatingCashFlow
                | Self::InvestingCashFlow
                | Self::FinancingCashFlow
        )
    }
}

impl fmt::Display for CanonicalElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Assets => "Assets",
            Self::Liabilities => "Liabilities",
            Self::Equity => "Equity",
            Self::Revenue => "Revenue",
            Self::Expenses => "Expenses",
            Self::Gains => "Gains",
            Self::Losses => "Losses",
            Self::NetIncome => "Net Income",
            Self::OtherComprehensiveIncome => "Other Comprehensive Income",
            Self::ComprehensiveIncome => "Comprehensive Income",
            Self::OperatingCashFlow => "Operating Cash Flow",
            Self::InvestingCashFlow => "Investing Cash Flow",
            Self::FinancingCashFlow => "Financing Cash Flow",
            Self::CurrentAssets => "Current Assets",
            Self::NonCurrentAssets => "Non-Current Assets",
            Self::CashAndEquivalents => "Cash and Equivalents",
            Self::ShortTermInvestments => "Short-Term Investments",
            Self::AccountsReceivable => "Accounts Receivable",
            Self::Inventory => "Inventory",
            Self::PropertyPlantEquipment => "Property, Plant & Equipment",
            Self::Goodwill => "Goodwill",
            Self::IntangibleAssets => "Intangible Assets",
            Self::CurrentLiabilities => "Current Liabilities",
            Self::NonCurrentLiabilities => "Non-Current Liabilities",
            Self::LongTermDebt => "Long-Term Debt",
            Self::RetainedEarnings => "Retained Earnings",
            Self::CostOfRevenue => "Cost of Revenue",
            Self::GrossProfit => "Gross Profit",
            Self::SellingGeneralAdmin => "SG&A",
            Self::ResearchDevelopment => "R&D",
            Self::InterestExpense => "Interest Expense",
            Self::OperatingIncome => "Operating Income",
            Self::EarningsPerShareBasic => "EPS (Basic)",
            Self::EarningsPerShareDiluted => "EPS (Diluted)",
            Self::DepreciationAmortization => "Depreciation & Amortization",
            Self::CapitalExpenditures => "Capital Expenditures",
            Self::DividendsPaid => "Dividends Paid",
            Self::ShareRepurchases => "Share Repurchases",
            Self::ShareBasedCompensation => "Share-Based Compensation",
        };
        write!(f, "{name}")
    }
}
