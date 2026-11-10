//! # Facts
//!
//! Provides the [`list_facts`] handler of `GET /v1/companies/{company_id}/facts`.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::{CompanyPath, FiscalPeriod};
use crate::error::{ApiError, Problem};

/// One value of one concept for one period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Fact {
    /// The concept, from `GET /v1/concepts`.
    #[schema(example = "revenue")]
    pub concept: String,
    /// The exact decimal value in whole units of `unit`, as a string.
    #[schema(example = "391035000000")]
    pub value: String,
    /// An ISO 4217 currency code, `shares`, `usd_per_share`, or `pure` for a ratio.
    #[schema(example = "USD")]
    pub unit: String,
    /// The fiscal year of the value.
    #[schema(example = 2024)]
    pub fiscal_year: i32,
    /// The fiscal period of the value.
    pub fiscal_period: FiscalPeriod,
    /// The period that the value covers.
    pub period: FactPeriod,
    /// `null` for a consolidated total. With `include=segments`, the axis and the member.
    #[schema(required = true, value_type = Option<Object>)]
    pub segment: Option<serde_json::Value>,
    /// The filing that the value comes from.
    pub source: FactSource,
    /// How the value maps to its concept. Present only with `include=lineage`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<Lineage>,
}

/// The period that a fact covers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FactPeriod {
    /// A range of days, for example a fiscal year of an income statement.
    Duration {
        /// The first day, as `YYYY-MM-DD`.
        #[schema(format = Date, example = "2023-10-01")]
        start: String,
        /// The last day, as `YYYY-MM-DD`.
        #[schema(format = Date, example = "2024-09-28")]
        end: String,
    },
    /// One day, for example a balance sheet date.
    Instant {
        /// The day, as `YYYY-MM-DD`.
        #[schema(format = Date, example = "2024-09-28")]
        end: String,
    },
}

/// The filing that a fact comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct FactSource {
    /// The accession number of the filing.
    #[schema(example = "0000320193-24-000123")]
    pub accession_number: String,
    /// The form type of the filing.
    #[schema(example = "10-K")]
    pub form: String,
    /// The filing date, as `YYYY-MM-DD`.
    #[schema(format = Date, example = "2024-11-01")]
    pub filed_at: String,
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
    #[schema(required = true, example = 2021)]
    pub history_limited_from: Option<i32>,
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
    pub fiscal_year_from: Option<i32>,
    /// The last fiscal year, inclusive.
    #[param(example = 2024)]
    pub fiscal_year_to: Option<i32>,
    /// `segments`, `lineage`, or both, separated by commas. Full depth only.
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
    use pretty_assertions::assert_eq;
    use serde_json::json;

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

    #[test]
    fn should_serialize_only_the_end_when_the_period_is_an_instant() {
        let period = FactPeriod::Instant {
            end: "2024-09-28".to_owned(),
        };

        let expected_result = json!({ "kind": "instant", "end": "2024-09-28" });

        let result = serde_json::to_value(period)
            .expect("Given an enum of strings, serializing it should always succeed");

        assert_eq!(result, expected_result);
    }
}
