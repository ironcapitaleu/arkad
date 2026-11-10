//! # Companies
//!
//! Provides the routes under `/v1/companies` and the types they share.
//!
//! The routes need data from a store that no route can query yet. So each handler answers
//! `501 not_implemented`, while the spec shows the parameters and the response shape of the
//! contract.
//!
//! ## Modules
//!
//! - [`company`]: `GET /v1/companies` and `GET /v1/companies/{company_id}`.
//! - [`facts`]: `GET /v1/companies/{company_id}/facts`.
//! - [`filings`]: `GET /v1/companies/{company_id}/filings`.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

pub mod company;
pub mod facts;
pub mod filings;

/// The path parameter of a route that names one company.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct CompanyPath {
    /// The ID that the platform gave the company. A client must treat it as opaque.
    #[param(example = "cmp_01JB6Q2W5V8X3K9M7N4P2R1T0S")]
    pub company_id: String,
}

/// The fiscal period of a fact or a filing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FiscalPeriod {
    /// The full fiscal year.
    Fy,
    /// The first quarter of the fiscal year.
    Q1,
    /// The second quarter of the fiscal year.
    Q2,
    /// The third quarter of the fiscal year.
    Q3,
    /// The fourth quarter of the fiscal year.
    Q4,
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_company_path() {
        implements_auto_traits::<CompanyPath>();
    }

    #[test]
    const fn should_implement_auto_traits_for_fiscal_period() {
        implements_auto_traits::<FiscalPeriod>();
    }
}
