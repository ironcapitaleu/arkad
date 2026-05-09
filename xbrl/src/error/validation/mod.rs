//! # Validation Errors
//!
//! Error types for failures during financial statement validation.

use std::fmt;

use thiserror::Error;

use crate::core::elements::CanonicalElement;

pub mod imprecise_rollup;
pub mod incomplete_data;
pub mod inconsistent_identity;

/// Specific validation failure variants.
#[derive(Debug, Error)]
pub enum ValidationErrorKind {
    /// Required canonical elements are missing from the resolved data.
    #[error("[IncompleteData] Missing required elements: {missing_elements}")]
    IncompleteData {
        /// The elements that could not be resolved.
        missing_elements: MissingElements,
    },

    /// A financial identity (invariant) is violated.
    #[error(
        "[InconsistentIdentity] Invariant '{invariant}' violated, left={left_value}, right={right_value}"
    )]
    InconsistentIdentity {
        /// Which invariant was violated.
        invariant: Invariant,
        /// The left-hand side value of the identity.
        left_value: i64,
        /// The right-hand side value of the identity.
        right_value: i64,
    },

    /// A roll-up parent does not match the sum of its children within tolerance.
    #[error(
        "[ImpreciseRollup] Roll-up mismatch for '{parent}', reported={parent_value}, computed={children_sum}, Reason: 'Deviation of {deviation_pct:.4}% exceeds threshold of {threshold_pct:.4}%'"
    )]
    ImpreciseRollup {
        /// The parent element whose total is mismatched.
        parent: CanonicalElement,
        /// The value reported by the company for the parent.
        parent_value: i64,
        /// The sum computed from the children.
        children_sum: i64,
        /// The actual percentage deviation.
        deviation_pct: f64,
        /// The allowed tolerance threshold.
        threshold_pct: f64,
    },
}

/// A list of missing canonical elements with a stable `Display` implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingElements(pub Vec<CanonicalElement>);

impl fmt::Display for MissingElements {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = self
            .0
            .iter()
            .map(|e| format!("\"{e}\""))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "[{formatted}]")
    }
}

/// A financial invariant that must hold across resolved facts.
///
/// Exact invariants must hold with zero tolerance.
/// Non-exact invariants are checked within a configurable threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Invariant {
    /// Assets = Liabilities + Equity.
    BalanceSheetIdentity,
    /// Net Income = Revenue - Expenses + Gains - Losses.
    NetIncomeIdentity,
    /// Comprehensive Income = Net Income + OCI.
    ComprehensiveIncomeIdentity,
    /// Delta Equity = Comprehensive Income + Investments - Distributions.
    EquityChangeIdentity,
    /// Operating CF + Investing CF + Financing CF approximately equals Delta Cash.
    CashFlowReconciliation,
}

impl Invariant {
    /// Returns `true` if this invariant must hold with zero tolerance.
    #[must_use]
    pub const fn is_exact(self) -> bool {
        matches!(
            self,
            Self::BalanceSheetIdentity
                | Self::NetIncomeIdentity
                | Self::ComprehensiveIncomeIdentity
                | Self::EquityChangeIdentity
        )
    }
}

impl fmt::Display for Invariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::BalanceSheetIdentity => "BalanceSheetIdentity",
            Self::NetIncomeIdentity => "NetIncomeIdentity",
            Self::ComprehensiveIncomeIdentity => "ComprehensiveIncomeIdentity",
            Self::EquityChangeIdentity => "EquityChangeIdentity",
            Self::CashFlowReconciliation => "CashFlowReconciliation",
        };
        write!(f, "{name}")
    }
}
