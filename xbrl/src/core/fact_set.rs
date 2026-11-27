//! # Fact Set
//!
//! Provides [`FactSet`], the standardized facts of one entity and one period, and
//! [`FactSetError`], the error of [`FactSet::insert`] for a fact that does not belong to the set.

use std::collections::HashMap;

use thiserror::Error;

use super::elements::CanonicalElementKind;
use super::fact::Dimensions;
use super::filing::EntityId;
use super::period::Period;
use super::standardized::StandardizedFact;

/// The standardized facts of one entity and one period, at most one per standard concept.
///
/// Resolution fills the set, and validation checks it. It is a working set, not a storage type.
/// Every fact of the set has the entity and the period of the set, and no dimensions.
#[derive(Debug, Clone)]
pub struct FactSet {
    /// The reporting entity.
    entity: EntityId,
    /// The period these facts cover.
    period: Period,
    /// The standardized facts, keyed by standard concept.
    facts: HashMap<CanonicalElementKind, StandardizedFact>,
}

impl FactSet {
    /// Creates a new [`FactSet`] for the given entity and period.
    #[must_use]
    pub fn new(entity: EntityId, period: Period) -> Self {
        Self {
            entity,
            period,
            facts: HashMap::new(),
        }
    }

    /// Returns the entity ID.
    #[must_use]
    pub const fn entity(&self) -> &EntityId {
        &self.entity
    }

    /// Returns the period.
    #[must_use]
    pub const fn period(&self) -> Period {
        self.period
    }

    /// Returns the standardized facts, keyed by standard concept.
    #[must_use]
    pub const fn facts(&self) -> &HashMap<CanonicalElementKind, StandardizedFact> {
        &self.facts
    }

    /// Inserts a standardized fact into the set. A fact of the same standard concept is replaced.
    ///
    /// # Errors
    ///
    /// Returns a [`FactSetError`] and leaves the set unchanged if the fact does not belong to the
    /// set. The checks run in this order:
    ///
    /// - [`FactSetError::UnexpectedEntity`] if the entity of the fact is not the one of the set.
    /// - [`FactSetError::UnexpectedPeriod`] if the period of the fact is not the one of the set. Two
    ///   periods are the same if they are of one kind and have the same start and end. So an
    ///   instant fact never belongs to a set of a duration.
    /// - [`FactSetError::UnexpectedDimensions`] if the fact has dimensions.
    pub fn insert(&mut self, fact: StandardizedFact) -> Result<(), FactSetError> {
        if fact.entity != self.entity {
            return Err(FactSetError::UnexpectedEntity {
                concept: fact.concept,
                expected: self.entity.clone(),
                found: fact.entity,
            });
        }
        if fact.period != self.period {
            return Err(FactSetError::UnexpectedPeriod {
                concept: fact.concept,
                expected: self.period,
                found: fact.period,
            });
        }
        if !fact.dimensions.is_empty() {
            return Err(FactSetError::UnexpectedDimensions {
                concept: fact.concept,
                dimensions: fact.dimensions,
            });
        }
        self.facts.insert(fact.concept, fact);
        Ok(())
    }

    /// Returns the standardized fact of a standard concept, if the set holds one.
    #[must_use]
    pub fn get(&self, element: CanonicalElementKind) -> Option<&StandardizedFact> {
        self.facts.get(&element)
    }
}

/// The error of [`FactSet::insert`] when a fact does not belong to the set.
///
/// It is a leaf error. It names the standard concept of the fact and the field that does not match.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FactSetError {
    /// The entity of the fact is not the entity of the set.
    #[error(
        "[UnexpectedEntity] Fact of '{concept}' has entity '{} {}', the set holds entity '{} {}'",
        .found.scheme(), .found.id(), .expected.scheme(), .expected.id()
    )]
    UnexpectedEntity {
        /// The standard concept of the fact.
        concept: CanonicalElementKind,
        /// The entity of the set.
        expected: EntityId,
        /// The entity of the fact.
        found: EntityId,
    },

    /// The period of the fact is not the period of the set.
    #[error(
        "[UnexpectedPeriod] Fact of '{concept}' has period '{found}', the set holds period '{expected}'"
    )]
    UnexpectedPeriod {
        /// The standard concept of the fact.
        concept: CanonicalElementKind,
        /// The period of the set.
        expected: Period,
        /// The period of the fact.
        found: Period,
    },

    /// The fact has dimensions, and a fact of the set has none.
    #[error(
        "[UnexpectedDimensions] Fact of '{concept}' has dimensions '{}', the set holds facts without dimensions",
        axis_member_pairs(.dimensions)
    )]
    UnexpectedDimensions {
        /// The standard concept of the fact.
        concept: CanonicalElementKind,
        /// The dimensions of the fact.
        dimensions: Dimensions,
    },
}

