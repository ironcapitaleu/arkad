//! # SEC Request Constants Module
//!
//! This module defines constants related to the construction of HTTP requests to SEC API endpoints.
//!
//! ## Constants
//! - [`SEC_COMPANY_FACTS_URL_PREFIX`]: The base URL prefix for SEC company facts endpoints.
//! - [`SEC_COMPANY_FACTS_URL_SUFFIX`]: The file extension suffix for SEC company facts requests.
//!
//! ## Usage
//! Use these constants when implementing or testing SEC request construction logic for company facts endpoints.

/// The base URL prefix for SEC company facts endpoints.
///
/// This constant defines the common prefix used for all SEC company facts requests.
/// It should be combined with a CIK and the appropriate suffix to form a complete URL.
pub const SEC_COMPANY_FACTS_URL_PREFIX: &str = "https://data.sec.gov/api/xbrl/companyfacts/CIK";

/// The file extension suffix for SEC company facts requests.
///
/// This constant defines the file format extension used for SEC company facts endpoints.
/// All SEC company facts data is provided in JSON format.
pub const SEC_COMPANY_FACTS_URL_SUFFIX: &str = ".json";
