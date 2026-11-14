//! # Concept ID
//!
//! Provides the [`ConceptId`] of a canonical concept in the format of the `/v1` contract.

use serde::Serialize;
use utoipa::ToSchema;
use xbrl::core::elements::CanonicalElementKind;

use crate::shared::pascal_case::{InvalidPascalCase, PascalCase};
use crate::traits::from_domain::FromDomain;

/// The stable identifier of a concept in `snake_case`, for example `net_income`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[schema(example = "net_income")]
pub struct ConceptId(String);

impl ConceptId {
    /// Returns the identifier as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<CanonicalElementKind> for ConceptId {
    type Error = InvalidPascalCase;

    /// Converts a [`CanonicalElementKind`] into its [`ConceptId`]: the name in silver of the
    /// concept, from [`CanonicalElementKind::name`], in `snake_case`.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidPascalCase`] if the name of the concept is not in `PascalCase`.
    fn try_from(element: CanonicalElementKind) -> Result<Self, Self::Error> {
        Ok(Self(PascalCase::new(element.name())?.to_snake_case()))
    }
}

impl FromDomain<CanonicalElementKind> for ConceptId {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use xbrl::core::elements::IncomeStatementConcept;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_concept_id() {
        implements_auto_traits::<ConceptId>();
    }

    #[test]
    fn should_convert_to_snake_case_when_the_element_is_net_income() {
        let element = IncomeStatementConcept::NetIncome.element();

        let expected_result = Ok(ConceptId("net_income".to_owned()));

        let result = ConceptId::try_from(element);

        assert_eq!(result, expected_result);
    }
}
