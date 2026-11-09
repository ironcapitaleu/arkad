//! # US-GAAP Concept Mappings
//!
//! Canonical name constants and concept definition arrays mapping
//! XBRL taxonomy concept names to canonical financial elements.
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

/// Optional concept definitions that enrich financial statements when available.
/// Their absence does not cause validation failure.
pub const OPTIONAL_CONCEPTS: &[ConceptDefinition] = &[];
