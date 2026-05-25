//! # Fact Set
//!
//! A coherent collection of resolved facts for one entity and one period.

use std::collections::HashMap;

use super::elements::CanonicalElement;
use super::entity_name::EntityName;
use super::period::Period;
use super::resolved_fact::ResolvedFact;

/// A coherent set of resolved financial facts for a single entity and period.
///
/// Represents the resolved state of one financial reporting period — the
/// collection that validation checks operate on.
#[derive(Debug, Clone)]
pub struct FactSet {
    /// The reporting entity.
    entity: EntityName,
    /// The period these facts cover.
    period: Period,
    /// Resolved facts keyed by canonical element.
    facts: HashMap<CanonicalElement, ResolvedFact>,
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

    /// Returns the resolved facts map.
    #[must_use]
    pub const fn facts(&self) -> &HashMap<CanonicalElement, ResolvedFact> {
        &self.facts
    }

    /// Inserts a resolved fact into the set.
    pub fn insert(&mut self, fact: ResolvedFact) {
        self.facts.insert(fact.canonical_element(), fact);
    }

    /// Returns the resolved fact for a given element, if present.
    #[must_use]
    pub fn get(&self, element: CanonicalElement) -> Option<&ResolvedFact> {
        self.facts.get(&element)
    }
}
