//! # Read Repository
//!
//! Provides [`ReadRepository`], the trait through which a record is read from the store by key.
//!
//! ## Usage
//!
//! A caller bound to [`ReadRepository`] has [`get`](ReadRepository::get) in scope:
//!
//! ```rust
//! use storage::ReadRepository;
//!
//! async fn read_through_a_reader<R: ReadRepository>(
//!     reader: &R,
//!     key: R::Key,
//! ) -> Option<R::Record> {
//!     let read = reader.get(key).await;
//!     read.expect("Given a reader that holds the key, the read should always succeed")
//! }
//! ```

use async_trait::async_trait;

use crate::error::ReadError;

/// Reads a record from the store by key.
///
/// # Associated Types
///
/// Each implementor chooses the concrete types filling these slots, which is what keeps the trait
/// decoupled from any specific database:
///
/// - `Record`: The record this repository returns.
/// - `Key`: The primary key identifying one record in the store.
#[async_trait]
pub trait ReadRepository: Send + Sync {
    /// The record this repository returns.
    ///
    /// Bounded by [`Send`] because [`ReadRepository::get`] returns it across an `async` boundary.
    type Record: Send;

    /// The primary key identifying one record in the store.
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
    /// Returns a [`ReadError`] if the read itself fails.
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
    async fn should_return_the_record_when_reading_through_a_trait_object() {
        let repository: Box<dyn ReadRepository<Record = String, Key = String>> = Box::new(
            FakeReadRepository::initialized(vec![("0000320193".to_string(), "Apple".to_string())]),
        );
        let expected_result = Some("Apple".to_string());

        let result = repository
            .get("0000320193".to_string())
            .await
            .expect("Given a key the fake was initialized with, the read should always succeed");

        assert_eq!(result, expected_result);
    }
}
