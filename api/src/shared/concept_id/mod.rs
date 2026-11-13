//! # Concept ID
//!
//! Provides the [`ConceptId`] of a canonical concept in the format of the `/v1` contract.

use serde::Serialize;
use utoipa::ToSchema;
use xbrl::core::elements::CanonicalElement;

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

impl TryFrom<CanonicalElement> for ConceptId {
    type Error = InvalidPascalCase;

    /// Converts a [`CanonicalElement`] into its [`ConceptId`]: the serialized variant name in
    /// `snake_case`.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidPascalCase`] if the serialized name of the variant is not in
    /// `PascalCase`.
    ///
    /// # Panics
    ///
    /// Panics if serde does not serialize the unit variant as a string. A derived `Serialize` on
    /// a unit variant always writes a string.
    fn try_from(element: CanonicalElement) -> Result<Self, Self::Error> {
        let variant = serde_json::to_value(element)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .expect(
                "Given a unit variant of an enum, serde should always serialize it as a string",
            );
        Ok(Self(PascalCase::new(&variant)?.to_snake_case()))
    }
}

impl FromDomain<CanonicalElement> for ConceptId {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_concept_id() {
        implements_auto_traits::<ConceptId>();
    }

    #[test]
    fn should_convert_to_snake_case_when_the_element_is_net_income() {
        let element = CanonicalElement::NetIncome;

        let expected_result = Ok(ConceptId("net_income".to_owned()));

        let result = ConceptId::try_from(element);

        assert_eq!(result, expected_result);
    }
}
