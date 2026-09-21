//! # Fake Read Write Repository
//!
//! Provides [`FakeReadWriteRepository`], an in-memory test double that implements both
//! [`ReadRepository`] and [`WriteRepository`]. [`FakeReadWriteRepository::persist`] appends a
//! record and [`FakeReadWriteRepository::get`] returns the record at a position, so a test can
//! write a record and read the same record back. The blanket implementation gives it
//! [`ReadWriteRepository`](crate::repository::ReadWriteRepository).

use std::sync::Mutex;

use async_trait::async_trait;

use crate::error::{ReadError, WriteError};
use crate::repository::{ReadRepository, WriteRepository};

/// An in-memory test double that implements both [`ReadRepository`] and [`WriteRepository`].
///
/// Generic over the record type. The key is the position the record was persisted at, which lets a
/// test write a record with [`FakeReadWriteRepository::persist`] and read it back with
/// [`FakeReadWriteRepository::get`].
#[derive(Debug)]
pub struct FakeReadWriteRepository<Rec> {
    records: Mutex<Vec<Rec>>,
}

impl<Rec> Default for FakeReadWriteRepository<Rec> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Rec> FakeReadWriteRepository<Rec> {
    /// Creates an empty [`FakeReadWriteRepository`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            records: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl<Rec> WriteRepository for FakeReadWriteRepository<Rec>
where
    Rec: Send,
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
impl<Rec> ReadRepository for FakeReadWriteRepository<Rec>
where
    Rec: Clone + Send,
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
    const fn should_implement_auto_traits_for_fake_read_write_repository() {
        implements_auto_traits::<FakeReadWriteRepository<String>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_fake_read_write_repository() {
        implements_send::<FakeReadWriteRepository<String>>();
    }

    #[test]
    const fn should_implement_sync_for_fake_read_write_repository() {
        implements_sync::<FakeReadWriteRepository<String>>();
    }

    #[test]
    const fn should_be_thread_safe_for_fake_read_write_repository() {
        implements_send::<FakeReadWriteRepository<String>>();
        implements_sync::<FakeReadWriteRepository<String>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_being_sized_for_fake_read_write_repository() {
        implements_sized::<FakeReadWriteRepository<String>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_for_fake_read_write_repository() {
        implements_debug::<FakeReadWriteRepository<String>>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_be_able_to_rely_on_default_for_fake_read_write_repository() {
        implements_default::<FakeReadWriteRepository<String>>();
    }

    #[tokio::test]
    async fn should_read_back_the_persisted_record_when_the_record_was_persisted() {
        let store = FakeReadWriteRepository::new();
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
        let store: FakeReadWriteRepository<String> = FakeReadWriteRepository::new();
        let expected_result = None;

        let result = store
            .get(0)
            .await
            .expect("Given an empty fake store, the read should always succeed");

        assert_eq!(result, expected_result);
    }
}
