//! # Lease Store
//!
//! Provides [`LeaseStore`], the port through which one run of a job kind takes the lease that lets
//! it write, and each task reads it.
//!
//! ## Usage
//!
//! A task checks that its plan still holds the lease before its first write:
//!
//! ```rust
//! use storage::{JobKind, LeaseError, LeaseStore};
//!
//! async fn holds_the_lease<S: LeaseStore>(
//!     store: &S,
//!     kind: &JobKind,
//!     plan_id: &str,
//! ) -> Result<bool, LeaseError> {
//!     let current = store.current(kind).await?;
//!     Ok(current.is_some_and(|(lease, _generation)| lease.plan_id == plan_id))
//! }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::LeaseError;
use crate::shared::{JobKind, Lease, LeaseGeneration};

/// Takes and reads the lease of a job kind, so one run of the job kind writes at a time.
///
/// A store holds at most one lease per [`JobKind`]. A lease expires at its `expires_at`. Then
/// another plan can take it. Each lease the store writes gets a new [`LeaseGeneration`].
///
/// It is not a subtrait of a repository trait, because a store of leases reads and writes no
/// records.
#[async_trait]
pub trait LeaseStore: Send + Sync {
    /// Takes the lease of the job kind for the plan, until `expires_at`.
    ///
    /// Writes the lease if the job kind has none, or if its lease expired at or before the time of
    /// the call. Returns the generation of the written lease. Two calls at the same time never both
    /// succeed.
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`LeaseError::LeaseHeld`] with the lease that holds the job kind, if it has not expired.
    ///   The store changes nothing. This holds for the plan of the lease too.
    /// - [`LeaseError::Backend`] if the store cannot read or write the lease.
    async fn acquire(
        &self,
        kind: &JobKind,
        plan_id: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<LeaseGeneration, LeaseError>;

    /// Returns the lease of the job kind and its generation, or `None` if the job kind has none.
    ///
    /// Returns an expired lease too. The caller compares `expires_at` with its own time.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError::Backend`] if the store cannot read the lease.
    async fn current(&self, kind: &JobKind)
    -> Result<Option<(Lease, LeaseGeneration)>, LeaseError>;
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::fake_lease_store::FakeLeaseStore;

    const fn implements_lease_store<T: LeaseStore>() {}
    #[test]
    const fn should_implement_lease_store_for_fake_lease_store() {
        implements_lease_store::<FakeLeaseStore>();
    }

    fn transform_notes() -> JobKind {
        JobKind::new("transform-notes")
            .expect("Given a valid hardcoded job kind, the creation should always succeed")
    }

    #[tokio::test]
    async fn should_return_the_lease_when_reading_through_a_trait_object() {
        let store: Box<dyn LeaseStore> = Box::new(FakeLeaseStore::default());
        let kind = transform_notes();
        let expires_at = DateTime::<Utc>::MAX_UTC - TimeDelta::days(1);
        store
            .acquire(&kind, "plan-a", expires_at)
            .await
            .expect("Given a job kind without a lease, the acquire should always succeed");

        let expected_result = Some((
            Lease {
                plan_id: "plan-a".to_owned(),
                expires_at,
            },
            LeaseGeneration(1),
        ));

        let result = store
            .current(&kind)
            .await
            .expect("Given the fake, the read should always succeed");

        assert_eq!(result, expected_result);
    }
}
