//! # Write Repository
//!
//! Provides [`WriteRepository`], the trait through which a record is persisted to the store.
//!
//! ## Usage
//!
//! A caller bound to [`WriteRepository`] has [`persist`](WriteRepository::persist) in scope:
//!
//! ```rust
//! use storage::WriteRepository;
//!
//! async fn write_through_a_writer<W: WriteRepository>(writer: &W, record: W::Record) {
//!     let write = writer.persist(record).await;
//!     write.expect("Given a writer that accepts the record, the write should always succeed");
//! }
//! ```

use async_trait::async_trait;

use crate::error::WriteError;

/// Persists a record to the store.
///
/// # Associated Types
///
/// Each implementor chooses the concrete type filling this slot, which is what keeps the trait
/// decoupled from any specific database:
///
/// - `Record`: The record this repository persists.
#[async_trait]
pub trait WriteRepository: Send + Sync {
    /// The record this repository persists.
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
