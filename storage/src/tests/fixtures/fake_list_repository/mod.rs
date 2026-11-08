//! # Fake List Repository
//!
//! Provides [`FakeListRepository`], an in-memory store that lists and reads its seeded entries,
//! and [`collect`], which gathers a listing into a `Vec`.

use std::future::poll_fn;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_trait::async_trait;
use futures_core::stream::{BoxStream, Stream};

use crate::error::ReadError;
use crate::traits::repository::{ListRepository, ReadRepository};

/// An in-memory store that lists and reads its seeded entries.
///
/// Implements [`ListRepository`] and [`ReadRepository`] with one key type, so it is a
/// [`ReadListRepository`](crate::ReadListRepository). Each seeded item is a key with an entry or
/// with an error. An error item stands for a document that the store cannot read, for example
/// because its metadata does not parse. Both listings yield the items sorted by the bytes of the
/// key text, each key once. [`list_keys`](ListRepository::list_keys) yields every key, and
/// [`list`](ListRepository::list) and [`get`](ReadRepository::get) yield the error of an error
/// item. The filter `()` selects every item.
#[derive(Debug)]
pub struct FakeListRepository<Key, Ent> {
    items: Vec<(Key, Result<Ent, ReadError>)>,
}

impl<Key: AsRef<str>, Ent> FakeListRepository<Key, Ent> {
    /// Creates a [`FakeListRepository`] seeded with the given key-to-entry pairs.
    #[must_use]
    pub fn seeded(entries: Vec<(Key, Ent)>) -> Self {
        let items = entries
            .into_iter()
            .map(|(key, entry)| (key, Ok(entry)))
            .collect();
        Self::with_items(items)
    }

    /// Creates a [`FakeListRepository`] seeded with the given items, each an entry or an error.
    ///
    /// Keeps the first item of each key, as [`get`](ReadRepository::get) does.
    #[must_use]
    pub fn with_items(mut items: Vec<(Key, Result<Ent, ReadError>)>) -> Self {
        items.sort_by(|(left, _), (right, _)| {
            left.as_ref().as_bytes().cmp(right.as_ref().as_bytes())
        });
        items.dedup_by(|(right, _), (left, _)| left.as_ref() == right.as_ref());
        Self { items }
    }
}

impl<Key, Ent> ListRepository for FakeListRepository<Key, Ent>
where
    Key: Clone + Send + Sync,
    Ent: Clone + Send + Sync,
{
    type Key = Key;
    type Entry = Ent;
    type Filter = ();

    fn list_keys(&self, _filter: ()) -> BoxStream<'_, Result<Key, ReadError>> {
        Box::pin(IteratorStream(
            self.items.iter().map(|(key, _)| Ok(key.clone())),
        ))
    }

    fn list(&self, _filter: ()) -> BoxStream<'_, Result<Ent, ReadError>> {
        Box::pin(IteratorStream(
            self.items.iter().map(|(_, item)| item.clone()),
        ))
    }
}

#[async_trait]
impl<Key, Ent> ReadRepository for FakeListRepository<Key, Ent>
where
    Key: PartialEq + Send + Sync,
    Ent: Clone + Send + Sync,
{
    type Record = Ent;
    type Key = Key;

    async fn get(&self, key: Key) -> Result<Option<Ent>, ReadError> {
        self.items
            .iter()
            .find(|(item_key, _)| *item_key == key)
            .map(|(_, item)| item.clone())
            .transpose()
    }
}

/// Collects every item of a stream into a `Vec`, in stream order.
pub async fn collect<T>(mut stream: BoxStream<'_, T>) -> Vec<T> {
    let mut items = Vec::new();
    while let Some(item) = poll_fn(|context| stream.as_mut().poll_next(context)).await {
        items.push(item);
    }
    items
}

/// A [`Stream`] that yields the items of an iterator, each one ready at once.
struct IteratorStream<I>(I);

impl<I: Iterator + Unpin> Stream for IteratorStream<I> {
    type Item = I::Item;

    fn poll_next(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Option<I::Item>> {
        Poll::Ready(self.0.next())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::error::BackendError;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_fake_list_repository() {
        implements_auto_traits::<FakeListRepository<String, u64>>();
    }

    #[tokio::test]
    async fn should_list_the_keys_in_byte_order_when_seeded_out_of_order() {
        let entries = vec![
            ("a/x".to_string(), 1),
            ("b".to_string(), 2),
            ("a-b".to_string(), 3),
        ];
        let repository = FakeListRepository::seeded(entries);

        let expected_result = vec![
            Ok("a-b".to_string()),
            Ok("a/x".to_string()),
            Ok("b".to_string()),
        ];

        let result = collect(repository.list_keys(())).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_list_the_entries_in_byte_order_when_seeded_out_of_order() {
        let entries = vec![("a/x".to_string(), 1), ("a-b".to_string(), 3)];
        let repository = FakeListRepository::seeded(entries);

        let expected_result = vec![Ok(3), Ok(1)];

        let result = collect(repository.list(())).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_continue_the_stream_when_an_item_is_an_error() {
        let error = ReadError::Backend(BackendError::unreachable_storage("timeout"));
        let items = vec![
            ("a".to_string(), Ok(1)),
            ("b".to_string(), Err(error.clone())),
            ("c".to_string(), Ok(3)),
        ];
        let repository = FakeListRepository::with_items(items);

        let expected_result = vec![Ok(1), Err(error), Ok(3)];

        let result = collect(repository.list(())).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_first_entry_when_a_key_is_seeded_twice() {
        let entries = vec![("a".to_string(), 1), ("a".to_string(), 2)];
        let repository = FakeListRepository::seeded(entries);

        let expected_result = vec![Ok(1)];

        let result = collect(repository.list(())).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_list_the_key_when_the_item_is_an_error() {
        let error = ReadError::Backend(BackendError::unreachable_storage("timeout"));
        let items = vec![("a".to_string(), Err(error)), ("b".to_string(), Ok(2))];
        let repository = FakeListRepository::with_items(items);

        let expected_result = vec![Ok("a".to_string()), Ok("b".to_string())];

        let result = collect(repository.list_keys(())).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_entry_when_reading_a_seeded_key() {
        let repository = FakeListRepository::seeded(vec![("a".to_string(), 1_u64)]);
        let key = "a".to_string();

        let expected_result = Some(1);

        let result = repository
            .get(key)
            .await
            .expect("Given a key seeded with an entry, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_reading_a_key_that_was_not_seeded() {
        let repository = FakeListRepository::seeded(vec![("a".to_string(), 1_u64)]);
        let key = "b".to_string();

        let expected_result = Ok(None);

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_error_when_reading_a_key_seeded_with_an_error() {
        let error = ReadError::Backend(BackendError::unreachable_storage("timeout"));
        let repository = FakeListRepository::<String, u64>::with_items(vec![(
            "a".to_string(),
            Err(error.clone()),
        )]);
        let key = "a".to_string();

        let expected_result = Err(error);

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }
}
