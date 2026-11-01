//! # Constants
//!
//! The names of the environment variables that set the dates of a run, the word that names
//! yesterday, and the limits of a run.

use std::time::Duration;

/// The environment variable that sets the first date of the run. It is required.
pub const FROM_ENV_VAR: &str = "ARKAD_FROM";

/// The environment variable that sets the last date of the run. If it is unset, the last date is
/// yesterday.
pub const TO_ENV_VAR: &str = "ARKAD_TO";

/// The value of [`FROM_ENV_VAR`] or [`TO_ENV_VAR`] that names yesterday in New York. Followed by
/// `-N`, it names the day `N` days before yesterday.
pub const YESTERDAY: &str = "yesterday";

/// The number of dates, yesterday and the days before it, whose missing daily index counts as
/// pending. A pending date does not fail the run, because a later run reads it again.
pub const PENDING_DAYS: u64 = 3;

/// The number of filing machines, and of `submissions` machines, that run at once. The rate
/// limiter of the SEC client caps the requests, so a higher value does not make a run faster.
pub const IN_FLIGHT: usize = 4;

/// The waits before the second and the third attempt of a machine that the SEC answered with 503.
/// After the third attempt the machine fails.
pub const RETRY_DELAYS: [Duration; 2] = [Duration::from_secs(10), Duration::from_secs(60)];
