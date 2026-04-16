//! # Company Fact Module
//!
//! Provides the [`CompanyFact`] struct representing a company's reported data
//! for a specific financial concept. A `CompanyFact` is the result of matching
//! a [`ConceptDefinition`](crate::shared::financial::concept_definition::ConceptDefinition)
//! against a specific company's SEC data.

use std::fmt;

use crate::shared::financial::observation::Observation;

/// A company's reported data for a specific financial concept.
///
/// Contains the company's own label for the concept, which XBRL key was matched,
/// and the full time series of observations. This enables querying by canonical name
/// while preserving the company's original terminology.
///
/// # Example
/// ```
/// use sec::shared::financial::company_fact::CompanyFact;
///
/// let fact = CompanyFact::new(
///     "Net Sales".to_string(),
///     "RevenueFromContractWithCustomerExcludingAssessedTax".to_string(),
///     vec![],
/// );
/// assert_eq!(fact.company_label(), "Net Sales");
/// ```
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompanyFact {
    company_label: String,
    matched_xbrl_key: String,
    observations: Vec<Observation>,
}

impl CompanyFact {
    /// Creates a new [`CompanyFact`] with the given components.
    ///
    /// # Arguments
    ///
    /// * `company_label` - The label the company uses for this concept in its filings.
    /// * `matched_xbrl_key` - The XBRL key that was matched from the concept definition's aliases.
    /// * `observations` - The time series of measured values for this concept.
    #[must_use]
    pub const fn new(
        company_label: String,
        matched_xbrl_key: String,
        observations: Vec<Observation>,
    ) -> Self {
        Self {
            company_label,
            matched_xbrl_key,
            observations,
        }
    }

    /// Returns the company's own label for this concept (e.g., "Net Sales").
    #[must_use]
    pub fn company_label(&self) -> &str {
        &self.company_label
    }

    /// Returns which XBRL key was matched from the concept definition's aliases.
    #[must_use]
    pub fn matched_xbrl_key(&self) -> &str {
        &self.matched_xbrl_key
    }

    /// Returns a reference to the time series of observations.
    #[must_use]
    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }
}

impl PartialEq for CompanyFact {
    fn eq(&self, other: &Self) -> bool {
        self.company_label == other.company_label
            && self.matched_xbrl_key == other.matched_xbrl_key
            && self.observations == other.observations
    }
}

impl Eq for CompanyFact {}

// Observations are excluded from hashing for performance -- the time series
// can be large, and hashing it on every call would be expensive.
// `company_label` and `matched_xbrl_key` are sufficient to identify a fact.
impl std::hash::Hash for CompanyFact {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.company_label.hash(state);
        self.matched_xbrl_key.hash(state);
    }
}

impl PartialOrd for CompanyFact {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Ordering is based on the same fields as equality to preserve the `Ord` contract.
// `Vec<Observation>` implements `Ord` since `Observation` derives it.
impl Ord for CompanyFact {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.matched_xbrl_key
            .cmp(&other.matched_xbrl_key)
            .then_with(|| self.company_label.cmp(&other.company_label))
            .then_with(|| self.observations.cmp(&other.observations))
    }
}

impl fmt::Display for CompanyFact {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({}, {} observations)",
            self.company_label,
            self.matched_xbrl_key,
            self.observations.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_company_label_when_accessed() {
        let fact = CompanyFact::new(
            "Net Sales".to_string(),
            "SalesRevenueNet".to_string(),
            vec![],
        );

        let expected_result = "Net Sales";

        let result = fact.company_label();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_matched_xbrl_key_when_accessed() {
        let fact = CompanyFact::new(
            "Net Sales".to_string(),
            "SalesRevenueNet".to_string(),
            vec![],
        );

        let expected_result = "SalesRevenueNet";

        let result = fact.matched_xbrl_key();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_empty_observations_when_created_with_no_data() {
        let fact = CompanyFact::new("Revenue".to_string(), "Revenues".to_string(), vec![]);

        let expected_result = 0;

        let result = fact.observations().len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_fact_summary_when_formatted() {
        let fact = CompanyFact::new("Revenue".to_string(), "Revenues".to_string(), vec![]);

        let expected_result = "Revenue (Revenues, 0 observations)";

        let result = fact.to_string();

        assert_eq!(result, expected_result);
    }
}
