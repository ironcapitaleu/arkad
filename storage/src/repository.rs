//! # Repository — the neutral persistence port
//!
//! Provides [`Repository`], the persistence port callers inject. It is *neutral*: a single
//! associated [`Repository::Record`] and one [`Repository::persist`] write method, blind to how or
//! where the record lands. It names no tiers and no concrete backend.
//!
//! ## Usage
//!
//! ```rust
//! use async_trait::async_trait;
//! use storage::{Repository, WriteError};
//!
//! struct InMemoryRepository;
//!
//! #[async_trait]
//! impl Repository for InMemoryRepository {
//!     type Record = String;
//!
//!     async fn persist(&self, _record: Self::Record) -> Result<(), WriteError> {
//!         Ok(())
//!     }
//! }
//! ```

use async_trait::async_trait;

use crate::error::WriteError;

/// A neutral, backend-blind persistence port.
///
/// Exists as a trait so callers depend on an abstract persistence contract rather than a concrete
/// database: swapping the physical backend is a new implementation behind the same port, not a
/// change to its callers. It exposes a single write-unit ([`Repository::Record`]) and a single
/// write method ([`Repository::persist`]), naming neither the storage tiers nor any concrete
/// backend. Implementors are wired in by concrete type or generic parameter (static dispatch),
/// not erased behind `dyn`.
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
    /// Always a write; the return type is the narrow [`WriteError`] class, so illegal states are
    /// unrepresentable.
    ///
    /// # Errors
    ///
    /// Returns a [`WriteError`] if the record cannot be persisted:
    /// - [`WriteError::ConflictingWrite`] — the record conflicts with data already present.
    /// - [`WriteError::FailedIntegrityCheck`] — the record violates a data-integrity invariant.
    /// - [`WriteError::Backend`] — the write failed at the backend level.
    async fn persist(&self, record: Self::Record) -> Result<(), WriteError>;
}
