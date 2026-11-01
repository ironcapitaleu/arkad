//! # Constants
//!
//! The names of the environment variables that set the dates of a run, and the word that names
//! yesterday.

/// The environment variable that sets the first date of the run. It is required.
pub const FROM_ENV_VAR: &str = "ARKAD_FROM";

/// The environment variable that sets the last date of the run. If it is unset, the last date is
/// yesterday.
pub const TO_ENV_VAR: &str = "ARKAD_TO";

/// The value of [`FROM_ENV_VAR`] or [`TO_ENV_VAR`] that names yesterday in New York. Followed by
/// `-N`, it names the day `N` days before yesterday.
pub const YESTERDAY: &str = "yesterday";
