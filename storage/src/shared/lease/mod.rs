//! # Lease
//!
//! Provides [`Lease`], the record that names the one run of a job kind that can write, and the
//! [`LeaseGeneration`] a repository gives each lease it writes.

use std::fmt::{self, Display, Formatter};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The record that names the one run of a job kind that can write: its plan, and the time the
/// lease expires.
///
/// After `expires_at`, a [`LeaseRepository`](crate::LeaseRepository) lets another plan take the
/// lease.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Lease {
    /// The ID of the plan that holds the lease.
    pub plan_id: String,
    /// The time the lease expires.
    pub expires_at: DateTime<Utc>,
}

/// The number a [`LeaseRepository`](crate::LeaseRepository) gives each lease it writes for a job
/// kind.
///
/// Each write of a lease for a job kind gets a new generation. A caller that holds the generation
/// of a lease can tell whether another plan has replaced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LeaseGeneration(pub u64);

impl Display for LeaseGeneration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    /// Reads a stored lease and writes it back as the lease repository does.
    fn rewritten(stored: &str) -> String {
        let lease: Lease = serde_json::from_str(stored)
            .expect("Given lease JSON with every field, deserialization should always succeed");
        serde_json::to_string(&lease)
            .expect("Given a lease of a string and a time, serialization should always succeed")
    }

    #[test]
    fn should_write_the_stored_bytes_back_when_the_expiry_has_no_fraction() {
        let stored = r#"{"plan_id":"plan-a","expires_at":"2026-11-22T10:00:00Z"}"#;

        let expected_result = stored;

        let result = rewritten(stored);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_stored_bytes_back_when_the_expiry_has_milliseconds() {
        let stored = r#"{"plan_id":"plan-a","expires_at":"2026-11-22T10:00:00.500Z"}"#;

        let expected_result = stored;

        let result = rewritten(stored);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_stored_bytes_back_when_the_expiry_has_microseconds() {
        let stored = r#"{"plan_id":"plan-a","expires_at":"2026-11-22T10:00:00.123456Z"}"#;

        let expected_result = stored;

        let result = rewritten(stored);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_stored_bytes_back_when_the_expiry_has_nanoseconds() {
        let stored = r#"{"plan_id":"plan-a","expires_at":"2026-11-22T10:00:00.123456789Z"}"#;

        let expected_result = stored;

        let result = rewritten(stored);

        assert_eq!(result, expected_result);
    }
}
