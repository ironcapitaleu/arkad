//! # Company Data
//!
//! Provides the [`CompanyData`] struct: the as-is facts and the filings of one company.

use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use xbrl::core::entity_name::EntityName;
use xbrl::core::fact::AsIsFact;
use xbrl::core::filing::Filing;

use crate::shared::cik::Cik;

/// The as-is facts and the filings of one company, as its Company Facts body holds them.
///
/// The output of the
/// [`ParseCompanyFacts`](crate::implementations::states::transform::parse_company_facts) state.
/// It holds every fact of every form, amendment, unit, and namespace.
#[derive(Debug, Clone)]
pub struct CompanyData {
    cik: Cik,
    entity_name: EntityName,
    facts: Vec<AsIsFact>,
    filings: Vec<Filing>,
}

impl CompanyData {
    /// Creates a [`CompanyData`] from a CIK, an entity name, the as-is facts, and the filings.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::cik::Cik;
    /// use sec::shared::company_data::CompanyData;
    /// use xbrl::core::entity_name::EntityName;
    ///
    /// let cik = Cik::new("0000320193").expect("A hardcoded valid CIK should always parse");
    /// let company_data =
    ///     CompanyData::new(cik, EntityName::new("Apple Inc."), Vec::new(), Vec::new());
    ///
    /// let expected_result = "Apple Inc.";
    ///
    /// let result = company_data.entity_name().as_str();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(
        cik: Cik,
        entity_name: EntityName,
        facts: Vec<AsIsFact>,
        filings: Vec<Filing>,
    ) -> Self {
        Self {
            cik,
            entity_name,
            facts,
            filings,
        }
    }

    /// Returns the CIK of the company.
    #[must_use]
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }

    /// Returns the entity name.
    #[must_use]
    pub const fn entity_name(&self) -> &EntityName {
        &self.entity_name
    }

    /// Returns the as-is facts.
    #[must_use]
    pub fn facts(&self) -> &[AsIsFact] {
        &self.facts
    }

    /// Returns the filings, one per accession number.
    #[must_use]
    pub fn filings(&self) -> &[Filing] {
        &self.filings
    }
}

// Deviation: `AsIsFact` and `Filing` implement no `Ord`, and a state data type must implement
// `Ord`. So equality, hashing, and order use `cik` and `entity_name` only, and agree.
impl PartialEq for CompanyData {
    fn eq(&self, other: &Self) -> bool {
        self.cik == other.cik && self.entity_name == other.entity_name
    }
}

impl Eq for CompanyData {}

impl Hash for CompanyData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cik.hash(state);
        self.entity_name.hash(state);
    }
}

impl PartialOrd for CompanyData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CompanyData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cik
            .cmp(&other.cik)
            .then_with(|| self.entity_name.cmp(&other.entity_name))
    }
}

/// Serializes the CIK, the entity name, and the counts of facts and filings, so the logs of a
/// transition stay small.
impl Serialize for CompanyData {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("CompanyData", 4)?;
        state.serialize_field("cik", &self.cik)?;
        state.serialize_field("entity_name", &self.entity_name)?;
        state.serialize_field("facts_count", &self.facts.len())?;
        state.serialize_field("filings_count", &self.filings.len())?;
        state.end()
    }
}

impl Display for CompanyData {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{} (CIK: {}, {} facts, {} filings)",
            self.entity_name,
            self.cik,
            self.facts.len(),
            self.filings.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn apple_cik() -> Cik {
        Cik::new("0000320193")
            .expect("Given a hardcoded valid CIK, the creation should always succeed")
    }

    #[test]
    fn should_return_cik_when_accessed() {
        let company_data = CompanyData::new(
            apple_cik(),
            EntityName::new("Apple Inc."),
            Vec::new(),
            Vec::new(),
        );

        let expected_result = &apple_cik();

        let result = company_data.cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_entity_name_when_accessed() {
        let company_data = CompanyData::new(
            apple_cik(),
            EntityName::new("Apple Inc."),
            Vec::new(),
            Vec::new(),
        );

        let expected_result = "Apple Inc.";

        let result = company_data.entity_name().as_str();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_company_data_summary_when_formatted() {
        let company_data = CompanyData::new(
            apple_cik(),
            EntityName::new("Apple Inc."),
            Vec::new(),
            Vec::new(),
        );

        let expected_result = "Apple Inc. (CIK: 0000320193, 0 facts, 0 filings)";

        let result = company_data.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_counts_instead_of_facts_when_serialized() {
        let company_data = CompanyData::new(
            apple_cik(),
            EntityName::new("Apple Inc."),
            Vec::new(),
            Vec::new(),
        );

        let expected_result = serde_json::json!({
            "cik": { "value": "0000320193" },
            "entity_name": "Apple Inc.",
            "facts_count": 0,
            "filings_count": 0
        });

        let result = serde_json::to_value(&company_data)
            .expect("Given a company data with plain fields, serialization should always succeed");

        assert_eq!(result, expected_result);
    }
}
