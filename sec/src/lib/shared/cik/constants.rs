//! # CIK Constants Module
//!
//! This module defines constants related to the formatting and validation of SEC Central Index Keys (CIKs).
//! These constants are used throughout the [`crate::shared::cik`] module and by state machine implementations
//! that require strict validation of CIK input and output data.
//!
//! ## Constants
//! - [`CIK_LENGTH`]: The required length for a valid CIK, enforced by parsing and validation routines.
//!
//! ## Usage
//! Use these constants when implementing or testing CIK validation logic in state data modules such as
//! [`crate::implementations::states::extract::validate_cik_format::data`] and in shared utilities.

/// The required length for a valid CIK (Central Index Key).
///
/// This constant defines the exact number of digits a CIK must have to be considered valid
/// according to SEC requirements.
pub const CIK_LENGTH: usize = 10;

/// The raw CIK (Central Index Key) assigned to Berkshire Hathaway Inc. by the SEC, but without leading zeroes.
///
/// This constant can be used in tests, examples, or as a reference value when working with CIK-related
/// validation and parsing logic throughout the codebase.
pub const BERKSHIRE_HATHAWAY_CIK_RAW: &str = "1067983";

/// The CIK (Central Index Key) assigned to Berkshire Hathaway Inc. by the SEC.
///
/// This constant can be used in tests, examples, or as a reference value when working with CIK-related
/// validation and parsing logic throughout the codebase.
pub const BERKSHIRE_HATHAWAY_CIK: &str = "0001067983";
