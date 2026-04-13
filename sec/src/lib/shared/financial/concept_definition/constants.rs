//! # Concept Definition Constants
//!
//! Canonical name constants and predefined [`ConceptDefinition`] arrays for SEC XBRL concepts.
//!
//! ## Two-Tier Validation
//! - **Required**: Structural backbone concepts every public company reports. Validation fails if missing.
//! - **Optional**: Enriching detail extracted when available, skipped when absent.
//!
//! ## Statement-Agnostic
//! Concepts are not tied to a specific financial statement. A concept like Net Income
//! can appear in both the Income Statement and Cash Flow Statement.

use crate::shared::financial::concept_definition::ConceptDefinition;
use crate::shared::financial::unit::Unit;

// --- Top-Level Structure ---

/// Required top-level keys in the SEC Company Facts JSON response.
pub const REQUIRED_TOP_LEVEL_KEYS: &[&str] = &["cik", "entityName", "facts"];

/// Required taxonomy namespace under the `"facts"` key.
pub const REQUIRED_FACTS_NAMESPACE: &str = "us-gaap";

/// Namespace for general company information (DEI taxonomy).
pub const COMPANY_INFO_NAMESPACE: &str = "dei";

// --- Canonical Names ---

/// Canonical name for total revenue.
pub const REVENUE: &str = "Revenue";

/// Canonical name for operating income or loss.
pub const OPERATING_INCOME: &str = "Operating Income";

/// Canonical name for income tax expense or benefit.
pub const INCOME_TAX: &str = "Income Tax";

/// Canonical name for net income or loss.
pub const NET_INCOME: &str = "Net Income";

/// Canonical name for total assets.
pub const TOTAL_ASSETS: &str = "Total Assets";

/// Canonical name for total liabilities.
pub const TOTAL_LIABILITIES: &str = "Total Liabilities";

/// Canonical name for stockholders' equity.
pub const STOCKHOLDERS_EQUITY: &str = "Stockholders Equity";

/// Canonical name for net cash from operating activities.
pub const CASH_FROM_OPERATIONS: &str = "Cash from Operations";

/// Canonical name for net cash from investing activities.
pub const CASH_FROM_INVESTING: &str = "Cash from Investing";

/// Canonical name for net cash from financing activities.
pub const CASH_FROM_FINANCING: &str = "Cash from Financing";

/// Canonical name for common stock shares outstanding.
pub const SHARES_OUTSTANDING: &str = "Shares Outstanding";

// --- Required Concept Definitions ---

/// Required concept definitions that every public company is expected to report.
/// Validation fails if any of these cannot be resolved in the SEC response.
pub const REQUIRED_CONCEPTS: &[ConceptDefinition] = &[
    // Income Statement backbone
    ConceptDefinition::new(
        REVENUE,
        &[
            "Revenues",
            "RevenueFromContractWithCustomerExcludingAssessedTax",
            "SalesRevenueNet",
            "RevenueFromContractWithCustomerIncludingAssessedTax",
        ],
        Unit::Usd,
        true,
    ),
    ConceptDefinition::new(OPERATING_INCOME, &["OperatingIncomeLoss"], Unit::Usd, true),
    ConceptDefinition::new(INCOME_TAX, &["IncomeTaxExpenseBenefit"], Unit::Usd, true),
    ConceptDefinition::new(NET_INCOME, &["NetIncomeLoss"], Unit::Usd, true),
    // Balance Sheet backbone
    ConceptDefinition::new(TOTAL_ASSETS, &["Assets"], Unit::Usd, true),
    ConceptDefinition::new(TOTAL_LIABILITIES, &["Liabilities"], Unit::Usd, true),
    ConceptDefinition::new(
        STOCKHOLDERS_EQUITY,
        &[
            "StockholdersEquity",
            "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
        ],
        Unit::Usd,
        true,
    ),
    // Cash Flow backbone
    ConceptDefinition::new(
        CASH_FROM_OPERATIONS,
        &["NetCashProvidedByUsedInOperatingActivities"],
        Unit::Usd,
        true,
    ),
    ConceptDefinition::new(
        CASH_FROM_INVESTING,
        &["NetCashProvidedByUsedInInvestingActivities"],
        Unit::Usd,
        true,
    ),
    ConceptDefinition::new(
        CASH_FROM_FINANCING,
        &["NetCashProvidedByUsedInFinancingActivities"],
        Unit::Usd,
        true,
    ),
    // Company Info (dei namespace)
    ConceptDefinition::new(
        SHARES_OUTSTANDING,
        &["EntityCommonStockSharesOutstanding"],
        Unit::Shares,
        true,
    ),
];

/// Optional concept definitions that enrich financial statements when available.
/// Their absence does not cause validation failure.
///
/// To be extended in future iterations with concepts like:
/// - `GrossProfit`, `CostOfGoodsAndServicesSold`
/// - `EarningsPerShareBasic`, `EarningsPerShareDiluted`
/// - `AssetsCurrent`, `LiabilitiesCurrent`
/// - `DepreciationDepletionAndAmortization`
/// - `PaymentsToAcquirePropertyPlantAndEquipment` (`CapEx`)
pub const OPTIONAL_CONCEPTS: &[ConceptDefinition] = &[];
