//! # CIK Constants
//!
//! Formatting constants for SEC Central Index Keys, plus reference CIK values for tests and examples.

/// The exact number of digits a valid CIK must have.
pub const CIK_LENGTH: usize = 10;

/// Berkshire Hathaway Inc.'s CIK without leading zeros, as a reference value for tests and examples.
pub const BERKSHIRE_HATHAWAY_CIK_RAW: &str = "1067983";

/// Berkshire Hathaway Inc.'s CIK in full [`CIK_LENGTH`]-digit form, as a reference value for tests and examples.
pub const BERKSHIRE_HATHAWAY_CIK: &str = "0001067983";
