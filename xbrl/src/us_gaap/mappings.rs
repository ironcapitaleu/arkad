//! # US-GAAP Concept Mappings
//!
//! Canonical name constants and concept definition arrays mapping
//! XBRL taxonomy concept names to canonical financial elements.
//!
//! ## Two-Tier Validation
//! - **Required**: Structural backbone concepts every public company reports.
//! - **Optional**: Enriching detail extracted when available, skipped when absent.

use crate::core::elements::CanonicalElement;
use crate::core::unit::Unit;

/// Required top-level keys in the SEC Company Facts JSON response.
pub const REQUIRED_TOP_LEVEL_KEYS: &[&str] = &["cik", "entityName", "facts"];

/// Required taxonomy namespace under the `"facts"` key.
pub const REQUIRED_FACTS_NAMESPACE: &str = "us-gaap";

/// Namespace for general company information (DEI taxonomy).
pub const COMPANY_INFO_NAMESPACE: &str = "dei";

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
    /// The expected unit for this concept.
    expected_unit: Unit,
    /// Whether this concept is required (validation fails if absent).
    required: bool,
}

impl ConceptDefinition {
    /// Creates a new [`ConceptDefinition`].
    #[must_use]
    pub const fn new(
        canonical_element: CanonicalElement,
        xbrl_keys: &'static [&'static str],
        expected_unit: Unit,
        required: bool,
    ) -> Self {
        Self {
            canonical_element,
            xbrl_keys,
            expected_unit,
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

    /// Returns the expected unit type.
    #[must_use]
    pub const fn expected_unit(&self) -> Unit {
        self.expected_unit
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
        Unit::Usd,
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::OperatingIncome,
        &["OperatingIncomeLoss", "IncomeLossFromOperations"],
        Unit::Usd,
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::NetIncome,
        &["NetIncomeLoss"],
        Unit::Usd,
        true,
    ),
    // Balance Sheet backbone
    ConceptDefinition::new(CanonicalElement::Assets, &["Assets"], Unit::Usd, true),
    ConceptDefinition::new(
        CanonicalElement::Liabilities,
        &["Liabilities"],
        Unit::Usd,
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::Equity,
        &[
            "StockholdersEquity",
            "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
        ],
        Unit::Usd,
        true,
    ),
    // Cash Flow backbone
    ConceptDefinition::new(
        CanonicalElement::OperatingCashFlow,
        &["NetCashProvidedByUsedInOperatingActivities"],
        Unit::Usd,
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::InvestingCashFlow,
        &["NetCashProvidedByUsedInInvestingActivities"],
        Unit::Usd,
        true,
    ),
    ConceptDefinition::new(
        CanonicalElement::FinancingCashFlow,
        &["NetCashProvidedByUsedInFinancingActivities"],
        Unit::Usd,
        true,
    ),
];

/// Optional concept definitions that enrich financial statements when available.
/// Their absence does not cause validation failure.
pub const OPTIONAL_CONCEPTS: &[ConceptDefinition] = &[];
