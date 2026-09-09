//! # Fake Write Repository
//!
//! Provides [`FakeWriteRepository`], an in-memory [`WriteRepository`] test double that records
//! every persisted record. It proves the [`WriteRepository`] trait end-to-end and lets consumer
//! tests run with no database: [`FakeWriteRepository::persist`] records the record and returns
//! `Ok`, and [`FakeWriteRepository::persisted`] exposes what it recorded for assertions.

use std::sync::Mutex;

use async_trait::async_trait;

use crate::error::WriteError;
use crate::repository::WriteRepository;

/// An in-memory [`WriteRepository`] test double that records every persisted record.
///
/// Generic over the record type, so it fits any [`WriteRepository`] binding. It models no
/// transaction. It records the record and returns `Ok`, which keeps consumer tests pinned to the
/// trait's contract.
#[derive(Debug)]
pub struct FakeWriteRepository<Rec> {
    persisted: Mutex<Vec<Rec>>,
}

impl<Rec> Default for FakeWriteRepository<Rec> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Rec> FakeWriteRepository<Rec> {
    /// Creates an empty [`FakeWriteRepository`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            persisted: Mutex::new(Vec::new()),
        }
    }

    /// Returns a clone of every record persisted so far, in insertion order.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex has been poisoned by a previous panic while the lock was held.
    #[must_use]
    pub fn persisted(&self) -> Vec<Rec>
    where
        Rec: Clone,
    {
        self.persisted
            .lock()
            .expect("Fake write repository mutex should never be poisoned in a test")
            .clone()
    }
}

#[async_trait]
impl<Rec> WriteRepository for FakeWriteRepository<Rec>
where
    Rec: Send,
{
    type Record = Rec;

    async fn persist(&self, record: Self::Record) -> Result<(), WriteError> {
        self.persisted
            .lock()
            .expect("Fake write repository mutex should never be poisoned in a test")
            .push(record);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_fake_write_repository() {
        implements_auto_traits::<FakeWriteRepository<String>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_using_fake_write_repository() {
        implements_send::<FakeWriteRepository<String>>();
    }

    #[test]
    const fn should_implement_sync_when_using_fake_write_repository() {
        implements_sync::<FakeWriteRepository<String>>();
    }

    #[test]
    const fn should_be_thread_safe_when_using_fake_write_repository() {
        implements_send::<FakeWriteRepository<String>>();
        implements_sync::<FakeWriteRepository<String>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_being_sized_when_using_fake_write_repository() {
        implements_sized::<FakeWriteRepository<String>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_fake_write_repository() {
        implements_debug::<FakeWriteRepository<String>>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_be_able_to_rely_on_default_implementation_when_using_fake_write_repository() {
        implements_default::<FakeWriteRepository<String>>();
    }

    #[tokio::test]
    async fn should_record_the_record_when_persist_is_called() {
        let repository = FakeWriteRepository::new();
        let record = "CIK0001067983".to_string();
        let expected_result = vec![record.clone()];

        repository
            .persist(record)
            .await
            .expect("Given a fake write repository that records every record, the persist should always succeed");

        let result = repository.persisted();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_records_in_insertion_order_when_persist_is_called_repeatedly() {
        let repository = FakeWriteRepository::new();
        let expected_result = vec!["first".to_string(), "second".to_string()];

        repository.persist("first".to_string()).await.expect(
            "Given a fake write repository, persisting the first record should always succeed",
        );
        repository.persist("second".to_string()).await.expect(
            "Given a fake write repository, persisting the second record should always succeed",
        );

        let result = repository.persisted();

        assert_eq!(result, expected_result);
    }
}
