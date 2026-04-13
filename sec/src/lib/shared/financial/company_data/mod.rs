//! # Company Data Module
//!
//! Provides the [`CompanyData`] struct, the top-level output of the `ParseCompanyFacts` state.
//! Contains all resolved company facts keyed by their concept definition, enabling
//! canonical querying (e.g., look up "Revenue" regardless of the company's XBRL key).

use std::collections::HashMap;
use std::fmt;

use crate::shared::cik::Cik;
use crate::shared::financial::company_fact::CompanyFact;
use crate::shared::financial::concept_definition::ConceptDefinition;
use crate::shared::financial::entity_name::EntityName;

/// Top-level container for all resolved financial facts of a company.
///
/// Produced by the `ParseCompanyFacts` state after validating and resolving
/// SEC Company Facts data. Facts are keyed by their [`ConceptDefinition`] reference,
/// enabling querying by canonical concept name.
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use sec::shared::cik::Cik;
/// use sec::shared::financial::company_data::CompanyData;
/// use sec::shared::financial::entity_name::EntityName;
///
/// let company_data = CompanyData::new(
///     Cik::new("0000320193").unwrap(),
///     EntityName::new("Apple Inc."),
///     HashMap::new(),
/// );
/// assert_eq!(company_data.entity_name().value(), "Apple Inc.");
/// ```
#[derive(Debug, Clone)]
pub struct CompanyData {
    cik: Cik,
    entity_name: EntityName,
    facts: HashMap<&'static ConceptDefinition, CompanyFact>,
}

impl CompanyData {
    /// Creates a new [`CompanyData`] with the given components.
    #[must_use]
    pub const fn new(
        cik: Cik,
        entity_name: EntityName,
        facts: HashMap<&'static ConceptDefinition, CompanyFact>,
    ) -> Self {
        Self {
            cik,
            entity_name,
            facts,
        }
    }

    /// Returns a reference to the company's CIK.
    #[must_use]
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }

    /// Returns a reference to the entity name.
    #[must_use]
    pub const fn entity_name(&self) -> &EntityName {
        &self.entity_name
    }

    /// Returns a reference to the resolved facts map.
    #[must_use]
    pub const fn facts(&self) -> &HashMap<&'static ConceptDefinition, CompanyFact> {
        &self.facts
    }

    /// Looks up a [`CompanyFact`] by its [`ConceptDefinition`].
    #[must_use]
    pub fn get_fact(&self, concept: &'static ConceptDefinition) -> Option<&CompanyFact> {
        self.facts.get(concept)
    }
}

impl PartialEq for CompanyData {
    fn eq(&self, other: &Self) -> bool {
        self.cik == other.cik && self.entity_name == other.entity_name
    }
}

impl Eq for CompanyData {}

// Deviation: `HashMap` does not implement `Hash`, so only
// `cik` and `entity_name` are hashed.
impl std::hash::Hash for CompanyData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.cik.hash(state);
        self.entity_name.hash(state);
    }
}

impl PartialOrd for CompanyData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Deviation: ordering is based on `cik` only.
impl Ord for CompanyData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cik.cmp(&other.cik)
    }
}

impl serde::Serialize for CompanyData {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("CompanyData", 3)?;
        state.serialize_field("cik", &self.cik)?;
        state.serialize_field("entity_name", &self.entity_name)?;
        state.serialize_field("facts_count", &self.facts.len())?;
        state.end()
    }
}

impl fmt::Display for CompanyData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (CIK: {}, {} facts)",
            self.entity_name,
            self.cik,
            self.facts.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_cik_when_accessed() {
        let cik = Cik::new("0000320193").expect("Hardcoded CIK should always be valid");
        let company_data =
            CompanyData::new(cik.clone(), EntityName::new("Apple Inc."), HashMap::new());

        let expected_result = &cik;

        let result = company_data.cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_entity_name_when_accessed() {
        let cik = Cik::new("0000320193").expect("Hardcoded CIK should always be valid");
        let company_data = CompanyData::new(cik, EntityName::new("Apple Inc."), HashMap::new());

        let expected_result = "Apple Inc.";

        let result = company_data.entity_name().value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_fact_not_present() {
        let cik = Cik::new("0000320193").expect("Hardcoded CIK should always be valid");
        let company_data = CompanyData::new(cik, EntityName::new("Apple Inc."), HashMap::new());
        let concept =
            &crate::shared::financial::concept_definition::constants::REQUIRED_CONCEPTS[0];

        let expected_result = true;

        let result = company_data.get_fact(concept).is_none();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_company_data_summary_when_formatted() {
        let cik = Cik::new("0000320193").expect("Hardcoded CIK should always be valid");
        let company_data = CompanyData::new(cik, EntityName::new("Apple Inc."), HashMap::new());

        let expected_result = "Apple Inc. (CIK: 0000320193, 0 facts)";

        let result = company_data.to_string();

        assert_eq!(result, expected_result);
    }
}
