//! # Concept Definition Module
//!
//! Provides the [`ConceptDefinition`] struct that describes an XBRL financial concept
//! to look for in SEC Company Facts data. Each definition specifies a canonical name,
//! a prioritized list of XBRL key aliases, the expected measurement unit, and whether
//! the concept is required for validation.
//!
//! ## Modules
//! - [`constants`]: Canonical name constants and predefined concept definition arrays.
//!
//! ## Design
//! Concept definitions are **statement-agnostic** -- a concept like Net Income can appear
//! in both the Income Statement and the Cash Flow Statement. Financial statements reference
//! concepts; concepts do not know about statements.

pub mod constants;

use crate::shared::financial::unit::Unit;

/// Specification for an XBRL financial concept to extract from SEC data.
///
/// Describes what to look for in the SEC Company Facts JSON response.
/// The `xbrl_keys` are tried in priority order -- the first match wins.
/// This handles the fact that different companies may use different XBRL
/// tag names for the same economic concept (e.g., "Revenues" vs "`SalesRevenueNet`").
///
/// # Example
/// ```
/// use sec::shared::financial::concept_definition::ConceptDefinition;
/// use sec::shared::financial::unit::Unit;
///
/// let revenue = ConceptDefinition::new(
///     "Revenue",
///     &["Revenues", "SalesRevenueNet"],
///     Unit::Usd,
///     true,
/// );
/// assert_eq!(revenue.canonical_name(), "Revenue");
/// assert!(revenue.required());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConceptDefinition {
    canonical_name: &'static str,
    xbrl_keys: &'static [&'static str],
    expected_unit: Unit,
    required: bool,
}

impl ConceptDefinition {
    /// Creates a new [`ConceptDefinition`].
    ///
    /// # Arguments
    ///
    /// * `canonical_name` - The canonical name for querying (should reference a constant).
    /// * `xbrl_keys` - XBRL key aliases tried in priority order.
    /// * `expected_unit` - The expected measurement unit for this concept.
    /// * `required` - Whether this concept must be present for validation to succeed.
    #[must_use]
    pub const fn new(
        canonical_name: &'static str,
        xbrl_keys: &'static [&'static str],
        expected_unit: Unit,
        required: bool,
    ) -> Self {
        Self {
            canonical_name,
            xbrl_keys,
            expected_unit,
            required,
        }
    }

    /// Returns the canonical name used for querying.
    #[must_use]
    pub const fn canonical_name(&self) -> &'static str {
        self.canonical_name
    }

    /// Returns the prioritized list of XBRL key aliases.
    #[must_use]
    pub const fn xbrl_keys(&self) -> &'static [&'static str] {
        self.xbrl_keys
    }

    /// Returns the expected measurement unit.
    #[must_use]
    pub const fn expected_unit(&self) -> Unit {
        self.expected_unit
    }

    /// Returns whether this concept is required for validation.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
}

impl std::fmt::Display for ConceptDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} ({}{})",
            self.canonical_name,
            self.expected_unit,
            if self.required { ", required" } else { "" }
        )
    }
}

impl serde::Serialize for ConceptDefinition {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ConceptDefinition", 4)?;
        state.serialize_field("canonical_name", &self.canonical_name)?;
        state.serialize_field("xbrl_keys", &self.xbrl_keys)?;
        state.serialize_field("expected_unit", &self.expected_unit)?;
        state.serialize_field("required", &self.required)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_canonical_name_when_accessed() {
        let concept = ConceptDefinition::new("Revenue", &["Revenues"], Unit::Usd, true);

        let expected_result = "Revenue";

        let result = concept.canonical_name();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_xbrl_keys_when_accessed() {
        let keys: &[&str] = &["Revenues", "SalesRevenueNet"];
        let concept = ConceptDefinition::new("Revenue", keys, Unit::Usd, true);

        let expected_result = &["Revenues", "SalesRevenueNet"];

        let result = concept.xbrl_keys();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_true_when_concept_is_required() {
        let concept = ConceptDefinition::new("Revenue", &["Revenues"], Unit::Usd, true);

        let expected_result = true;

        let result = concept.required();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_expected_unit_when_accessed() {
        let concept = ConceptDefinition::new(
            "Shares Outstanding",
            &["EntityCommonStockSharesOutstanding"],
            Unit::Shares,
            true,
        );

        let expected_result = Unit::Shares;

        let result = concept.expected_unit();

        assert_eq!(result, expected_result);
    }
}
