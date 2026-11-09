//! # US-GAAP Concept Mappings
//!
//! Canonical name constants and concept definition arrays mapping
//! XBRL taxonomy concept names to canonical financial elements.
//!
//! Which concepts a list can hold: `xbrl/DESIGN.md`, "`us_gaap`".
//!
//! Each definition names the ID of its alias rule. The rule name is the kebab case of its
//! canonical element, as `xbrl/DESIGN.md`, "`us_gaap`", describes.
//!
//! ## Two-Tier Validation
//! - **Required**: Structural backbone concepts every public company reports.
//! - **Optional**: Enriching detail extracted when available, skipped when absent.

use crate::core::elements::CanonicalElement;

/// A mapping from one or more XBRL concept names to a canonical financial element.
///
/// The `xbrl_keys` array is ordered by priority: the first key that matches
/// in the SEC JSON wins. This handles taxonomy version changes over time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConceptDefinition {
    /// The canonical element this concept maps to.
    canonical_element: CanonicalElement,
    /// XBRL concept names to search for, in priority order.
    xbrl_keys: &'static [&'static str],
    /// The ID of the alias rule, for example `us-gaap/alias/revenue@1`.
    alias_rule: &'static str,
    /// Whether this concept is required (validation fails if absent).
    required: bool,
}

impl ConceptDefinition {
    /// Creates a new [`ConceptDefinition`].
    #[must_use]
    pub const fn new(
        canonical_element: CanonicalElement,
        xbrl_keys: &'static [&'static str],
        alias_rule: &'static str,
        required: bool,
    ) -> Self {
        Self {
            canonical_element,
            xbrl_keys,
            alias_rule,
            required,
        }
    }

    /// Returns the canonical element this definition maps to.
    #[must_use]
    pub const fn canonical_element(&self) -> CanonicalElement {
        self.canonical_element
    }

    /// Returns the XBRL concept keys in priority order.
    #[must_use]
    pub const fn xbrl_keys(&self) -> &'static [&'static str] {
        self.xbrl_keys
    }

    /// Returns the ID of the alias rule, for example `us-gaap/alias/revenue@1`.
    #[must_use]
    pub const fn alias_rule(&self) -> &'static str {
        self.alias_rule
    }

    /// Returns `true` if this concept is required for valid output.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
}

/// The ID of the alias rule of [`CanonicalElement::Revenue`].
pub const REVENUE_RULE: &str = "us-gaap/alias/revenue@1";
/// The ID of the alias rule of [`CanonicalElement::OperatingIncome`].
pub const OPERATING_INCOME_RULE: &str = "us-gaap/alias/operating-income@1";
/// The ID of the alias rule of [`CanonicalElement::NetIncome`].
pub const NET_INCOME_RULE: &str = "us-gaap/alias/net-income@1";
/// The ID of the alias rule of [`CanonicalElement::Assets`].
pub const ASSETS_RULE: &str = "us-gaap/alias/assets@1";
/// The ID of the alias rule of [`CanonicalElement::Liabilities`].
pub const LIABILITIES_RULE: &str = "us-gaap/alias/liabilities@1";
/// The ID of the alias rule of [`CanonicalElement::Equity`].
pub const EQUITY_RULE: &str = "us-gaap/alias/equity@1";
/// The ID of the alias rule of [`CanonicalElement::OperatingCashFlow`].
pub const OPERATING_CASH_FLOW_RULE: &str = "us-gaap/alias/operating-cash-flow@1";
/// The ID of the alias rule of [`CanonicalElement::InvestingCashFlow`].
pub const INVESTING_CASH_FLOW_RULE: &str = "us-gaap/alias/investing-cash-flow@1";
/// The ID of the alias rule of [`CanonicalElement::FinancingCashFlow`].
pub const FINANCING_CASH_FLOW_RULE: &str = "us-gaap/alias/financing-cash-flow@1";

