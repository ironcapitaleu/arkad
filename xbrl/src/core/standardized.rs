//! # Standardized Fact
//!
//! Provides [`StandardizedFact`], one value of one standard concept, with the rule and the as-is
//! facts it came from.

use bigdecimal::BigDecimal;

use super::confidence::Confidence;
use super::elements::CanonicalElement;
use super::fact::{Dimensions, FactKey};
use super::filing::EntityId;
use super::period::Period;
use super::release::TaxonomyRelease;
use super::rule::RuleId;
use super::unit::Unit;

/// One value of one standard concept for one entity and one period.
///
/// Resolution and dedup produce it from as-is facts. `rule` and `inputs` trace the value back to
/// the [`AsIsFact`](super::fact::AsIsFact) values it came from, and each input key names its filing.
///
/// Every field is public, because each field type checks its own rules, and the rules that
/// produce the fact decide which values belong together.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct StandardizedFact {
    /// The entity the value is about.
    pub entity: EntityId,
    /// The standard concept, for example [`CanonicalElement::Revenue`].
    pub concept: CanonicalElement,
    /// The instant or duration the value covers.
    pub period: Period,
    /// The unit, for example `USD`.
    pub unit: Unit,
    /// The axis and member pairs. Empty in the MVP.
    pub dimensions: Dimensions,
    /// The value.
    pub value: BigDecimal,
    /// The tier that produced the value.
    pub confidence: Confidence,
    /// The rule that produced the value, for example `us-gaap/alias/revenue@1`.
    pub rule: RuleId,
    /// The release of the plug-in that resolved the concept. A core rule carries the release of
    /// its inputs.
    pub release: TaxonomyRelease,
    /// The value keys of the as-is facts the value came from.
    pub inputs: Vec<FactKey>,
    /// `true` if an earlier filing reported another value for this value or for one of its inputs.
    pub restated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_standardized_fact() {
        implements_auto_traits::<StandardizedFact>();
    }
}
