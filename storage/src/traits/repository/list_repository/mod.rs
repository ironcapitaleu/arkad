//! # List Repository
//!
//! Provides [`ListRepository`], the trait through which a caller lists the documents a store holds.

use futures_core::stream::BoxStream;

use crate::error::ReadError;

/// Lists the documents a store holds, as metadata, never as bytes.
///
/// A caller that must know what a store holds, without knowing every key in advance, depends on
/// this trait. It is separate from [`ReadRepository`](crate::ReadRepository), so a store that only
/// reads by key implements no listing, and a caller bound to `ListRepository` alone has no `get`.
/// A store that reads and lists with one key type is a
/// [`ReadListRepository`](crate::ReadListRepository).
///
/// An implementor must keep this contract:
///
/// - **Order**: the stream yields the documents sorted by the UTF-8 bytes of the key text, with `/`
///   between the components. So the key `a-b` comes before `a/x`, because `-` is byte 0x2D and `/`
///   is byte 0x2F.
/// - **Once**: a key appears at most once in one listing.
/// - **Error items**: an error about one document is one `Err` item, and the stream continues
///   after it. An error that stops the whole listing is the last item.
///
/// A trait object sets all three associated types, as in
/// `Box<dyn ListRepository<Key = K, Entry = E, Filter = F>>`. A trait object that leaves one out
/// fails with E0191:
///
/// ```compile_fail,E0191
/// use storage::ListRepository;
///
/// fn takes_a_lister(_lister: Box<dyn ListRepository<Key = String, Entry = u64>>) {}
/// ```
///
/// # Associated Types
///
/// - `Key`: The key of one listed document.
/// - `Entry`: What one listed document reports: its key and its metadata.
/// - `Filter`: What a listing selects, for example a key prefix.
pub trait ListRepository: Send + Sync {
    /// The key of one listed document.
    type Key: Send;

    /// What one listed document reports: its key and its metadata.
    type Entry: Send;

    /// What a listing selects, for example a key prefix.
    type Filter: Send;

    /// Lists the key of each stored document that the filter selects.
    ///
    /// Reads no metadata. So a store that keeps the document type in the metadata yields the keys
    /// of every type, and the caller keeps the keys it needs.
    ///
    /// # Errors
    ///
    /// Yields a [`ReadError`] item if the store cannot list a document.
    fn list_keys(&self, filter: Self::Filter) -> BoxStream<'_, Result<Self::Key, ReadError>>;

    /// Lists each stored document that the filter selects, with its metadata.
    ///
    /// Reads the metadata, never the bytes of a document.
    ///
    /// # Errors
    ///
    /// Yields a [`ReadError`] item if the store cannot list a document or read its metadata.
    fn list(&self, filter: Self::Filter) -> BoxStream<'_, Result<Self::Entry, ReadError>>;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::fake_list_repository::{FakeListRepository, collect};

    const fn implements_list_repository<T: ListRepository>() {}

    #[test]
    const fn should_implement_list_repository_for_fake_list_repository() {
        implements_list_repository::<FakeListRepository<String, u64>>();
    }

    #[tokio::test]
    async fn should_list_the_keys_when_listing_through_a_trait_object() {
        let repository: Box<dyn ListRepository<Key = String, Entry = u64, Filter = ()>> = Box::new(
            FakeListRepository::seeded(vec![("sec/notes".to_string(), 7)]),
        );

        let expected_result = vec![Ok("sec/notes".to_string())];

        let result = collect(repository.list_keys(())).await;

        assert_eq!(result, expected_result);
    }
}
