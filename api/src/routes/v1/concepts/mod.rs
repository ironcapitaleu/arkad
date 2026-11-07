//! # Concepts
//!
//! Provides the [`list_concepts`] handler of `GET /v1/concepts`.

use axum::Json;
use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use xbrl::core::elements::CanonicalElement;
use xbrl::us_gaap::mappings::{OPTIONAL_CONCEPTS, REQUIRED_CONCEPTS};

/// One canonical concept that `xbrl` maps from US-GAAP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Concept {
    /// The stable identifier of the concept.
    #[schema(example = "net_income")]
    pub concept: String,
    /// The name of the concept for people.
    #[schema(example = "Net Income")]
    pub name: String,
}

impl Concept {
    /// Builds the [`Concept`] of `element`.
    ///
    /// The identifier is the serialized name of the variant in `snake_case`, so `NetIncome`
    /// becomes `net_income`. The name is the `Display` text of the variant.
    fn from_element(element: CanonicalElement) -> Self {
        let variant = serde_json::to_value(element)
            .ok()
            .and_then(|value| value.as_str().map(snake_case))
            .expect(
                "Given a unit variant of an enum, serde should always serialize it as a string",
            );
        Self {
            concept: variant,
            name: element.to_string(),
        }
    }
}

/// The body of a `GET /v1/concepts` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ConceptList {
    /// Every concept that `xbrl` maps from US-GAAP, the required concepts first.
    pub data: Vec<Concept>,
    /// The cursor of the next page. Always `null`, because the list fits on one page.
    #[schema(required = true)]
    pub next_cursor: Option<String>,
    /// The time of the response, RFC 3339 in UTC.
    #[schema(format = DateTime, example = "2026-10-30T06:00:00Z")]
    pub as_of: String,
}

impl ConceptList {
    fn new(as_of: DateTime<Utc>) -> Self {
        Self {
            data: REQUIRED_CONCEPTS
                .iter()
                .chain(OPTIONAL_CONCEPTS)
                .map(|definition| Concept::from_element(definition.canonical_element()))
                .collect(),
            next_cursor: None,
            as_of: as_of.to_rfc3339_opts(SecondsFormat::Secs, true),
        }
    }
}

/// Answers `GET /v1/concepts` with every concept that `xbrl` maps from US-GAAP.
#[utoipa::path(
    get,
    path = "/concepts",
    tag = "concepts",
    responses(
        (status = 200, description = "Every concept that `xbrl` maps from US-GAAP", body = ConceptList),
    ),
)]
// axum requires an async handler. This one has nothing to await.
#[allow(clippy::unused_async)]
pub async fn list_concepts() -> Json<ConceptList> {
    Json(ConceptList::new(Utc::now()))
}

fn snake_case(pascal_case: &str) -> String {
    let mut text = String::with_capacity(pascal_case.len() + 4);
    for (index, character) in pascal_case.char_indices() {
        if index > 0 && character.is_ascii_uppercase() {
            text.push('_');
        }
        text.push(character.to_ascii_lowercase());
    }
    text
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_concept() {
        implements_auto_traits::<Concept>();
    }

    #[test]
    const fn should_implement_auto_traits_for_concept_list() {
        implements_auto_traits::<ConceptList>();
    }

    #[test]
    fn should_build_snake_case_concept_and_display_name_for_net_income() {
        let element = CanonicalElement::NetIncome;

        let expected_result = Concept {
            concept: "net_income".to_owned(),
            name: "Net Income".to_owned(),
        };

        let result = Concept::from_element(element);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_as_of_as_rfc_3339_in_utc_for_concept_list() {
        let as_of = Utc
            .with_ymd_and_hms(2026, 10, 30, 6, 0, 0)
            .single()
            .expect("Given a valid hardcoded UTC time, building it should always succeed");

        let expected_result = "2026-10-30T06:00:00Z";

        let result = ConceptList::new(as_of).as_of;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_a_single_word_for_snake_case_of_assets() {
        let pascal_case = "Assets";

        let expected_result = "assets";

        let result = snake_case(pascal_case);

        assert_eq!(result, expected_result);
    }
}
