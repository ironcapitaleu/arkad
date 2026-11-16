//! # Check Tolerance
//!
//! Provides the tolerance and the confidence that every check of a series shares:
//! [`exceeds_tolerance`] and [`weakest`].

use bigdecimal::BigDecimal;

use super::confidence::Confidence;
use super::standardized::StandardizedFact;

/// One basis point as a share: 1 in 10,000.
const BASIS_POINTS_DIVISOR: u32 = 10_000;

/// Returns `true` if `amount` lies outside `basis_points` of the largest absolute value of `facts`.
pub(super) fn exceeds_tolerance(
    amount: &BigDecimal,
    facts: &[&StandardizedFact],
    basis_points: u32,
) -> bool {
    let base = facts
        .iter()
        .map(|fact| fact.value.abs())
        .max()
        .unwrap_or_default();
    amount.abs() * BigDecimal::from(BASIS_POINTS_DIVISOR) > base * BigDecimal::from(basis_points)
}

/// Returns the weakest confidence of `facts`, or [`Confidence::Exact`] if `facts` is empty.
pub(super) fn weakest(facts: &[&StandardizedFact]) -> Confidence {
    facts
        .iter()
        .map(|fact| fact.confidence)
        .max()
        .unwrap_or(Confidence::Exact)
}
