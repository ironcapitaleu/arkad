//! # Reference Data
//!
//! Provides the tables a plug-in hands to the core: [`ConceptAliases`], [`PartAliases`],
//! [`CalculationTree`], and [`ElementReference`].
//!
//! The types are records. A plug-in fills them from static data compiled into the binary, so every
//! field is public.

use std::fmt;

use bigdecimal::BigDecimal;

use crate::core::concept::ConceptName;
use crate::core::elements::CanonicalElementKind;
use crate::core::release::TaxonomyReleaseId;
use crate::core::rule::RuleId;
use crate::core::scope::Scope;

/// The taxonomy concepts of one standard concept, in priority order, with the ID of the alias rule.
///
/// The first concept that a filing reports wins. A list spans releases, so it bridges renamed
/// concepts, for example `SalesRevenueNet` and `Revenues`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ConceptAliases {
    /// The standard concept.
    pub element: CanonicalElementKind,
    /// The ID of the alias rule, for example `us-gaap/alias/revenue@1`.
    pub rule: RuleId,
    /// The taxonomy concepts in priority order.
    pub concepts: Vec<ConceptName>,
    /// The parts that stand for the whole standard concept when no concept of the list resolves.
    pub parts: Option<PartAliases>,
    /// The scope of each concept of the list or of the parts that covers less than the standard
    /// concept. A concept that is not here covers the full scope.
    pub scopes: Vec<(ConceptName, Scope)>,
}

impl ConceptAliases {
    /// Returns the scope of `concept`, or `None` if it covers the full scope of the standard
    /// concept.
    #[must_use]
    pub fn scope_of(&self, concept: &ConceptName) -> Option<Scope> {
        self.scopes
            .iter()
            .find(|(scoped, _)| scoped == concept)
            .map(|(_, scope)| *scope)
    }
}

/// The parts of a standard concept that stand for the whole if a filing reports no other part.
///
/// A part is a taxonomy concept whose value is one share of the standard concept, for example
/// `StockholdersEquity` for the equity with the noncontrolling interest. If a filing reports the
/// part and none of [`PartAliases::other_parts`] in one period, the part equals the whole. An other
/// part counts in any dimension of the period, and only with a value other than zero.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct PartAliases {
    /// The ID of the part rule, for example `us-gaap/part/equity@1`.
    pub rule: RuleId,
    /// The parts in priority order.
    pub parts: Vec<ConceptName>,
    /// The other parts of the standard concept. If a filing reports one of them in the period, no
    /// part stands for the whole. A part never blocks itself.
    pub other_parts: Vec<ConceptName>,
}

/// One arc of a calculation linkbase: within one role, the parent sums its children, each times its
/// weight.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct CalculationArc {
    /// The role of the calculation link that holds the arc, for example
    /// `http://fasb.org/us-gaap/role/statement/StatementOfIncome`.
    pub role: String,
    /// The total.
    pub parent: ConceptName,
    /// One part of the total.
    pub child: ConceptName,
    /// The weight of the child, usually `1` or `-1`.
    pub weight: BigDecimal,
}

/// The arcs of the calculation linkbases of one release, in all their roles.
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

/// The period type of an element or a value: a point in time or a span of time.
///
/// The taxonomy declares it for each element. A [`Period`](crate::core::period::Period) and a
/// [`StatementKind`](crate::core::statement::StatementKind) each give theirs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum PeriodType {
    /// A point in time.
    Instant,
    /// A span of time.
    Duration,
}

impl fmt::Display for PeriodType {
    /// Writes the value of `periodType` in the XBRL specification: `instant` or `duration`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Instant => "instant",
            Self::Duration => "duration",
        })
    }
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
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn should_write_the_xbrl_name_when_displaying_a_period_type() {
        let period_types = [PeriodType::Instant, PeriodType::Duration];

        let expected_result = vec!["instant".to_string(), "duration".to_string()];

        let result: Vec<String> = period_types.iter().map(ToString::to_string).collect();

        assert_eq!(result, expected_result);
    }

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
