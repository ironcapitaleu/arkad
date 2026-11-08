//! # Reference Data
//!
//! Provides the tables a plug-in hands to the core: [`ConceptAliases`], [`CalculationTree`], and
//! [`ElementReference`].
//!
//! The types are records. A plug-in fills them from static data compiled into the binary, so every
//! field is public.

use bigdecimal::BigDecimal;

use crate::core::concept::ConceptName;
use crate::core::elements::CanonicalElement;
use crate::core::release::TaxonomyReleaseId;
use crate::core::rule::RuleId;

/// The taxonomy concepts of one standard concept, in priority order, with the ID of the alias rule.
///
/// The first concept that a filing reports wins. A list spans releases, so it bridges renamed
/// concepts, for example `SalesRevenueNet` and `Revenues`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ConceptAliases {
    /// The standard concept.
    pub element: CanonicalElement,
    /// The ID of the alias rule, for example `us-gaap/alias/revenue@1`.
    pub rule: RuleId,
    /// The taxonomy concepts in priority order.
    pub concepts: Vec<ConceptName>,
}

/// One arc of a calculation linkbase: the parent sums its children, each times its weight.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct CalculationArc {
    /// The total.
    pub parent: ConceptName,
    /// One part of the total.
    pub child: ConceptName,
    /// The weight of the child, usually `1` or `-1`.
    pub weight: BigDecimal,
}

/// The arcs of the calculation linkbase of one release.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize)]
pub struct CalculationTree {
    /// Every arc of the tree.
    pub arcs: Vec<CalculationArc>,
}

/// The balance type of a monetary element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Balance {
    /// A debit balance, for example an asset or an expense.
    Debit,
    /// A credit balance, for example a liability or a revenue.
    Credit,
}

/// The period type that the taxonomy declares for an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum PeriodType {
    /// The element is reported at a point in time.
    Instant,
    /// The element is reported over a span of time.
    Duration,
}

/// The reference data of one element of a release, or of a retired element.
///
/// The record names the release it comes from, because a retired element takes its data from the
/// last release that defined it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ElementReference {
    /// The element.
    pub concept: ConceptName,
    /// The release the data comes from.
    pub release: TaxonomyReleaseId,
    /// The English standard label.
    pub label: String,
    /// The data type, for example `xbrli:monetaryItemType`.
    pub data_type: String,
    /// The balance type. `None` for an element that is not monetary.
    pub balance: Option<Balance>,
    /// The period type.
    pub period_type: PeriodType,
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_concept_aliases() {
        implements_auto_traits::<ConceptAliases>();
    }

    #[test]
    const fn should_implement_auto_traits_for_calculation_tree() {
        implements_auto_traits::<CalculationTree>();
    }

    #[test]
    const fn should_implement_auto_traits_for_element_reference() {
        implements_auto_traits::<ElementReference>();
    }
}
