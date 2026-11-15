//! # Validation Errors
//!
//! Error types for failures during financial statement validation.

use std::fmt;

use bigdecimal::BigDecimal;
use thiserror::Error;

use crate::core::elements::CanonicalElementKind;

pub mod incomplete_data;
pub mod inconsistent_identity;

/// Specific validation failure variants.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum ValidationErrorKind {
    /// Required standard concepts are missing from the resolved data.
    #[error("[IncompleteData] Missing required elements: {missing_elements}")]
    IncompleteData {
        /// The standard concepts that could not be resolved.
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
        left_value: BigDecimal,
        /// The right-hand side value of the identity.
        right_value: BigDecimal,
    },
}

/// A list of missing standard concepts with a stable `Display` implementation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MissingElements(pub Vec<CanonicalElementKind>);

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

/// A financial invariant that must hold across standardized facts.
///
/// Exact invariants must hold with zero tolerance.
/// Non-exact invariants are checked within a configurable threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Invariant {
    /// Assets = Liabilities + Equity.
    BalanceSheetIdentity,
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
                | Self::ComprehensiveIncomeIdentity
                | Self::EquityChangeIdentity
        )
    }
}

impl fmt::Display for Invariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::BalanceSheetIdentity => "BalanceSheetIdentity",
            Self::ComprehensiveIncomeIdentity => "ComprehensiveIncomeIdentity",
            Self::EquityChangeIdentity => "EquityChangeIdentity",
            Self::CashFlowReconciliation => "CashFlowReconciliation",
        };
        write!(f, "{name}")
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hash;
    use std::str::FromStr;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn is_value_type<T: Clone + Eq + Ord + Hash>() {}

    #[test]
    const fn should_be_a_value_type_for_validation_error_kind() {
        is_value_type::<ValidationErrorKind>();
    }

    #[test]
    const fn should_be_a_value_type_for_missing_elements() {
        is_value_type::<MissingElements>();
    }

    #[test]
    fn should_print_the_decimals_when_an_identity_value_has_a_fraction() {
        let left_value = BigDecimal::from_str("500000000.5")
            .expect("Given a hardcoded decimal text, the parse should always succeed");
        let error = ValidationErrorKind::InconsistentIdentity {
            invariant: Invariant::BalanceSheetIdentity,
            left_value,
            right_value: BigDecimal::from(490_000_000),
        };

        let expected_result = "[InconsistentIdentity] Invariant 'BalanceSheetIdentity' violated, left=500000000.5, right=490000000";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    const fn should_be_a_value_type_for_invariant() {
        is_value_type::<Invariant>();
    }
}