/// Required concept definitions that every public company is expected to report.
/// Validation fails if any of these cannot be resolved in the SEC response.
pub const REQUIRED_CONCEPTS: &[ConceptDefinition] = &[
    // Income Statement backbone
    ConceptDefinition::new(
        CanonicalElement::Revenue,
        &[
            "Revenues",
            "RevenueFromContractWithCustomerExcludingAssessedTax",
            "SalesRevenueNet",
            "RevenueFromContractWithCustomerIncludingAssessedTax",
        ],
        REVENUE_RULE,
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::OperatingIncome,
        &["OperatingIncomeLoss", "IncomeLossFromOperations"],
        OPERATING_INCOME_RULE,
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::NetIncome,
        &["NetIncomeLoss"],
        NET_INCOME_RULE,
        true,
    ),
    // Balance Sheet backbone
    ConceptDefinition::new(CanonicalElement::Assets, &["Assets"], ASSETS_RULE, true),
    ConceptDefinition::new(
        CanonicalElement::Liabilities,
        &["Liabilities"],
        LIABILITIES_RULE,
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::Equity,
        // The total with the noncontrolling interest comes first, because it is the equity of
        // `Assets = Liabilities + Equity`. A filer without a noncontrolling interest reports only
        // `StockholdersEquity`.
        &[
            "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
            "StockholdersEquity",
        ],
        EQUITY_RULE,
        true,
    ),
    // Cash Flow backbone
    ConceptDefinition::new(
        CanonicalElement::OperatingCashFlow,
        &["NetCashProvidedByUsedInOperatingActivities"],
        OPERATING_CASH_FLOW_RULE,
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::InvestingCashFlow,
        &["NetCashProvidedByUsedInInvestingActivities"],
        INVESTING_CASH_FLOW_RULE,
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::FinancingCashFlow,
        &["NetCashProvidedByUsedInFinancingActivities"],
        FINANCING_CASH_FLOW_RULE,
        true,
    ),
];

/// The ID of the alias rule of [`CanonicalElement::CurrentAssets`].
pub const CURRENT_ASSETS_RULE: &str = "us-gaap/alias/current-assets@1";
/// The ID of the alias rule of [`CanonicalElement::NonCurrentAssets`].
pub const NON_CURRENT_ASSETS_RULE: &str = "us-gaap/alias/non-current-assets@1";
/// The ID of the alias rule of [`CanonicalElement::CashAndEquivalents`].
pub const CASH_AND_EQUIVALENTS_RULE: &str = "us-gaap/alias/cash-and-equivalents@1";
/// The ID of the alias rule of [`CanonicalElement::ShortTermInvestments`].
pub const SHORT_TERM_INVESTMENTS_RULE: &str = "us-gaap/alias/short-term-investments@1";
/// The ID of the alias rule of [`CanonicalElement::AccountsReceivable`].
pub const ACCOUNTS_RECEIVABLE_RULE: &str = "us-gaap/alias/accounts-receivable@1";
/// The ID of the alias rule of [`CanonicalElement::Inventory`].
pub const INVENTORY_RULE: &str = "us-gaap/alias/inventory@1";
/// The ID of the alias rule of [`CanonicalElement::PropertyPlantEquipment`].
pub const PROPERTY_PLANT_EQUIPMENT_RULE: &str = "us-gaap/alias/property-plant-equipment@1";
/// The ID of the alias rule of [`CanonicalElement::Goodwill`].
pub const GOODWILL_RULE: &str = "us-gaap/alias/goodwill@1";
/// The ID of the alias rule of [`CanonicalElement::IntangibleAssets`].
pub const INTANGIBLE_ASSETS_RULE: &str = "us-gaap/alias/intangible-assets@1";
/// The ID of the alias rule of [`CanonicalElement::CurrentLiabilities`].
pub const CURRENT_LIABILITIES_RULE: &str = "us-gaap/alias/current-liabilities@1";
/// The ID of the alias rule of [`CanonicalElement::NonCurrentLiabilities`].
pub const NON_CURRENT_LIABILITIES_RULE: &str = "us-gaap/alias/non-current-liabilities@1";
/// The ID of the alias rule of [`CanonicalElement::LongTermDebt`].
pub const LONG_TERM_DEBT_RULE: &str = "us-gaap/alias/long-term-debt@1";
/// The ID of the alias rule of [`CanonicalElement::RetainedEarnings`].
pub const RETAINED_EARNINGS_RULE: &str = "us-gaap/alias/retained-earnings@1";

