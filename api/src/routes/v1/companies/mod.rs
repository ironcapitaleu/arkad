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

use serde::Deserialize;
use utoipa::IntoParams;

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

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_company_path() {
        implements_auto_traits::<CompanyPath>();
    }
}
