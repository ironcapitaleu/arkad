//! # Facts
//!
//! Provides the [`list_facts`] handler of `GET /v1/companies/{company_id}/facts`.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use xbrl::core::fiscal_year::FiscalYear;
use xbrl::core::form::Form;

use super::CompanyPath;
use crate::error::{ApiError, Problem};
use crate::shared::accession_number::AccessionNumber;
use crate::shared::concept_id::ConceptId;
use crate::shared::confidence::Confidence;
use crate::shared::exact_decimal::ExactDecimal;
use crate::shared::fiscal_period::FiscalPeriod;
use crate::shared::period::Period;
use crate::shared::unit::Unit;

/// One value of one concept for one period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Fact {
    /// The concept, from `GET /v1/concepts`.
    pub concept: ConceptId,
    /// The exact decimal value in whole units of `unit`, as a string.
    pub value: ExactDecimal,
    /// An ISO 4217 currency code, `shares`, `usd_per_share`, or `pure` for a ratio.
    pub unit: Unit,
    /// The fiscal year of the value.
    #[schema(value_type = u16, example = 2024)]
    pub fiscal_year: FiscalYear,
    /// The fiscal period of the value.
    pub fiscal_period: FiscalPeriod,
    /// The period that the value covers.
    pub period: Period,
    /// `null` for a consolidated total. With `include=segments`, the axis and the member.
    #[schema(required = true, value_type = Option<Object>)]
    pub segment: Option<serde_json::Value>,
    /// The filing that the value comes from.
    pub source: FactSource,
    /// How the value maps to its concept. Present only with `include=lineage`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub lineage: Option<Lineage>,
}

/// The filing that a fact comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct FactSource {
    /// The accession number of the filing.
    pub accession_number: AccessionNumber,
    /// The form type of the filing.
    #[schema(value_type = String, example = "10-K")]
    pub form: Form,
    /// The filing date.
    #[schema(value_type = String, format = Date, example = "2024-11-01")]
    pub filed_at: NaiveDate,
    /// The folder of the filing on SEC EDGAR.
    #[schema(example = "https://www.sec.gov/Archives/edgar/data/320193/000032019324000123/")]
    pub url: String,
}

/// How a fact maps to its concept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Lineage {
    /// The resolution tier that produced the value.
    pub confidence: Confidence,
    /// The concept that the filing used.
    #[schema(example = "us-gaap:RevenueFromContractWithCustomerExcludingAssessedTax")]
    pub native_concept: String,
    /// The filed concepts that the value comes from, in order.
    #[schema(example = json!(["us-gaap:RevenueFromContractWithCustomerExcludingAssessedTax"]))]
    pub resolution_path: Vec<String>,
}

/// The body of a `GET /v1/companies/{company_id}/facts` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct FactList {
    /// The facts of this page, sorted by concept, period end, and segment.
    pub data: Vec<Fact>,
    /// The cursor of the next page, or `null` on the last page.
    #[schema(required = true)]
    pub next_cursor: Option<String>,
    /// The time that fixes the data of every page, RFC 3339 in UTC.
    #[schema(format = DateTime, example = "2026-10-30T06:00:00Z")]
    pub as_of: String,
    /// The first fiscal year that the plan allows if the plan cut the result, or `null`.
    #[schema(required = true, value_type = Option<u16>, example = 2021)]
    pub history_limited_from: Option<FiscalYear>,
}

/// The query parameters of `GET /v1/companies/{company_id}/facts`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FactQuery {
    /// One or more concepts from `GET /v1/concepts`, separated by commas. At most 20.
    #[param(example = "revenue,net_income")]
    pub concept: String,
    /// One fiscal period. Default: all.
    pub fiscal_period: Option<FiscalPeriod>,
    /// The first fiscal year, inclusive.
    #[param(example = 2020)]
    pub fiscal_year_from: Option<u16>,
    /// The last fiscal year, inclusive.
    #[param(example = 2024)]
    pub fiscal_year_to: Option<u16>,
    /// `segments`, `lineage`, or both, separated by commas.
    #[param(example = "segments,lineage")]
    pub include: Option<String>,
    /// The number of facts per page. Default: 100.
    #[param(minimum = 1, maximum = 1000)]
    pub limit: Option<u32>,
    /// The `next_cursor` of the previous page.
    pub cursor: Option<String>,
}

