//! # Sample Run
//!
//! Provides the `execution_id` and the `User-Agent` of a sample run.

use uuid::Uuid;

use crate::shared::user_agent::UserAgent;

/// The `execution_id` of the sample run.
pub const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);

/// Returns the `User-Agent` of the sample run.
///
/// # Panics
///
/// Never panics, because the hardcoded string has the SEC format.
#[must_use]
pub fn sample_user_agent() -> UserAgent {
    UserAgent::new("Sample Company contact@example.com")
        .expect("Given a hardcoded SEC-format user agent, validation should always succeed")
}
