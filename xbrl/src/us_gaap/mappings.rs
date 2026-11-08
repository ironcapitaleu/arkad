//! # US-GAAP Concept Mappings
//!
//! Canonical name constants and concept definition arrays mapping
//! XBRL taxonomy concept names to canonical financial elements.
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
    /// Whether this concept is required (validation fails if absent).
    required: bool,
}

impl ConceptDefinition {
    /// Creates a new [`ConceptDefinition`].
    #[must_use]
    pub const fn new(
        canonical_element: CanonicalElement,
        xbrl_keys: &'static [&'static str],
        required: bool,
    ) -> Self {
        Self {
            canonical_element,
            xbrl_keys,
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

    /// Returns `true` if this concept is required for valid output.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
}

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
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::OperatingIncome,
        &["OperatingIncomeLoss", "IncomeLossFromOperations"],
        true,
    ),
    ConceptDefinition::new(CanonicalElement::NetIncome, &["NetIncomeLoss"], true),
    // Balance Sheet backbone
    ConceptDefinition::new(CanonicalElement::Assets, &["Assets"], true),
    ConceptDefinition::new(CanonicalElement::Liabilities, &["Liabilities"], true),
    ConceptDefinition::new(
        CanonicalElement::Equity,
        &[
            "StockholdersEquity",
            "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
        ],
        true,
    ),
    // Cash Flow backbone
    ConceptDefinition::new(
        CanonicalElement::OperatingCashFlow,
        &["NetCashProvidedByUsedInOperatingActivities"],
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::InvestingCashFlow,
        &["NetCashProvidedByUsedInInvestingActivities"],
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::FinancingCashFlow,
        &["NetCashProvidedByUsedInFinancingActivities"],
        true,
    ),
];

/// Optional concept definitions that enrich financial statements when available.
/// Their absence does not cause validation failure.
pub const OPTIONAL_CONCEPTS: &[ConceptDefinition] = &[];