/// Writes the pairs of `dimensions` as `axis=member`, sorted by axis and separated by `, `.
fn axis_member_pairs(dimensions: &Dimensions) -> String {
    dimensions
        .iter()
        .map(|(axis, member)| format!("{axis}={member}"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;
    use jiff::civil::date;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::confidence::Confidence;
    use crate::core::elements::IncomeStatementConcept;
    use crate::core::period::Duration;
    use crate::core::release::{Sha256, TaxonomyRelease, TaxonomyReleaseId};
    use crate::core::rule::RuleId;
    use crate::core::unit::Unit;

    fn apple() -> EntityId {
        EntityId::new("http://www.sec.gov/CIK", "0000320193")
            .expect("Given a hardcoded scheme and ID, the creation should always succeed")
    }

    fn fiscal_year_2024() -> Period {
        fiscal_year_2024_with_start(1)
    }

    fn fiscal_year_2023() -> Period {
        let start = date(2022, 9, 25);
        let end = date(2023, 9, 30);
        Period::Duration(
            Duration::from_dates(start, end)
                .expect("Given an end after the start, the creation should always succeed"),
        )
    }

    fn fiscal_year_2024_with_start(start_day: i8) -> Period {
        let start = date(2023, 10, start_day);
        let end = date(2024, 9, 28);
        Period::Duration(
            Duration::from_dates(start, end)
                .expect("Given an end after the start, the creation should always succeed"),
        )
    }

    fn end_of_fiscal_year_2024() -> Period {
        let end = date(2024, 9, 28);
        Period::Instant { end }
    }

    fn revenue(value: &str) -> StandardizedFact {
        StandardizedFact {
            entity: apple(),
            concept: IncomeStatementConcept::Revenue.element(),
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

    fn microsoft() -> EntityId {
        EntityId::new("http://www.sec.gov/CIK", "0000789019")
            .expect("Given a hardcoded scheme and ID, the creation should always succeed")
    }

    fn americas_segment() -> Dimensions {
        Dimensions::new([("StatementBusinessSegmentsAxis", "AmericasSegmentMember")])
            .expect("Given one hardcoded axis and member pair, the creation should always succeed")
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_fact_set() {
        implements_auto_traits::<FactSet>();
    }

    #[test]
    fn should_keep_the_later_fact_when_two_facts_have_one_standard_concept() {
        let mut fact_set = FactSet::new(apple(), fiscal_year_2024());
        fact_set
            .insert(revenue("391035000000"))
            .expect("Given a fact of the entity and the period of the set, the insert should always succeed");
        let later = revenue("391036000000");

        let expected_result = Some(revenue("391036000000"));

        fact_set
            .insert(later)
            .expect("Given a fact of the entity and the period of the set, the insert should always succeed");
        let result = fact_set
            .get(IncomeStatementConcept::Revenue.element())
            .cloned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_insert_the_fact_when_it_has_the_entity_and_period_of_the_set_and_no_dimensions() {
        let mut fact_set = FactSet::new(apple(), fiscal_year_2024());
        let fact = revenue("391035000000");

        let expected_result = Ok(());

        let result = fact_set.insert(fact);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_the_fact_when_the_insert_succeeds() {
        let mut fact_set = FactSet::new(apple(), fiscal_year_2024());
        let fact = revenue("391035000000");

        let expected_result = Some(revenue("391035000000"));

        fact_set
            .insert(fact)
            .expect("Given a fact of the entity and the period of the set, the insert should always succeed");
        let result = fact_set
            .get(IncomeStatementConcept::Revenue.element())
            .cloned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_fact_when_its_entity_is_not_the_entity_of_the_set() {
        let mut fact_set = FactSet::new(apple(), fiscal_year_2024());
        let fact = StandardizedFact {
            entity: microsoft(),
            ..revenue("391035000000")
        };

        let expected_result = Err(FactSetError::UnexpectedEntity {
            concept: IncomeStatementConcept::Revenue.element(),
            expected: apple(),
            found: microsoft(),
        });

        let result = fact_set.insert(fact);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_fact_when_its_period_is_not_the_period_of_the_set() {
        let mut fact_set = FactSet::new(apple(), fiscal_year_2024());
        let fact = StandardizedFact {
            period: fiscal_year_2023(),
            ..revenue("383285000000")
        };

        let expected_result = Err(FactSetError::UnexpectedPeriod {
            concept: IncomeStatementConcept::Revenue.element(),
            expected: fiscal_year_2024(),
            found: fiscal_year_2023(),
        });

        let result = fact_set.insert(fact);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_fact_when_it_has_dimensions() {
        let mut fact_set = FactSet::new(apple(), fiscal_year_2024());
        let fact = StandardizedFact {
            dimensions: americas_segment(),
            ..revenue("167045000000")
        };

        let expected_result = Err(FactSetError::UnexpectedDimensions {
            concept: IncomeStatementConcept::Revenue.element(),
            dimensions: americas_segment(),
        });

        let result = fact_set.insert(fact);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_set_unchanged_when_the_insert_rejects_a_fact() {
        let mut fact_set = FactSet::new(apple(), fiscal_year_2024());
        fact_set
            .insert(revenue("391035000000"))
            .expect("Given a fact of the entity and the period of the set, the insert should always succeed");
        let fact = StandardizedFact {
            period: fiscal_year_2023(),
            ..revenue("383285000000")
        };

        let expected_result = Some(revenue("391035000000"));

        fact_set
            .insert(fact)
            .expect_err("Given a fact of another period, the insert should always fail");
        let result = fact_set
            .get(IncomeStatementConcept::Revenue.element())
            .cloned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_fact_when_the_set_and_the_fact_give_different_exact_starts() {
        let mut fact_set = FactSet::new(apple(), fiscal_year_2024_with_start(1));
        let fact = StandardizedFact {
            period: fiscal_year_2024_with_start(2),
            ..revenue("391035000000")
        };

        let expected_result = Err(FactSetError::UnexpectedPeriod {
            concept: IncomeStatementConcept::Revenue.element(),
            expected: fiscal_year_2024_with_start(1),
            found: fiscal_year_2024_with_start(2),
        });

        let result = fact_set.insert(fact);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_fact_when_it_is_an_instant_at_the_end_of_the_period_of_the_set() {
        let mut fact_set = FactSet::new(apple(), fiscal_year_2024());
        let fact = StandardizedFact {
            period: end_of_fiscal_year_2024(),
            ..revenue("391035000000")
        };

        let expected_result = Err(FactSetError::UnexpectedPeriod {
            concept: IncomeStatementConcept::Revenue.element(),
            expected: fiscal_year_2024(),
            found: end_of_fiscal_year_2024(),
        });

        let result = fact_set.insert(fact);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_check_the_entity_first_when_the_entity_and_the_period_differ() {
        let mut fact_set = FactSet::new(apple(), fiscal_year_2024());
        let fact = StandardizedFact {
            entity: microsoft(),
            period: fiscal_year_2023(),
            ..revenue("211915000000")
        };

        let expected_result = Err(FactSetError::UnexpectedEntity {
            concept: IncomeStatementConcept::Revenue.element(),
            expected: apple(),
            found: microsoft(),
        });

        let result = fact_set.insert(fact);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_check_the_period_before_the_dimensions_when_both_differ() {
        let mut fact_set = FactSet::new(apple(), fiscal_year_2024());
        let fact = StandardizedFact {
            period: fiscal_year_2023(),
            dimensions: americas_segment(),
            ..revenue("162560000000")
        };

        let expected_result = Err(FactSetError::UnexpectedPeriod {
            concept: IncomeStatementConcept::Revenue.element(),
            expected: fiscal_year_2024(),
            found: fiscal_year_2023(),
        });

        let result = fact_set.insert(fact);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_both_entities_when_an_unexpected_entity_error_is_printed() {
        let error = FactSetError::UnexpectedEntity {
            concept: IncomeStatementConcept::Revenue.element(),
            expected: apple(),
            found: microsoft(),
        };

        let expected_result = "[UnexpectedEntity] Fact of 'Revenue' has entity 'http://www.sec.gov/CIK 0000789019', the set holds entity 'http://www.sec.gov/CIK 0000320193'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_both_periods_when_an_unexpected_period_error_is_printed() {
        let error = FactSetError::UnexpectedPeriod {
            concept: IncomeStatementConcept::Revenue.element(),
            expected: fiscal_year_2024(),
            found: fiscal_year_2023(),
        };

        let expected_result = "[UnexpectedPeriod] Fact of 'Revenue' has period '2022-09-25 to 2023-09-30', the set holds period '2023-10-01 to 2024-09-28'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_axis_member_pairs_when_an_unexpected_dimensions_error_is_printed() {
        let error = FactSetError::UnexpectedDimensions {
            concept: IncomeStatementConcept::Revenue.element(),
            dimensions: americas_segment(),
        };

        let expected_result = "[UnexpectedDimensions] Fact of 'Revenue' has dimensions 'StatementBusinessSegmentsAxis=AmericasSegmentMember', the set holds facts without dimensions";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
