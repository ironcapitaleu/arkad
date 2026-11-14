//! # Concepts
//!
//! Provides the [`list_concepts`] handler of `GET /v1/concepts`.

use axum::Json;
use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;
use xbrl::core::elements::CanonicalElement;
use xbrl::us_gaap::mappings::{OPTIONAL_CONCEPTS, REQUIRED_CONCEPTS};

use crate::error::{ApiError, Problem};
use crate::logging::ApiEvent;
use crate::shared::concept_id::ConceptId;
use crate::shared::pascal_case::InvalidPascalCase;

/// One canonical concept that `xbrl` maps from US-GAAP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Concept {
    /// The stable identifier of the concept.
    pub concept: ConceptId,
    /// The name of the concept for people.
    #[schema(example = "Net Income")]
    pub name: String,
}

impl Concept {
    /// Builds the [`Concept`] of `element`.
    ///
    /// The name is the `Display` text of the variant.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidPascalCase`] if the serialized name of the variant is not in
    /// `PascalCase`.
    fn from_element(element: CanonicalElement) -> Result<Self, InvalidPascalCase> {
        Ok(Self {
            concept: element.try_into()?,
            name: element.to_string(),
        })
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
    fn new(as_of: DateTime<Utc>) -> Result<Self, InvalidPascalCase> {
        Ok(Self {
            data: REQUIRED_CONCEPTS
                .iter()
                .chain(OPTIONAL_CONCEPTS)
                .map(|definition| Concept::from_element(definition.canonical_element()))
                .collect::<Result<_, _>>()?,
            next_cursor: None,
            as_of: as_of.to_rfc3339_opts(SecondsFormat::Secs, true),
        })
    }
}

/// Answers `GET /v1/concepts` with every concept that `xbrl` maps from US-GAAP.
///
/// # Errors
///
/// Returns [`ApiError::InternalError`] if the name of a mapped element is not in `PascalCase`.
/// The handler logs the rejected name, because the problem body carries no internal detail.
#[utoipa::path(
    get,
    path = "/concepts",
    tag = "concepts",
    description = "Lists every concept that `xbrl` maps from US-GAAP, the required concepts first.",
    responses(
        (status = 200, description = "Every concept that `xbrl` maps from US-GAAP", body = ConceptList),
        (status = 500, description = "The server failed to build the list", body = Problem, content_type = "application/problem+json"),
    ),
)]
// axum requires an async handler. This one has nothing to await.
#[allow(clippy::unused_async)]
pub async fn list_concepts() -> Result<Json<ConceptList>, ApiError> {
    let list = ConceptList::new(Utc::now()).inspect_err(|error| {
        tracing::error!(
            event = %ApiEvent::RequestFailed,
            message = %format!("[FailedConceptList] Failed to build the concept list, Caused by: {error}"),
            context = %json!({ "path": "/v1/concepts", "name": error.name }),
        );
    })?;
    Ok(Json(list))
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

        let expected_result = json!({ "concept": "net_income", "name": "Net Income" });

        let concept = Concept::from_element(element).expect(
            "Given a variant with a PascalCase name, building the concept should always succeed",
        );
        let result = serde_json::to_value(concept)
            .expect("Given a struct of strings, serializing it should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_as_of_as_rfc_3339_in_utc_for_concept_list() {
        let as_of = Utc
            .with_ymd_and_hms(2026, 10, 30, 6, 0, 0)
            .single()
            .expect("Given a valid hardcoded UTC time, building it should always succeed");

        let expected_result = Ok("2026-10-30T06:00:00Z".to_owned());

        let result = ConceptList::new(as_of).map(|list| list.as_of);

        assert_eq!(result, expected_result);
    }
}
