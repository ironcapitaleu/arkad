//! # Fake Repository
//!
//! Provides [`FakeRepository`], an in-memory [`Repository`] test double that records every
//! persisted record. It proves the [`Repository`] trait end-to-end and lets consumer tests run with
//! zero database: [`FakeRepository::persist`] simply records the record and returns `Ok`, and
//! [`FakeRepository::persisted`] exposes what was recorded for assertions.

use std::sync::Mutex;

use async_trait::async_trait;

use crate::error::WriteError;
use crate::repository::Repository;

/// An in-memory [`Repository`] test double that records every persisted record.
///
/// Generic over the record type so it fits any `Repository` binding. It neither composes tier
/// writes nor models a transaction — it records the record and returns `Ok`, keeping consumer tests
/// pinned to the trait's contract.
#[derive(Debug)]
pub struct FakeRepository<Rec> {
    persisted: Mutex<Vec<Rec>>,
}

impl<Rec> Default for FakeRepository<Rec> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Rec> FakeRepository<Rec> {
    /// Creates a new, empty [`FakeRepository`].
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
            .expect("Fake repository mutex should never be poisoned in a single-threaded test")
            .clone()
    }
}

#[async_trait]
impl<Rec> Repository for FakeRepository<Rec>
where
    Rec: Send,
{
    type Record = Rec;

    async fn persist(&self, record: Self::Record) -> Result<(), WriteError> {
        self.persisted
            .lock()
            .expect("Fake repository mutex should never be poisoned in a single-threaded test")
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
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_fake_repository() {
        implements_auto_traits::<FakeRepository<String>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_using_fake_repository() {
        implements_send::<FakeRepository<String>>();
    }

    #[test]
    const fn should_implement_sync_when_using_fake_repository() {
        implements_sync::<FakeRepository<String>>();
    }

    #[test]
    const fn should_be_thread_safe_when_using_fake_repository() {
        implements_send::<FakeRepository<String>>();
        implements_sync::<FakeRepository<String>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_being_sized_when_using_fake_repository() {
        implements_sized::<FakeRepository<String>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_fake_repository() {
        implements_debug::<FakeRepository<String>>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_be_able_to_rely_on_default_implementation_when_using_fake_repository() {
        implements_default::<FakeRepository<String>>();
    }

    #[tokio::test]
    async fn should_record_the_record_when_persist_is_called() {
        let repository = FakeRepository::new();
        let record = "CIK0001067983".to_string();
        let expected_result = vec![record.clone()];

        repository
            .persist(record)
            .await
            .expect("Persisting into the fake repository should always succeed");

        let result = repository.persisted();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_records_in_insertion_order_when_persist_is_called_repeatedly() {
        let repository = FakeRepository::new();
        let expected_result = vec!["first".to_string(), "second".to_string()];

        repository
            .persist("first".to_string())
            .await
            .expect("Persisting the first record into the fake repository should always succeed");
        repository
            .persist("second".to_string())
            .await
            .expect("Persisting the second record into the fake repository should always succeed");

        let result = repository.persisted();

        assert_eq!(result, expected_result);
    }
}
