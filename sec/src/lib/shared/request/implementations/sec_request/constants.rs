//! # SEC Request Constants
//!
//! URL fragments for building SEC company-facts endpoints, joined around a CIK.

/// The company-facts URL prefix, placed before the CIK.
pub const SEC_COMPANY_FACTS_URL_PREFIX: &str = "https://data.sec.gov/api/xbrl/companyfacts/CIK";

/// The company-facts URL suffix, placed after the CIK (the data is served as JSON).
pub const SEC_COMPANY_FACTS_URL_SUFFIX: &str = ".json";
