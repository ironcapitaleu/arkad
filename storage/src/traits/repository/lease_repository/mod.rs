//! # Lease Repository
//!
//! Provides [`LeaseRepository`], the port through which one run of a job kind takes the lease that
//! lets it write, and each task reads it.
//!
//! ## Usage
//!
//! A task checks that its plan still holds the lease before its first write:
//!
//! ```rust
//! use storage::{JobKind, LeaseError, LeaseRepository};
//!
//! async fn holds_the_lease<S: LeaseRepository>(
//!     repository: &S,
//!     kind: &JobKind,
//!     plan_id: &str,
//! ) -> Result<bool, LeaseError> {
//!     let current = repository.current(kind).await?;
//!     Ok(current.is_some_and(|(lease, _generation)| lease.plan_id == plan_id))
//! }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::LeaseError;
use crate::shared::{JobKind, Lease, LeaseGeneration};

/// Takes and reads the lease of a job kind, so one run of the job kind writes at a time.
///
/// A repository holds at most one lease per [`JobKind`]. A lease expires at its `expires_at`. Then
/// another plan can take it. Each lease the repository writes gets a new [`LeaseGeneration`].
#[async_trait]
pub trait LeaseRepository: Send + Sync {
    /// Takes the lease of the job kind for the plan, until `expires_at`.
    ///
    /// Writes the lease if the job kind has none, or if its lease expired at or before the time of
    /// the call. Returns the generation of the written lease. If `expires_at` is after the time of
    /// the calls, two calls at the same time never both succeed. If the lease expires before a call
    /// reads it, that call replaces it.
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`LeaseError::LeaseHeld`] with the lease that holds the job kind, if it has not expired.
    ///   The repository changes nothing. This holds for the plan of the lease too.
    /// - [`LeaseError::Backend`] if the repository cannot read or write the lease.
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
    /// Returns [`LeaseError::Backend`] if the repository cannot read the lease.
    async fn current(&self, kind: &JobKind)
    -> Result<Option<(Lease, LeaseGeneration)>, LeaseError>;
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::fake_lease_repository::FakeLeaseRepository;

    const fn implements_lease_repository<T: LeaseRepository>() {}
    #[test]
    const fn should_implement_lease_repository_for_fake_lease_repository() {
        implements_lease_repository::<FakeLeaseRepository>();
    }

    fn transform_notes() -> JobKind {
        JobKind::new("transform-notes")
            .expect("Given a valid hardcoded job kind, the creation should always succeed")
    }

    #[tokio::test]
    async fn should_return_lease_held_from_the_fake_when_the_lease_has_not_expired() {
        let repository = FakeLeaseRepository::default();
        let kind = transform_notes();
        let expires_at = DateTime::<Utc>::MAX_UTC - TimeDelta::days(1);
        repository
            .acquire(&kind, "plan-a", expires_at)
            .await
            .expect("Given a job kind without a lease, the acquire should always succeed");
        let plan_id = "plan-b";

        let expected_result = Err(LeaseError::LeaseHeld {
            kind: transform_notes(),
            lease: Lease {
                plan_id: "plan-a".to_owned(),
                expires_at,
            },
        });

        let result = repository.acquire(&kind, plan_id, expires_at).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_next_generation_from_the_fake_when_the_lease_expired() {
        let repository = FakeLeaseRepository::default();
        let kind = transform_notes();
        repository
            .acquire(&kind, "plan-a", DateTime::<Utc>::UNIX_EPOCH)
            .await
            .expect("Given a job kind without a lease, the acquire should always succeed");
        let plan_id = "plan-b";
        let expires_at = DateTime::<Utc>::MAX_UTC;

        let expected_result = Ok(LeaseGeneration(2));

        let result = repository.acquire(&kind, plan_id, expires_at).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_lease_when_reading_through_a_trait_object() {
        let repository: Box<dyn LeaseRepository> = Box::new(FakeLeaseRepository::default());
        let kind = transform_notes();
        let expires_at = DateTime::<Utc>::MAX_UTC - TimeDelta::days(1);
        repository
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

        let result = repository
            .current(&kind)
            .await
            .expect("Given the fake, the read should always succeed");

        assert_eq!(result, expected_result);
    }
}