/// Answers `GET /v1/companies/{company_id}/facts` with `501 not_implemented`.
#[utoipa::path(
    get,
    path = "/companies/{company_id}/facts",
    tag = "companies",
    summary = "List the facts of one company, filtered by concept and period",
    description = "For each concept, period, and segment, the response holds the value from the newest filing. The route answers `501` with the code `not_implemented` until its data store is live.",
    params(CompanyPath, FactQuery),
    responses(
        (status = 200, description = "The facts of one page", body = FactList),
        (status = 501, description = "The route has no data yet", body = Problem, content_type = "application/problem+json"),
    ),
)]
// axum requires an async handler. This one has nothing to await.
#[allow(clippy::unused_async)]
pub async fn list_facts() -> ApiError {
    ApiError::NotImplemented
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use xbrl::core::elements::CanonicalElement;
    use xbrl::core::filing::FilingId;
    use xbrl::core::fiscal_period::FiscalPeriod as DomainFiscalPeriod;
    use xbrl::core::period::Period as DomainPeriod;
    use xbrl::core::unit::Unit as DomainUnit;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_fact_list() {
        implements_auto_traits::<FactList>();
    }

    #[test]
    const fn should_implement_auto_traits_for_fact_query() {
        implements_auto_traits::<FactQuery>();
    }

    fn revenue_fact_without_lineage() -> Result<Fact, ApiError> {
        let concept = CanonicalElement::Revenue;
        let value = BigDecimal::from_str("391035000000")
            .expect("Given a hardcoded integer text, parsing it should always succeed");
        let unit = DomainUnit::from_str("USD")
            .expect("Given the hardcoded unit 'USD', parsing it should always succeed");
        let fiscal_period = DomainFiscalPeriod::new("FY")
            .expect("Given the hardcoded period 'FY', the creation should always succeed");
        let period = DomainPeriod::Instant {
            end: NaiveDate::from_ymd_opt(2024, 9, 28)
                .expect("Given a valid hardcoded date, building it should always succeed"),
        };
        let filing_id = FilingId::new("sec", "0000320193-24-000123")
            .expect("Given a hardcoded source and ID, the creation should always succeed");
        let fact = Fact {
            concept: concept.try_into()?,
            value: value.into(),
            unit: unit.into(),
            fiscal_year: FiscalYear::new(2024),
            fiscal_period: fiscal_period.try_into()?,
            period: period.try_into()?,
            segment: None,
            source: FactSource {
                accession_number: filing_id.into(),
                form: Form::new("10-K")
                    .expect("Given the hardcoded form '10-K', the creation should always succeed"),
                filed_at: NaiveDate::from_ymd_opt(2024, 11, 1)
                    .expect("Given a valid hardcoded date, building it should always succeed"),
                url: "https://www.sec.gov/Archives/edgar/data/320193/000032019324000123/"
                    .to_owned(),
            },
            lineage: None,
        };
        Ok(fact)
    }

    #[test]
    fn should_write_the_contract_json_for_a_fact_of_domain_values() -> Result<(), ApiError> {
        let fact = revenue_fact_without_lineage()?;

        let expected_result = json!({
            "concept": "revenue",
            "value": "391035000000",
            "unit": "USD",
            "fiscal_year": 2024,
            "fiscal_period": "fy",
            "period": { "kind": "instant", "end": "2024-09-28" },
            "segment": null,
            "source": {
                "accession_number": "0000320193-24-000123",
                "form": "10-K",
                "filed_at": "2024-11-01",
                "url": "https://www.sec.gov/Archives/edgar/data/320193/000032019324000123/",
            },
        });

        let result = serde_json::to_value(fact)
            .expect("Given a fact of contract types, serializing it should always succeed");

        assert_eq!(result, expected_result);
        Ok(())
    }

    #[test]
    fn should_omit_the_lineage_key_when_the_fact_has_no_lineage() -> Result<(), ApiError> {
        let fact = revenue_fact_without_lineage()?;

        let expected_result = false;

        let result = serde_json::to_value(fact)
            .expect("Given a fact of contract types, serializing it should always succeed")
            .as_object()
            .expect("Given a struct, serde should always serialize it as a JSON object")
            .contains_key("lineage");

        assert_eq!(result, expected_result);
        Ok(())
    }
}
