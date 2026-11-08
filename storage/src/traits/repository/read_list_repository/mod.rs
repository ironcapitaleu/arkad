//! # Read List Repository
//!
//! Provides [`ReadListRepository`], the one name for a store that reads and lists with one key
//! type.

use crate::traits::repository::list_repository::ListRepository;
use crate::traits::repository::read_repository::ReadRepository;

/// Names a store that reads and lists with one key type.
///
/// Adds no method of its own. A blanket implementation grants it to every type that implements
/// [`ReadRepository`] and [`ListRepository`] with the same `Key`. So a caller bound to
/// `R: ReadListRepository` passes each key from [`list_keys`](ListRepository::list_keys) to
/// [`get`](ReadRepository::get) with no conversion.
///
/// # Naming the Key Type
///
/// Both parent traits declare `Key`, so a bare `R::Key` is ambiguous and fails with E0221. A
/// caller writes `<R as ReadRepository>::Key`. The bound makes the two keys one type, so
/// `<R as ListRepository>::Key` names the same type.
///
/// ```compile_fail,E0221
/// use storage::ReadListRepository;
///
/// fn takes_a_key<R: ReadListRepository>(_key: R::Key) {}
/// ```
///
/// For the same reason, `ReadListRepository` has no trait object. A caller that needs one uses
/// `dyn ListRepository<…>` or `dyn ReadRepository<…>`.
///
/// A store whose two keys differ gets no `ReadListRepository`. The bound fails with E0271:
///
/// ```compile_fail,E0271
/// use futures_core::stream::BoxStream;
/// use storage::{ListRepository, ReadError, ReadListRepository, ReadRepository};
///
/// struct TwoKeyStore;
///
/// #[async_trait::async_trait]
/// impl ReadRepository for TwoKeyStore {
///     type Record = String;
///     type Key = String;
///
///     async fn get(&self, _key: String) -> Result<Option<String>, ReadError> {
///         Ok(None)
///     }
/// }
///
/// impl ListRepository for TwoKeyStore {
///     type Key = u64;
///     type Entry = u64;
///     type Filter = ();
///
///     fn list_keys(&self, _filter: ()) -> BoxStream<'_, Result<u64, ReadError>> {
///         unimplemented!()
///     }
///
///     fn list(&self, _filter: ()) -> BoxStream<'_, Result<u64, ReadError>> {
///         unimplemented!()
///     }
/// }
///
/// const fn implements_read_list_repository<T: ReadListRepository>() {}
/// implements_read_list_repository::<TwoKeyStore>();
/// ```
///
/// # Required Traits
///
/// - [`ReadRepository`]: gives the store its [`get`](ReadRepository::get) method.
/// - [`ListRepository`]: gives the store its [`list_keys`](ListRepository::list_keys) and
///   [`list`](ListRepository::list) methods, with the `Key` of [`ReadRepository`].
pub trait ReadListRepository:
    ReadRepository + ListRepository<Key = <Self as ReadRepository>::Key>
{
}

impl<T> ReadListRepository for T where
    T: ReadRepository + ListRepository<Key = <T as ReadRepository>::Key>
{
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::fake_list_repository::{FakeListRepository, collect};

    const fn implements_read_list_repository<T: ReadListRepository>() {}

    #[test]
    const fn should_implement_read_list_repository_for_fake_list_repository() {
        implements_read_list_repository::<FakeListRepository<String, u64>>();
    }

    async fn read_the_first_listed_record<R: ReadListRepository>(
        repository: &R,
        filter: <R as ListRepository>::Filter,
    ) -> Option<<R as ReadRepository>::Record> {
        let keys = collect(repository.list_keys(filter)).await;
        let first_key = keys
            .into_iter()
            .next()
            .expect("Given a fake seeded with one entry, the listing should always yield a key")
            .expect("Given a fake whose items are all entries, the listed key should always be Ok");
        repository
            .get(first_key)
            .await
            .expect("Given a key the fake listed, the read should always succeed")
    }

    #[tokio::test]
    async fn should_read_the_record_when_the_key_comes_from_list_keys() {
        let repository = FakeListRepository::seeded(vec![("sec/notes".to_string(), 7_u64)]);

        let expected_result = Some(7);

        let result = read_the_first_listed_record(&repository, ()).await;

        assert_eq!(result, expected_result);
    }
}
