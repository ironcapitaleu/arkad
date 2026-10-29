//! # SEC Request Constants
//!
//! URL fragments for SEC endpoints. The company-facts URL puts a CIK between its prefix and suffix.
//! The Notes URL puts a Notes file name after its prefix.

/// The company-facts URL prefix, placed before the CIK.
pub const SEC_COMPANY_FACTS_URL_PREFIX: &str = "https://data.sec.gov/api/xbrl/companyfacts/CIK";

/// The company-facts URL suffix, placed after the CIK (the data is served as JSON).
pub const SEC_COMPANY_FACTS_URL_SUFFIX: &str = ".json";

/// The Notes URL prefix, placed before the file name of a Notes file.
pub const SEC_NOTES_URL_PREFIX: &str =
    "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/";
