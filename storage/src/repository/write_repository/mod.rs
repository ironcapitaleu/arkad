//! # Write Repository
//!
//! Provides [`WriteRepository`], the trait through which a record is persisted to the store.
//!
//! ## Usage
//!
//! A caller bound to [`WriteRepository`] has `persist` in scope:
//!
//! ```rust
//! use storage::WriteRepository;
//!
//! async fn write_through_a_writer<W: WriteRepository>(writer: &W, record: W::Record) {
//!     let write = writer.persist(record).await;
//!     write.expect("Given a writer that accepts the record, the write should always succeed");
//! }
//! ```
//!
//! The trait carries no read method, so a read from that same caller does not compile:
//!
//! ```compile_fail
//! use storage::WriteRepository;
//!
//! fn read_through_a_writer<W: WriteRepository>(mut writer: W) {
//!     writer.get(todo!());
//! }
//! ```

// Keep the guard above unconstrained: `todo!()` for the argument, the result unused, the receiver
// by value. Anything more specific makes the block fail on that instead of on the missing method.

use async_trait::async_trait;

use crate::error::WriteError;

/// Persists a record to the store.
///
/// Injected as a concrete type — production wires a real backend, tests wire a fake — so callers
/// depend on this trait rather than on a database. Each implementor binds
/// [`WriteRepository::Record`] to its own write-unit.
///
/// # Associated Types
///
/// - [`WriteRepository::Record`]: the write-unit accepted by [`WriteRepository::persist`].
#[async_trait]
pub trait WriteRepository: Send + Sync {
    /// The unit of persistence this repository accepts — one record per
    /// [`WriteRepository::persist`] call. Implementations bind it to their concrete write-unit
    /// (for example, a filing record).
    ///
    /// Bounded by [`Send`] because [`WriteRepository::persist`] moves it across an `async`
    /// boundary.
    type Record: Send;

    /// Persists a single record.
    ///
    /// # Errors
    ///
    /// Returns a [`WriteError`] if the record cannot be persisted.
    async fn persist(&self, record: Self::Record) -> Result<(), WriteError>;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::fake_write_repository::FakeWriteRepository;

    const fn implements_write_repository<T: WriteRepository>() {}

    #[test]
    const fn should_implement_write_repository_when_using_fake_write_repository() {
        implements_write_repository::<FakeWriteRepository<String>>();
    }

    #[tokio::test]
    async fn should_return_ok_when_persisting_through_a_trait_object() {
        let repository: Box<dyn WriteRepository<Record = String>> =
            Box::new(FakeWriteRepository::new());
        let expected_result = Ok(());

        let result = repository.persist("CIK0001067983".to_string()).await;

        assert_eq!(result, expected_result);
    }
}
