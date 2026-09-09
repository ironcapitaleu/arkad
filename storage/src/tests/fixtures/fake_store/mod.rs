//! # Fake Store
//!
//! Provides [`FakeStore`], an in-memory test double that implements both persistence ports. It
//! round-trips a record: [`FakeStore::persist`] appends the record, and [`FakeStore::get`] returns
//! the record at a position. The blanket implementation gives it
//! [`ReadWriteRepository`](crate::repository::ReadWriteRepository).

use std::sync::Mutex;

use async_trait::async_trait;

use crate::error::{ReadError, WriteError};
use crate::repository::{ReadRepository, WriteRepository};

/// An in-memory test double that implements both persistence ports.
///
/// Generic over the record type. The key is the position the record was persisted at, which lets a
/// test write a record and read the same record back through the two ports.
#[derive(Debug)]
pub struct FakeStore<Rec> {
    records: Mutex<Vec<Rec>>,
}

impl<Rec> Default for FakeStore<Rec> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Rec> FakeStore<Rec> {
    /// Creates an empty [`FakeStore`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            records: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl<Rec> WriteRepository for FakeStore<Rec>
where
    Rec: Send + Sync,
{
    type Record = Rec;

    async fn persist(&self, record: Self::Record) -> Result<(), WriteError> {
        self.records
            .lock()
            .expect("Fake store mutex should never be poisoned in a test")
            .push(record);
        Ok(())
    }
}

#[async_trait]
impl<Rec> ReadRepository for FakeStore<Rec>
where
    Rec: Clone + Send + Sync,
{
    type Record = Rec;
    type Key = usize;

    async fn get(&self, key: Self::Key) -> Result<Option<Self::Record>, ReadError> {
        Ok(self
            .records
            .lock()
            .expect("Fake store mutex should never be poisoned in a test")
            .get(key)
            .cloned())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_auto_trait_implementation_when_using_fake_store() {
        implements_auto_traits::<FakeStore<String>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_using_fake_store() {
        implements_send::<FakeStore<String>>();
    }

    #[test]
    const fn should_implement_sync_when_using_fake_store() {
        implements_sync::<FakeStore<String>>();
    }

    #[test]
    const fn should_be_thread_safe_when_using_fake_store() {
        implements_send::<FakeStore<String>>();
        implements_sync::<FakeStore<String>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_being_sized_when_using_fake_store() {
        implements_sized::<FakeStore<String>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_fake_store() {
        implements_debug::<FakeStore<String>>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_be_able_to_rely_on_default_implementation_when_using_fake_store() {
        implements_default::<FakeStore<String>>();
    }

    #[tokio::test]
    async fn should_read_back_the_persisted_record_when_the_record_was_persisted() {
        let store = FakeStore::new();
        let expected_result = Some("CIK0001067983".to_string());

        store.persist("CIK0001067983".to_string()).await.expect(
            "Given a fake store that records every record, the persist should always succeed",
        );
        let result = store
            .get(0)
            .await
            .expect("Given a record persisted at this position, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_the_position_holds_no_record() {
        let store: FakeStore<String> = FakeStore::new();
        let expected_result = None;

        let result = store
            .get(0)
            .await
            .expect("Given an empty fake store, the read should always succeed");

        assert_eq!(result, expected_result);
    }
}
