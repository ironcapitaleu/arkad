//! # Fake Read Repository
//!
//! Provides [`FakeReadRepository`], an in-memory [`ReadRepository`] test double seeded with
//! key-to-record entries. It proves the [`ReadRepository`] trait end-to-end and lets consumer
//! tests run with no database: [`FakeReadRepository::get`] serves the seeded record, and
//! [`FakeReadRepository::failing`] builds a double that fails every read.

use async_trait::async_trait;

use crate::error::ReadError;
use crate::repository::ReadRepository;

/// An in-memory [`ReadRepository`] test double seeded with key-to-record entries.
///
/// Generic over the key type and the record type, so it fits any [`ReadRepository`] binding. It
/// models no query beyond the key lookup the trait declares.
#[derive(Debug)]
pub struct FakeReadRepository<Key, Rec> {
    entries: Vec<(Key, Rec)>,
    failure: Option<ReadError>,
}

impl<Key, Rec> Default for FakeReadRepository<Key, Rec> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Key, Rec> FakeReadRepository<Key, Rec> {
    /// Creates a [`FakeReadRepository`] that holds no entry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            failure: None,
        }
    }

    /// Creates a [`FakeReadRepository`] seeded with the given key-to-record entries.
    #[must_use]
    pub const fn seeded(entries: Vec<(Key, Rec)>) -> Self {
        Self {
            entries,
            failure: None,
        }
    }

    /// Creates a [`FakeReadRepository`] that returns the given [`ReadError`] for every read.
    #[must_use]
    pub const fn failing(error: ReadError) -> Self {
        Self {
            entries: Vec::new(),
            failure: Some(error),
        }
    }
}

#[async_trait]
impl<Key, Rec> ReadRepository for FakeReadRepository<Key, Rec>
where
    Key: PartialEq + Send + Sync,
    Rec: Clone + Send + Sync,
{
    type Record = Rec;
    type Key = Key;

    async fn get(&self, key: Self::Key) -> Result<Option<Self::Record>, ReadError> {
        if let Some(failure) = self.failure.clone() {
            return Err(failure);
        }

        Ok(self
            .entries
            .iter()
            .find(|(seeded_key, _)| *seeded_key == key)
            .map(|(_, record)| record.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::error::BackendError;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_auto_traits_when_using_fake_read_repository() {
        implements_auto_traits::<FakeReadRepository<String, String>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_using_fake_read_repository() {
        implements_send::<FakeReadRepository<String, String>>();
    }

    #[test]
    const fn should_implement_sync_when_using_fake_read_repository() {
        implements_sync::<FakeReadRepository<String, String>>();
    }

    #[test]
    const fn should_be_thread_safe_when_using_fake_read_repository() {
        implements_send::<FakeReadRepository<String, String>>();
        implements_sync::<FakeReadRepository<String, String>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_being_sized_when_using_fake_read_repository() {
        implements_sized::<FakeReadRepository<String, String>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_fake_read_repository() {
        implements_debug::<FakeReadRepository<String, String>>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_be_able_to_rely_on_default_implementation_when_using_fake_read_repository() {
        implements_default::<FakeReadRepository<String, String>>();
    }

    #[tokio::test]
    async fn should_return_the_seeded_record_when_the_key_is_seeded() {
        let repository =
            FakeReadRepository::seeded(vec![("0000320193".to_string(), "Apple".to_string())]);
        let expected_result = Some("Apple".to_string());

        let result = repository
            .get("0000320193".to_string())
            .await
            .expect("Given a key the fake was seeded with, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_the_key_is_not_seeded() {
        let repository =
            FakeReadRepository::seeded(vec![("0000320193".to_string(), "Apple".to_string())]);
        let expected_result = None;

        let result = repository
            .get("0001067983".to_string())
            .await
            .expect("Given a fake that was not seeded to fail, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_seeded_error_when_the_repository_is_seeded_to_fail() {
        let error = ReadError::Backend(BackendError::unavailable("timeout"));
        let repository: FakeReadRepository<String, String> =
            FakeReadRepository::failing(error.clone());
        let expected_result = error;

        let result = repository.get("0000320193".to_string()).await.expect_err(
            "Given a fake seeded to fail, the read should always return the seeded error",
        );

        assert_eq!(result, expected_result);
    }
}
