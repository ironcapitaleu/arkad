//! # Fake Lease Repository
//!
//! Provides [`FakeLeaseRepository`], an in-memory [`LeaseRepository`] for tests.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::LeaseError;
use crate::shared::{JobKind, Lease, LeaseGeneration};
use crate::traits::repository::LeaseRepository;

/// An in-memory [`LeaseRepository`] that keeps one lease and its generation per job kind.
///
/// It follows the contract of the trait: it replaces a lease only after the lease expired, and
/// gives each written lease the next generation, starting at 1.
#[derive(Debug, Default)]
pub struct FakeLeaseRepository {
    leases: Mutex<HashMap<JobKind, (Lease, LeaseGeneration)>>,
}

#[async_trait]
impl LeaseRepository for FakeLeaseRepository {
    async fn acquire(
        &self,
        kind: &JobKind,
        plan_id: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<LeaseGeneration, LeaseError> {
        let mut leases = self
            .leases
            .lock()
            .expect("Given no test panics while it holds the lock, the lock should never poison");
        let generation = match leases.get(kind) {
            Some((lease, _)) if lease.expires_at > Utc::now() => {
                return Err(LeaseError::LeaseHeld {
                    kind: kind.clone(),
                    lease: lease.clone(),
                });
            }
            Some((_, LeaseGeneration(previous))) => LeaseGeneration(previous + 1),
            None => LeaseGeneration(1),
        };
        let lease = Lease {
            plan_id: plan_id.to_owned(),
            expires_at,
        };
        leases.insert(kind.clone(), (lease, generation));
        drop(leases);
        Ok(generation)
    }

    async fn current(
        &self,
        kind: &JobKind,
    ) -> Result<Option<(Lease, LeaseGeneration)>, LeaseError> {
        let leases = self
            .leases
            .lock()
            .expect("Given no test panics while it holds the lock, the lock should never poison");
        Ok(leases.get(kind).cloned())
    }
}
