//! # Read Repository
//!
//! Provides [`ReadRepository`], the trait through which a record is read from the store by key.
//!
//! The trait carries no write method. A caller bound to [`ReadRepository`] has no `persist` in
//! scope, so a write from a read-only caller does not compile:
//!
//! ```compile_fail
//! use storage::ReadRepository;
//!
//! async fn write_through_a_reader<R: ReadRepository>(reader: &R, record: R::Record) {
//!     reader.persist(record).await.expect("A reader has no persist method");
//! }
//! ```

use async_trait::async_trait;

use crate::error::ReadError;

/// Reads a record from the store by key.
///
/// Injected as a concrete type — production wires a real backend, tests wire a fake — so callers
/// depend on this trait rather than on a database. Each implementor binds
/// [`ReadRepository::Record`] to its own read-unit and [`ReadRepository::Key`] to the value that
/// identifies one record.
///
/// # Associated Types
///
/// - [`ReadRepository::Record`]: the read-unit returned by [`ReadRepository::get`].
/// - [`ReadRepository::Key`]: the lookup value that identifies one record.
#[async_trait]
pub trait ReadRepository: Send + Sync {
    /// The unit this repository returns — one record per [`ReadRepository::get`] call.
    /// Implementations bind it to their concrete read-unit (for example, a filing record).
    ///
    /// Bounded by [`Send`] because [`ReadRepository::get`] returns it across an `async` boundary.
    type Record: Send;

    /// The lookup value that identifies one record. Implementations bind it to their concrete key
    /// (for example, an accession number).
    ///
    /// Bounded by [`Send`] because [`ReadRepository::get`] moves it across an `async` boundary.
    type Key: Send;

    /// Reads the record a key identifies.
    ///
    /// Returns `Ok(None)` when the key resolves to no record. An absent record is a read result,
    /// not a failure.
    ///
    /// # Errors
    ///
    /// Returns a [`ReadError`] if the read itself fails:
    /// - [`ReadError::MissingRecord`] — the caller required a record the store does not hold.
    /// - [`ReadError::Backend`] — the read failed at the backend level.
    async fn get(&self, key: Self::Key) -> Result<Option<Self::Record>, ReadError>;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::fake_read_repository::FakeReadRepository;

    const fn implements_read_repository<T: ReadRepository>() {}

    #[test]
    const fn should_implement_read_repository_when_using_fake_read_repository() {
        implements_read_repository::<FakeReadRepository<String, String>>();
    }

    #[tokio::test]
    async fn should_dispatch_get_through_a_trait_object_when_the_repository_is_boxed() {
        let repository: Box<dyn ReadRepository<Record = String, Key = String>> = Box::new(
            FakeReadRepository::seeded(vec![("0000320193".to_string(), "Apple".to_string())]),
        );
        let expected_result = Some("Apple".to_string());

        let result = repository
            .get("0000320193".to_string())
            .await
            .expect("A seeded fake read repository serves a seeded key without failing");

        assert_eq!(result, expected_result);
    }
}
