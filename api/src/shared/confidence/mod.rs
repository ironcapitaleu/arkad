//! # Confidence
//!
//! Provides the [`Confidence`] of a fact in the format of the `/v1` contract.

use serde::Serialize;
use utoipa::ToSchema;
use xbrl::core::confidence::Confidence as DomainConfidence;

use crate::traits::from_domain::FromDomain;

/// The resolution tier that produced a fact. A later tier is less certain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// The filing used the first concept of the alias list.
    Exact,
    /// The filing used a later concept of the alias list, or a part that it reports alone.
    Synonym,
    /// The value comes from other standardized values by an accounting or a period identity.
    Derived,
    /// The value is a sum along the calculation tree.
    Computed,
}

impl From<DomainConfidence> for Confidence {
    /// Converts an `xbrl` [`Confidence`](DomainConfidence) into a contract [`Confidence`].
    fn from(confidence: DomainConfidence) -> Self {
        match confidence {
            DomainConfidence::Exact => Self::Exact,
            DomainConfidence::Synonym => Self::Synonym,
            DomainConfidence::Derived => Self::Derived,
            DomainConfidence::Computed => Self::Computed,
        }
    }
}

impl FromDomain<DomainConfidence> for Confidence {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_confidence() {
        implements_auto_traits::<Confidence>();
    }

    #[test]
    fn should_convert_to_synonym_when_the_domain_confidence_is_synonym() {
        let domain = DomainConfidence::Synonym;

        let expected_result = Confidence::Synonym;

        let result = Confidence::from(domain);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_in_snake_case_for_exact() {
        let confidence = Confidence::Exact;

        let expected_result = json!("exact");

        let result = serde_json::to_value(confidence)
            .expect("Given a unit variant, serializing it should always succeed");

        assert_eq!(result, expected_result);
    }
}
