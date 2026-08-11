//! # Repository — the neutral persistence port
//!
//! Provides [`Repository`], the sole persistence port the pipeline injects. It is *neutral*: a
//! single associated [`Repository::Record`] and one [`Repository::persist`] write method, blind to
//! how or where the record lands. It names no tiers and no backend — the tier capability traits and
//! the composing `SecRepository` (which binds `type Record` to a concrete filing record) are
//! deferred to later slices.
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

/// The neutral persistence port the pipeline depends on.
///
/// Backend-blind by design: it exposes a single write-unit ([`Repository::Record`]) and a single
/// write method ([`Repository::persist`]), and names neither the storage tiers nor any concrete
/// backend. Swapping the physical store is a new implementation of this trait, not a pipeline
/// change. Consistent with the codebase's non-object-safe trait style, implementations are injected
/// by concrete type, never behind `dyn`.
#[async_trait]
pub trait Repository: Send + Sync {
    /// The unit of persistence this repository accepts — one record per [`Repository::persist`]
    /// call. Implementations bind it to their concrete write-unit (for example, a filing record).
    type Record;

    /// Persists a single record.
    ///
    /// Always a write; the return type is the narrow [`WriteError`] class, never the read class, so
    /// illegal states are unrepresentable.
    ///
    /// # Errors
    ///
    /// Returns a [`WriteError`] if the record cannot be persisted — a conflict with existing data,
    /// a violated integrity invariant, or a backend-level failure.
    async fn persist(&self, record: Self::Record) -> Result<(), WriteError>;
}
