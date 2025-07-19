//! # SEC Request Constants Module
//!
//! This module defines constants related to the construction of HTTP requests to SEC API endpoints.
//! These constants are used throughout the [`crate::shared::sec_request`] module and by state machine implementations
//! that require consistent URL formatting for SEC data retrieval.
//!
//! ## Constants
//! - [`SEC_REQUEST_URL_PREFIX`]: The base URL prefix for SEC CIK submission data endpoints.
//! - [`SEC_REQUEST_URL_SUFFIX`]: The file extension suffix for SEC submission data requests.
//! - [`SEC_CIK_BERKSHIRE_HATHAWAY_URL`]: A complete URL example for Berkshire Hathaway's submission data.
//!
//! ## Usage
//! Use these constants when implementing or testing SEC request construction logic in state data modules
//! and in shared utilities for building properly formatted SEC API URLs.

/// The base URL prefix for SEC CIK submission data endpoints.
///
/// This constant defines the common prefix used for all SEC submission data requests.
/// It should be combined with a CIK and the appropriate suffix to form a complete URL.
pub const SEC_REQUEST_URL_PREFIX: &str = "https://data.sec.gov/submissions/CIK";

/// The file extension suffix for SEC submission data requests.
///
/// This constant defines the file format extension used for SEC submission data endpoints.
/// All SEC submission data is provided in JSON format.
pub const SEC_REQUEST_URL_SUFFIX: &str = ".json";

/// A complete URL for Berkshire Hathaway's SEC submission data.
///
/// This constant provides a real-world example of a properly formatted SEC submission URL
/// and can be used in tests, examples, or as a reference value when working with SEC request
/// construction logic throughout the codebase.
pub const SEC_CIK_BERKSHIRE_HATHAWAY_URL: &str =
    "https://data.sec.gov/submissions/CIK0001067983.json";
