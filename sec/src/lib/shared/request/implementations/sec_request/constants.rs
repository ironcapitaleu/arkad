//! # SEC Request Constants
//!
//! URL fragments for building SEC endpoints: the company-facts URL joins its fragments around a
//! CIK, and the Notes URL puts its prefix before a Notes file name.

/// The company-facts URL prefix, placed before the CIK.
pub const SEC_COMPANY_FACTS_URL_PREFIX: &str = "https://data.sec.gov/api/xbrl/companyfacts/CIK";

/// The company-facts URL suffix, placed after the CIK (the data is served as JSON).
pub const SEC_COMPANY_FACTS_URL_SUFFIX: &str = ".json";

/// The Notes URL prefix, placed before the file name of a Notes file.
pub const SEC_NOTES_URL_PREFIX: &str =
    "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/";
