//! # Fact Set
//!
//! Provides [`FactSet`], the standardized facts of one entity and one period.

use std::collections::HashMap;

use super::elements::CanonicalElement;
use super::entity_name::EntityName;
use super::period::Period;
use super::standardized::StandardizedFact;

/// The standardized facts of one entity and one period, at most one per standard concept.
///
/// Resolution fills the set, and validation checks it. It is a working set, not a storage type.
#[derive(Debug, Clone)]
pub struct FactSet {
    /// The reporting entity.
    entity: EntityName,
    /// The period these facts cover.
    period: Period,
    /// The standardized facts, keyed by standard concept.
    facts: HashMap<CanonicalElement, StandardizedFact>,
}

impl FactSet {
    /// Creates a new [`FactSet`] for the given entity and period.
    #[must_use]
    pub fn new(entity: EntityName, period: Period) -> Self {
        Self {
            entity,
            period,
            facts: HashMap::new(),
        }
    }

    /// Returns the entity name.
    #[must_use]
    pub const fn entity(&self) -> &EntityName {
        &self.entity
    }

    /// Returns the period.
    #[must_use]
    pub const fn period(&self) -> Period {
        self.period
    }

    /// Returns the standardized facts, keyed by standard concept.
    #[must_use]
    pub const fn facts(&self) -> &HashMap<CanonicalElement, StandardizedFact> {
        &self.facts
    }

    /// Inserts a standardized fact into the set. A fact of the same standard concept is replaced.
    pub fn insert(&mut self, fact: StandardizedFact) {
        self.facts.insert(fact.concept, fact);
    }

    /// Returns the standardized fact of a standard concept, if the set holds one.
    #[must_use]
    pub fn get(&self, element: CanonicalElement) -> Option<&StandardizedFact> {
        self.facts.get(&element)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::confidence::Confidence;
    use crate::core::fact::Dimensions;
    use crate::core::filing::EntityId;
    use crate::core::period::Duration;
    use crate::core::release::{Sha256, TaxonomyRelease, TaxonomyReleaseId};
    use crate::core::rule::RuleId;
    use crate::core::unit::Unit;

    fn fiscal_year_2024() -> Period {
        let end = NaiveDate::from_ymd_opt(2024, 9, 28)
            .expect("Given a hardcoded valid date, the creation should always succeed");
        Period::Duration(Duration::from_end_and_quarters(end, 4))
    }

    fn revenue(value: &str) -> StandardizedFact {
        StandardizedFact {
            entity: EntityId::new("cik", "0000320193")
                .expect("Given a hardcoded scheme and ID, the creation should always succeed"),
            concept: CanonicalElement::Revenue,
            period: fiscal_year_2024(),
            unit: Unit::from_str("USD")
                .expect("Given a hardcoded unit with one measure, the parse should always succeed"),
            dimensions: Dimensions::default(),
            value: BigDecimal::from_str(value)
                .expect("Given a hardcoded decimal text, the parse should always succeed"),
            confidence: Confidence::Exact,
            rule: RuleId::from_str("us-gaap/alias/revenue@1").expect(
                "Given a hardcoded rule ID in the documented form, the parse should always succeed",
            ),
            release: TaxonomyRelease {
                id: TaxonomyReleaseId::new("us-gaap/2026").expect(
                    "Given a hardcoded release ID with a year, the creation should always succeed",
                ),
                checksum: Sha256::new([0; 32]),
            },
            inputs: Vec::new(),
            restated: false,
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_fact_set() {
        implements_auto_traits::<FactSet>();
    }

    #[test]
    fn should_keep_the_later_fact_when_two_facts_have_one_standard_concept() {
        let mut fact_set = FactSet::new(EntityName::new("Apple Inc."), fiscal_year_2024());
        fact_set.insert(revenue("391035000000"));
        let later = revenue("391036000000");

        let expected_result = Some(revenue("391036000000"));

        fact_set.insert(later);
        let result = fact_set.get(CanonicalElement::Revenue).cloned();

        assert_eq!(result, expected_result);
    }
}