/// Optional concept definitions that enrich financial statements when available.
/// Their absence does not cause validation failure.
pub const OPTIONAL_CONCEPTS: &[ConceptDefinition] = &[
    // Balance Sheet detail
    ConceptDefinition::new(
        CanonicalElement::CurrentAssets,
        &["AssetsCurrent"],
        CURRENT_ASSETS_RULE,
        false,
    ),
    ConceptDefinition::new(
        CanonicalElement::NonCurrentAssets,
        // Most filers report no noncurrent total. Tier 4, which is not built yet, sums the
        // children of `AssetsNoncurrent` along the calculation tree only if the filing reports
        // every child. So most filings give no value for this concept.
        &["AssetsNoncurrent"],
        NON_CURRENT_ASSETS_RULE,
        false,
    ),
    ConceptDefinition::new(
        CanonicalElement::CashAndEquivalents,
        // `Cash` is not in the list. It is one part of the total, next to the cash equivalents,
        // so a filer that reports both parts without the total gets the part as the total.
        &["CashAndCashEquivalentsAtCarryingValue"],
        CASH_AND_EQUIVALENTS_RULE,
        false,
    ),
    ConceptDefinition::new(
        CanonicalElement::ShortTermInvestments,
        // `MarketableSecuritiesCurrent` is not in the list. It is one part of the total, next to
        // debt securities, equity securities, and other short-term investments.
        &["ShortTermInvestments"],
        SHORT_TERM_INVESTMENTS_RULE,
        false,
    ),
    ConceptDefinition::new(
        CanonicalElement::AccountsReceivable,
        // `ReceivablesNetCurrent` is not in the list. It is wider than the standard concept: it
        // also holds receivables that are not from customers, such as notes and receivables from
        // related parties. A filer that shows one line for all receivables gets no value.
        &["AccountsReceivableNetCurrent"],
        ACCOUNTS_RECEIVABLE_RULE,
        false,
    ),
    ConceptDefinition::new(
        CanonicalElement::Inventory,
        &["InventoryNet"],
        INVENTORY_RULE,
        false,
    ),
    ConceptDefinition::new(
        CanonicalElement::PropertyPlantEquipment,
        // `PropertyPlantAndEquipmentAndFinanceLeaseRightOfUseAssetAfterAccumulatedDepreciationAndAmortization`
        // is not in the list. It is wider than the standard concept: it also holds the right-of-use
        // assets of finance leases. A filer that shows both in one line gets no value.
        &["PropertyPlantAndEquipmentNet"],
        PROPERTY_PLANT_EQUIPMENT_RULE,
        false,
    ),
    ConceptDefinition::new(
        CanonicalElement::Goodwill,
        &["Goodwill"],
        GOODWILL_RULE,
        false,
    ),
    ConceptDefinition::new(
        CanonicalElement::IntangibleAssets,
        // `FiniteLivedIntangibleAssetsNet` is not in the list. It is one part of the total, so a
        // filer that reports it next to `IndefiniteLivedIntangibleAssetsExcludingGoodwill`, without
        // the total, gets the part as the total.
        &["IntangibleAssetsNetExcludingGoodwill"],
        INTANGIBLE_ASSETS_RULE,
        false,
    ),
    ConceptDefinition::new(
        CanonicalElement::CurrentLiabilities,
        &["LiabilitiesCurrent"],
        CURRENT_LIABILITIES_RULE,
        false,
    ),
    ConceptDefinition::new(
        CanonicalElement::NonCurrentLiabilities,
        // Most filers report no noncurrent total. Tier 4, which is not built yet, sums the
        // children of `LiabilitiesNoncurrent` along the calculation tree only if the filing
        // reports every child. So most filings give no value for this concept.
        &["LiabilitiesNoncurrent"],
        NON_CURRENT_LIABILITIES_RULE,
        false,
    ),
    ConceptDefinition::new(
        CanonicalElement::LongTermDebt,
        // The parent is `NonCurrentLiabilities`, so the list holds the noncurrent part only.
        // Two concepts are not in the list, because they are wider than the standard concept.
        // `LongTermDebt` also holds the current maturities. `LongTermDebtAndCapitalLeaseObligations`
        // also holds finance lease obligations. A filer that shows debt and finance leases in one
        // line gets no value.
        &["LongTermDebtNoncurrent"],
        LONG_TERM_DEBT_RULE,
        false,
    ),
    ConceptDefinition::new(
        CanonicalElement::RetainedEarnings,
        &["RetainedEarningsAccumulatedDeficit"],
        RETAINED_EARNINGS_RULE,
        false,
    ),
];
