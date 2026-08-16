//! # Repository
//!
//! Provides [`Repository`], the trait through which a record is persisted to the store.

use async_trait::async_trait;

use crate::error::WriteError;

/// Persists a record to the store.
///
/// Injected as a concrete type — production wires a real backend, tests wire a fake — so callers
/// depend on this trait rather than on a database. Each implementor binds [`Repository::Record`]
/// to its own write-unit.
///
/// # Associated Types
///
/// - [`Repository::Record`]: the write-unit accepted by [`Repository::persist`].
#[async_trait]
pub trait Repository: Send + Sync {
    /// The unit of persistence this repository accepts — one record per [`Repository::persist`]
    /// call. Implementations bind it to their concrete write-unit (for example, a filing record).
    ///
    /// Bounded by [`Send`] because [`Repository::persist`] moves it across an `async` boundary.
    type Record: Send;

    /// Persists a single record.
    ///
    /// # Errors
    ///
    /// Returns a [`WriteError`] if the record cannot be persisted:
    /// - [`WriteError::ConflictingWrite`] — the record conflicts with data already present.
    /// - [`WriteError::FailedIntegrityCheck`] — the record violates a data-integrity invariant.
    /// - [`WriteError::Backend`] — the write failed at the backend level.
    async fn persist(&self, record: Self::Record) -> Result<(), WriteError>;
}
