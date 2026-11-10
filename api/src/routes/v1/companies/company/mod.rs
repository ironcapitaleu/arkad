//! # Company
//!
//! Provides the [`list_companies`] handler of `GET /v1/companies` and the [`get_company`]
//! handler of `GET /v1/companies/{company_id}`.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::CompanyPath;
use crate::error::{ApiError, Problem};

/// One company with its identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Company {
    /// The ID that the platform gave the company. It never changes.
    #[schema(example = "cmp_01JB6Q2W5V8X3K9M7N4P2R1T0S")]
    pub company_id: String,
    /// The name of the company.
    #[schema(example = "Alphabet Inc.")]
    pub name: String,
    /// Whether the company still files with the SEC.
    pub status: CompanyStatus,
    /// The current tickers of every share class.
    #[schema(example = json!(["GOOGL", "GOOG"]))]
    pub tickers: Vec<String>,
    /// The current CIK, or `null`.
    #[schema(required = true, example = "0001652044")]
    pub cik: Option<String>,
    /// The current LEI, or `null`.
    #[schema(required = true, example = "5493006MHB84DD0ZWV18")]
    pub lei: Option<String>,
    /// The SIC code of the industry.
    #[schema(example = "7370")]
    pub sic: String,
    /// The sector, from the SIC code.
    #[schema(example = "Information Technology")]
    pub sector: String,
    /// The last day of the fiscal year, as `MM-DD`.
    #[schema(example = "12-31")]
    pub fiscal_year_end: String,
    /// Every known identifier of the company, with its date range.
    pub identifiers: Vec<CompanyIdentifier>,
}

/// Whether a company still files with the SEC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CompanyStatus {
    /// The company files with the SEC.
    Active,
    /// The company stopped filing.
    Inactive,
}

/// One identifier of a company, valid for a date range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct CompanyIdentifier {
    /// The kind of identifier.
    pub scheme: IdentifierScheme,
    /// The identifier.
    #[schema(example = "0001652044")]
    pub value: String,
    /// The first day the identifier was valid, as `YYYY-MM-DD`.
    #[schema(format = Date, example = "2015-10-02")]
    pub valid_from: String,
    /// The last day the identifier was valid, or `null` if it is current.
    #[schema(required = true, format = Date, example = "2015-10-01")]
    pub valid_to: Option<String>,
    /// `false` for an LEI that no one checked against GLEIF.
    pub verified: bool,
}

/// The kind of a company identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IdentifierScheme {
    /// A Central Index Key of the SEC.
    Cik,
    /// A Legal Entity Identifier.
    Lei,
}

/// The body of a `GET /v1/companies/{company_id}` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct CompanyEnvelope {
    /// The company.
    pub data: Company,
}

/// The body of a `GET /v1/companies` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct CompanyList {
    /// The companies of this page, sorted by `company_id`.
    pub data: Vec<Company>,
    /// The cursor of the next page, or `null` on the last page.
    #[schema(required = true)]
    pub next_cursor: Option<String>,
    /// The time that fixes the data of every page, RFC 3339 in UTC.
    #[schema(format = DateTime, example = "2026-10-30T06:00:00Z")]
    pub as_of: String,
}

/// The query parameters of `GET /v1/companies`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CompanySearch {
    /// A current ticker. Case does not matter. Send at most one of `ticker`, `cik`, and `lei`.
    #[param(example = "GOOGL")]
    pub ticker: Option<String>,
    /// Any CIK that the company ever used, with or without leading zeros.
    #[param(example = "1652044")]
    pub cik: Option<String>,
    /// Any LEI that the company ever had.
    #[param(example = "5493006MHB84DD0ZWV18")]
    pub lei: Option<String>,
    /// The number of companies per page. Default: 100.
    #[param(minimum = 1, maximum = 1000)]
    pub limit: Option<u32>,
    /// The `next_cursor` of the previous page.
    pub cursor: Option<String>,
}

/// Answers `GET /v1/companies` with `501 not_implemented`.
#[utoipa::path(
    get,
    path = "/companies",
    tag = "companies",
    summary = "List the covered companies, or find a company by one identifier",
    description = "A lookup that matches nothing answers `200` with an empty `data` list. The route answers `501` with the code `not_implemented` until its data store is live.",
    params(CompanySearch),
    responses(
        (status = 200, description = "The companies of one page", body = CompanyList),
        (status = 501, description = "The route has no data yet", body = Problem, content_type = "application/problem+json"),
    ),
)]
// axum requires an async handler. This one has nothing to await.
#[allow(clippy::unused_async)]
pub async fn list_companies() -> ApiError {
    ApiError::NotImplemented
}

/// Answers `GET /v1/companies/{company_id}` with `501 not_implemented`.
#[utoipa::path(
    get,
    path = "/companies/{company_id}",
    tag = "companies",
    summary = "Get one company with its identifiers",
    description = "The route answers `501` with the code `not_implemented` until its data store is live.",
    params(CompanyPath),
    responses(
        (status = 200, description = "The company", body = CompanyEnvelope),
        (status = 501, description = "The route has no data yet", body = Problem, content_type = "application/problem+json"),
    ),
)]
// axum requires an async handler. This one has nothing to await.
#[allow(clippy::unused_async)]
pub async fn get_company() -> ApiError {
    ApiError::NotImplemented
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_company_envelope() {
        implements_auto_traits::<CompanyEnvelope>();
    }

    #[test]
    const fn should_implement_auto_traits_for_company_list() {
        implements_auto_traits::<CompanyList>();
    }

    #[test]
    const fn should_implement_auto_traits_for_company_search() {
        implements_auto_traits::<CompanySearch>();
    }
}
