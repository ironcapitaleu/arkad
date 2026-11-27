//! # Lease
//!
//! Provides [`Lease`], the record that names the one run of a job kind that can write, and the
//! [`LeaseGeneration`] a repository gives each lease it writes.

use std::fmt::{self, Display, Formatter};

use jiff::Timestamp;
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
    pub expires_at: Timestamp,
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
