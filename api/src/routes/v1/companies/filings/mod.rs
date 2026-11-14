//! # Filings
//!
//! Provides the [`list_filings`] handler of `GET /v1/companies/{company_id}/filings`.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use xbrl::core::fiscal_year::FiscalYear;
use xbrl::core::form::Form;

use super::CompanyPath;
use crate::error::{ApiError, Problem};
use crate::shared::accession_number::AccessionNumber;
use crate::shared::fiscal_period::FiscalPeriod;

/// One filing of a company.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Filing {
    /// The accession number of the filing.
    pub accession_number: AccessionNumber,
    /// The form type of the filing.
    #[schema(value_type = String, example = "10-K")]
    pub form: Form,
    /// The filing date.
    #[schema(value_type = String, format = Date, example = "2024-11-01")]
    pub filed_at: NaiveDate,
    /// The last day of the period that the filing reports on.
    #[schema(value_type = String, format = Date, example = "2024-09-28")]
    pub period_end: NaiveDate,
    /// The fiscal year that the filing reports on.
    #[schema(value_type = u16, example = 2024)]
    pub fiscal_year: FiscalYear,
    /// The fiscal period that the filing reports on.
    pub fiscal_period: FiscalPeriod,
    /// The folder of the filing on SEC EDGAR.
    #[schema(example = "https://www.sec.gov/Archives/edgar/data/320193/000032019324000123/")]
    pub url: String,
}

/// The body of a `GET /v1/companies/{company_id}/filings` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct FilingList {
    /// The filings of this page, newest first.
    pub data: Vec<Filing>,
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

/// The query parameters of `GET /v1/companies/{company_id}/filings`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FilingQuery {
    /// One or more form types, separated by commas.
    #[param(example = "10-K,10-Q")]
    pub form: Option<String>,
    /// The first filing date, inclusive, as `YYYY-MM-DD`.
    #[param(value_type = Option<String>, format = Date, example = "2020-01-01")]
    pub filed_from: Option<NaiveDate>,
    /// The last filing date, inclusive, as `YYYY-MM-DD`.
    #[param(value_type = Option<String>, format = Date, example = "2024-12-31")]
    pub filed_to: Option<NaiveDate>,
    /// The number of filings per page. Default: 100.
    #[param(minimum = 1, maximum = 1000)]
    pub limit: Option<u32>,
    /// The `next_cursor` of the previous page.
    pub cursor: Option<String>,
}

/// Answers `GET /v1/companies/{company_id}/filings` with `501 not_implemented`.
#[utoipa::path(
    get,
    path = "/companies/{company_id}/filings",
    tag = "companies",
    summary = "List the filings of one company, newest first",
    description = "The route answers `501` with the code `not_implemented` until its data store is live.",
    params(CompanyPath, FilingQuery),
    responses(
        (status = 200, description = "The filings of one page", body = FilingList),
        (status = 501, description = "The route has no data yet", body = Problem, content_type = "application/problem+json"),
    ),
)]
// axum requires an async handler. This one has nothing to await.
#[allow(clippy::unused_async)]
pub async fn list_filings() -> ApiError {
    ApiError::NotImplemented
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_filing_list() {
        implements_auto_traits::<FilingList>();
    }

    #[test]
    const fn should_implement_auto_traits_for_filing_query() {
        implements_auto_traits::<FilingQuery>();
    }
}
