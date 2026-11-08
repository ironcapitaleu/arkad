//! # List Repository
//!
//! Implements [`ListRepository`] for [`FilesystemRepository`].

use std::collections::VecDeque;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use chrono::{DateTime, Utc};
use futures_core::future::BoxFuture;
use futures_core::stream::{BoxStream, Stream};
use tokio::fs;

use crate::error::{BackendError, ReadError};
use crate::implementations::repository::filesystem::FilesystemRepository;
use crate::implementations::repository::filesystem::error::{
    FailedFilesystemOperation, FilesystemOperation,
};
use crate::implementations::repository::filesystem::implementations::key_walk::WalkedKey;
use crate::implementations::repository::filesystem::implementations::read_repository::{
    StoredDocumentType, check_document_type, parse_metadata, read_metadata_file,
};
use crate::implementations::repository::filesystem::shared::document_key::DocumentKey;
use crate::implementations::repository::filesystem::shared::key_prefix::KeyPrefix;
use crate::implementations::repository::filesystem::shared::listed_document::ListedDocument;
use crate::implementations::repository::filesystem::shared::stored_document::StoredDocument;
use crate::implementations::repository::filesystem::traits::access::CanRead;
use crate::implementations::repository::filesystem::traits::document::Document;
use crate::traits::repository::ListRepository;

/// The most metadata files that [`list`](ListRepository::list) reads at once.
///
/// On a mounted bucket, each read is one request. Reads in flight hide the latency of each
/// request, and the bound keeps the memory of a listing flat.
const METADATA_READS_IN_FLIGHT: usize = 16;

impl<A: CanRead, D: Document> ListRepository for FilesystemRepository<A, D>
where
    D::Context: Send,
{
    type Key = DocumentKey;
    type Entry = ListedDocument<D::Context>;
    type Filter = KeyPrefix;

    /// Lists the key of each document under the prefix that has a document file and a metadata
    /// file.
    ///
    /// Reads no file, so it yields the keys of every document type under the prefix. A key comes
    /// out sorted by the bytes of the key text, each key once. The walk skips a hidden `.tmp` file
    /// of a write, a document without metadata, and metadata without its document.
    ///
    /// # Errors
    ///
    /// Yields a [`ReadError::Backend`] item that holds the [`FailedFilesystemOperation`] message
    /// if:
    /// - the root does not exist or is not a directory. The stream ends after this item.
    /// - the filesystem cannot list the directory the prefix names. The stream ends after this
    ///   item.
    /// - the filesystem cannot list a directory below it. The stream continues with the next name
    ///   of its parent.
    fn list_keys(&self, filter: KeyPrefix) -> BoxStream<'_, Result<DocumentKey, ReadError>> {
        Box::pin(KeysWithDocument(self.walk_keys(&filter)))
    }

    /// Lists each document of type `D` under the prefix, with its metadata.
    ///
    /// Reads the metadata file of each key, and inspects the document file for its size. It
    /// never reads the bytes of a document. It yields metadata without its document, as
    /// [`StoredDocument::Missing`]. It skips metadata of another document type, because the
    /// repository of that type lists it. It reads a fixed number of metadata files at once, and
    /// yields the entries in the order of [`list_keys`](ListRepository::list_keys).
    ///
    /// # Errors
    ///
    /// Yields a [`ReadError::Backend`] item that holds the [`FailedFilesystemOperation`] message
    /// for each error of [`list_keys`](ListRepository::list_keys), and for a document whose
    /// metadata:
    /// - holds the document type of `D` with another document type version.
    /// - holds no valid metadata.
    /// - the filesystem cannot read or inspect.
    ///
    /// The stream continues after an error item about one document.
    fn list(&self, filter: KeyPrefix) -> BoxStream<'_, Result<Self::Entry, ReadError>> {
        Box::pin(OrderedReads::new(self.walk_keys(&filter), move |walked| {
            Box::pin(read_entry(self, walked))
        }))
    }
}

/// The keys of a walk that have a document file beside their metadata file.
struct KeysWithDocument(BoxStream<'static, Result<WalkedKey, ReadError>>);

impl Stream for KeysWithDocument {
    type Item = Result<DocumentKey, ReadError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.0.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(WalkedKey {
                    has_document: false,
                    ..
                }))) => {}
                Poll::Ready(item) => return Poll::Ready(item.map(|item| item.map(|key| key.key))),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// The outcome of reading one walked key: an item, an error, or `None` for a key to skip, such as
/// metadata of another document type.
type ReadOutcome<T> = Option<Result<T, ReadError>>;

/// One read in walk order, still running or done.
enum OrderedRead<'a, T> {
    Running(BoxFuture<'a, ReadOutcome<T>>),
    // Boxed, so a running read takes no more room than its future.
    Done(Box<ReadOutcome<T>>),
}

/// The outcomes of a read of each walked key, up to [`METADATA_READS_IN_FLIGHT`] reads at once, in
/// walk order.
///
/// A read that finishes early waits in the queue until every read before it has finished.
struct OrderedReads<'a, T, F> {
    walk: Option<BoxStream<'static, Result<WalkedKey, ReadError>>>,
    read: F,
    in_flight: VecDeque<OrderedRead<'a, T>>,
}

impl<'a, T, F> OrderedReads<'a, T, F>
where
    F: FnMut(WalkedKey) -> BoxFuture<'a, ReadOutcome<T>>,
{
    const fn new(walk: BoxStream<'static, Result<WalkedKey, ReadError>>, read: F) -> Self {
        Self {
            walk: Some(walk),
            read,
            in_flight: VecDeque::new(),
        }
    }

    /// Takes keys from the walk and starts their reads, until the reads in flight reach the bound
    /// or the walk has no key ready.
    fn start_reads(&mut self, cx: &mut Context<'_>) {
        while self.in_flight.len() < METADATA_READS_IN_FLIGHT {
            let Some(walk) = self.walk.as_mut() else {
                return;
            };
            let read = match walk.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(walked))) => OrderedRead::Running((self.read)(walked)),
                Poll::Ready(Some(Err(error))) => OrderedRead::Done(Box::new(Some(Err(error)))),
                Poll::Ready(None) => {
                    self.walk = None;
                    return;
                }
                Poll::Pending => return,
            };
            self.in_flight.push_back(read);
        }
    }
}

impl<'a, T, F> Stream for OrderedReads<'a, T, F>
where
    F: FnMut(WalkedKey) -> BoxFuture<'a, ReadOutcome<T>>,
{
    type Item = Result<T, ReadError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            this.start_reads(cx);
            for read in &mut this.in_flight {
                if let OrderedRead::Running(future) = read
                    && let Poll::Ready(outcome) = future.as_mut().poll(cx)
                {
                    *read = OrderedRead::Done(Box::new(outcome));
                }
            }
            match this.in_flight.front() {
                None if this.walk.is_none() => return Poll::Ready(None),
                None | Some(OrderedRead::Running(_)) => return Poll::Pending,
                Some(OrderedRead::Done(_)) => {}
            }
            if let Some(OrderedRead::Done(outcome)) = this.in_flight.pop_front()
                && let Some(item) = *outcome
            {
                return Poll::Ready(Some(item));
            }
        }
    }
}

// The stream never pins a field: each running read sits in its own `Box`. Without this impl, the
// stream is `Unpin` only if `T` is, and a metadata context need not be.
impl<T, F> Unpin for OrderedReads<'_, T, F> {}

/// Reads the entry of one walked key, and converts its error into a [`ReadError`].
async fn read_entry<A: CanRead, D: Document>(
    repository: &FilesystemRepository<A, D>,
    walked: WalkedKey,
) -> ReadOutcome<ListedDocument<D::Context>> {
    repository
        .read_listed_document(walked)
        .await
        .map_err(|error| ReadError::Backend(BackendError::from(error)))
        .transpose()
}

impl<A: CanRead, D: Document> FilesystemRepository<A, D> {
    /// Reads the metadata file of a walked key and inspects its document file.
    ///
    /// Returns `None` for metadata of another document type.
    async fn read_listed_document(
        &self,
        walked: WalkedKey,
    ) -> Result<Option<ListedDocument<D::Context>>, FailedFilesystemOperation> {
        let metadata_path = self.metadata_path(&walked.key);
        let metadata_json = read_metadata_file(&metadata_path).await?;
        let stored: StoredDocumentType = parse_metadata(&metadata_json, &metadata_path)?;
        if stored.document_type != D::DOCUMENT_TYPE.to_string() {
            return Ok(None);
        }
        check_document_type::<D>(stored, &metadata_path)?;
        let metadata_modified_at = modified_at(&metadata_path).await?;
        let document = if walked.has_document {
            self.inspect_document(&walked.key).await?
        } else {
            StoredDocument::Missing
        };
        // Parses after both `stat` calls, so a failed `stat` costs no full parse.
        let metadata = parse_metadata(&metadata_json, &metadata_path)?;
        Ok(Some(ListedDocument::new(
            walked.key,
            metadata,
            metadata_modified_at,
            document,
        )))
    }

    /// Returns the size of the document file of a key, or `Missing` if the file does not exist.
    async fn inspect_document(
        &self,
        key: &DocumentKey,
    ) -> Result<StoredDocument, FailedFilesystemOperation> {
        let document_path = self.document_path(key);
        match fs::metadata(&document_path).await {
            Ok(metadata) => Ok(StoredDocument::Present {
                size_in_bytes: metadata.len(),
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(StoredDocument::Missing),
            Err(error) => Err(FailedFilesystemOperation::from_io(
                FilesystemOperation::Inspect,
                &document_path,
                &error,
            )),
        }
    }
}

/// Returns the time a file was last modified.
async fn modified_at(path: &Path) -> Result<DateTime<Utc>, FailedFilesystemOperation> {
    let inspect_error = |error: io::Error| {
        FailedFilesystemOperation::from_io(FilesystemOperation::Inspect, path, &error)
    };
    let metadata = fs::metadata(path).await.map_err(inspect_error)?;
    let modified = metadata.modified().map_err(inspect_error)?;
    Ok(DateTime::<Utc>::from(modified))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::repository::filesystem::error::FailedFilesystemOperationReason;
    use crate::implementations::repository::filesystem::{METADATA_SUFFIX, ReadWrite};
    use crate::tests::fixtures::fake_list_repository::collect;
    use crate::tests::fixtures::sample_document::{
        SampleContext, SampleDocument, sample_metadata_of,
    };
    use crate::traits::repository::WriteRepository;

    const PRODUCED_AT: &str = "2026-09-22T20:05:30Z";

    fn repository_at(root: impl Into<PathBuf>) -> FilesystemRepository<ReadWrite, SampleDocument> {
        FilesystemRepository::builder()
            .read_write()
            .document_type::<SampleDocument>()
            .root(root)
            .build()
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn key(path: &str) -> DocumentKey {
        DocumentKey::new(path)
            .expect("Given a valid hardcoded relative path, the key should always build")
    }

    /// Writes a `{}` document under the key through `persist`.
    async fn persist_at(repository: &FilesystemRepository<ReadWrite, SampleDocument>, path: &str) {
        let record = SampleDocument::from_parts(
            key(path),
            b"{}".to_vec(),
            sample_metadata_of(b"{}", PRODUCED_AT),
        );
        repository
            .persist(record)
            .await
            .expect("Given a writable temp root, persisting a document should always succeed");
    }

    /// Writes the given text as a file under the root, with its parent directories.
    fn write_file(root: &Path, path: &str, text: &str) {
        let path = root.join(path);
        let directory = path
            .parent()
            .expect("Given a path under a root, the path should always have a parent");
        std::fs::create_dir_all(directory).expect(
            "Given a writable temp directory, creating subdirectories should always succeed",
        );
        std::fs::write(&path, text)
            .expect("Given an existing directory, writing a file should always succeed");
    }

    /// Returns the JSON of the sample metadata with one field replaced.
    fn metadata_json_with(field: &str, value: serde_json::Value) -> String {
        let mut metadata = serde_json::to_value(sample_metadata_of(b"{}", PRODUCED_AT))
            .expect("Given metadata of owned fields, serialization should always succeed");
        metadata[field] = value;
        metadata.to_string()
    }

    /// Reduces an entry to its key and its document file, the parts a test can know in advance.
    fn key_and_document(
        item: Result<ListedDocument<SampleContext>, ReadError>,
    ) -> Result<(DocumentKey, StoredDocument), ReadError> {
        item.map(|entry| (entry.key().clone(), entry.document()))
    }

    async fn list_key_and_document(
        repository: &FilesystemRepository<ReadWrite, SampleDocument>,
        prefix: KeyPrefix,
    ) -> Vec<Result<(DocumentKey, StoredDocument), ReadError>> {
        collect(repository.list(prefix))
            .await
            .into_iter()
            .map(key_and_document)
            .collect()
    }

    fn read_error(reason: FailedFilesystemOperationReason, path: PathBuf) -> ReadError {
        ReadError::Backend(BackendError::from(FailedFilesystemOperation::new(
            reason, path,
        )))
    }

    const fn implements_list_repository<T: ListRepository>() {}
    #[test]
    const fn should_implement_list_repository_for_filesystem_repository() {
        implements_list_repository::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    #[tokio::test]
    async fn should_yield_the_entry_as_present_when_the_document_and_its_metadata_exist() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        persist_at(&repository, "sec/x.json").await;
        let prefix = KeyPrefix::root();

        let expected_result = vec![Ok((
            key("sec/x.json"),
            StoredDocument::Present { size_in_bytes: 2 },
        ))];

        let result = list_key_and_document(&repository, prefix).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_the_stored_metadata_when_listing_a_document() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        persist_at(&repository, "sec/x.json").await;
        let prefix = KeyPrefix::root();

        let expected_result = vec![Some(sample_metadata_of(b"{}", PRODUCED_AT))];

        let result: Vec<_> = collect(repository.list(prefix))
            .await
            .into_iter()
            .map(|item| item.ok().map(|entry| entry.metadata().clone()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_the_entry_as_missing_when_only_the_metadata_exists() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        persist_at(&repository, "sec/x.json").await;
        std::fs::remove_file(root.path().join("sec/x.json"))
            .expect("Given a persisted document, removing its file should always succeed");
        let prefix = KeyPrefix::root();

        let expected_result = vec![Ok((key("sec/x.json"), StoredDocument::Missing))];

        let result = list_key_and_document(&repository, prefix).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_nothing_when_a_document_has_no_metadata() {
        let root = temporary_root();
        write_file(root.path(), "sec/x.json", "{}");
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result: Vec<Result<(DocumentKey, StoredDocument), ReadError>> = vec![];

        let result = list_key_and_document(&repository, prefix).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_nothing_when_the_directory_holds_only_temporary_files_of_a_write() {
        let root = temporary_root();
        let metadata_json = serde_json::to_string(&sample_metadata_of(b"{}", PRODUCED_AT))
            .expect("Given metadata of owned fields, serialization should always succeed");
        write_file(root.path(), "sec/.x.json.4242.0.tmp", "{}");
        write_file(
            root.path(),
            "sec/.x.json.meta.json.4242.1.tmp",
            &metadata_json,
        );
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result: Vec<Result<(DocumentKey, StoredDocument), ReadError>> = vec![];

        let result = list_key_and_document(&repository, prefix).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_skip_the_entry_when_the_metadata_holds_another_document_type() {
        let root = temporary_root();
        let metadata_json = metadata_json_with("document_type", "other_document".into());
        write_file(root.path(), "sec/x.json", "{}");
        write_file(
            root.path(),
            &format!("sec/x.json{METADATA_SUFFIX}"),
            &metadata_json,
        );
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result: Vec<Result<(DocumentKey, StoredDocument), ReadError>> = vec![];

        let result = list_key_and_document(&repository, prefix).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_an_error_when_the_metadata_holds_another_version_of_the_document_type() {
        let root = temporary_root();
        let metadata_json = metadata_json_with("document_type_version", 2.into());
        write_file(root.path(), "sec/x.json", "{}");
        write_file(
            root.path(),
            &format!("sec/x.json{METADATA_SUFFIX}"),
            &metadata_json,
        );
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result = vec![Err(read_error(
            FailedFilesystemOperationReason::UnexpectedDocumentType {
                found_document_type: "sample_document".to_owned(),
                found_document_type_version: 2,
                expected_document_type: "sample_document".to_owned(),
                expected_document_type_version: 1,
            },
            root.path().join(format!("sec/x.json{METADATA_SUFFIX}")),
        ))];

        let result = list_key_and_document(&repository, prefix).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_an_error_and_continue_when_the_metadata_does_not_parse() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        write_file(root.path(), "sec/a.json", "{}");
        write_file(root.path(), &format!("sec/a.json{METADATA_SUFFIX}"), "{");
        persist_at(&repository, "sec/b.json").await;
        let prefix = KeyPrefix::root();

        let expected_result = vec![
            Err(read_error(
                FailedFilesystemOperationReason::InvalidMetadata { line: 1, column: 1 },
                root.path().join(format!("sec/a.json{METADATA_SUFFIX}")),
            )),
            Ok((
                key("sec/b.json"),
                StoredDocument::Present { size_in_bytes: 2 },
            )),
        ];

        let result = list_key_and_document(&repository, prefix).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_one_error_and_end_when_the_root_is_missing() {
        let root = temporary_root();
        let missing_root = root.path().join("missing");
        let repository = repository_at(&missing_root);
        let prefix = KeyPrefix::root();

        let expected_result = vec![Err(read_error(
            FailedFilesystemOperationReason::MissingRoot,
            missing_root.clone(),
        ))];

        let result = list_key_and_document(&repository, prefix).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_the_entries_in_key_order_when_more_reads_than_the_bound_run() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let paths = [
            "a-b", "a/x", "c01", "c02", "c03", "c04", "c05", "c06", "c07", "c08", "c09", "c10",
            "c11", "c12", "c13", "c14", "c15", "c16", "c17", "c18",
        ];
        for path in paths {
            persist_at(&repository, path).await;
        }
        let prefix = KeyPrefix::root();

        let expected_result = vec![
            key("a-b"),
            key("a/x"),
            key("c01"),
            key("c02"),
            key("c03"),
            key("c04"),
            key("c05"),
            key("c06"),
            key("c07"),
            key("c08"),
            key("c09"),
            key("c10"),
            key("c11"),
            key("c12"),
            key("c13"),
            key("c14"),
            key("c15"),
            key("c16"),
            key("c17"),
            key("c18"),
        ];

        let result: Vec<DocumentKey> = collect(repository.list(prefix))
            .await
            .into_iter()
            .filter_map(|item| item.ok().map(|entry| entry.key().clone()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_list_the_keys_of_every_document_type_when_both_files_exist() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        persist_at(&repository, "sec/a.json").await;
        let other_type_json = metadata_json_with("document_type", "other_document".into());
        write_file(root.path(), "sec/b.json", "{}");
        write_file(
            root.path(),
            &format!("sec/b.json{METADATA_SUFFIX}"),
            &other_type_json,
        );
        persist_at(&repository, "sec/c.json").await;
        std::fs::remove_file(root.path().join("sec/c.json"))
            .expect("Given a persisted document, removing its file should always succeed");
        let prefix = KeyPrefix::root();

        let expected_result = vec![Ok(key("sec/a.json")), Ok(key("sec/b.json"))];

        let result = collect(repository.list_keys(prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_each_completed_document_once_when_a_write_runs_during_the_listing() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let completed = ["sec/b/01", "sec/b/02", "sec/b/03", "sec/b/04", "sec/b/05"];
        for path in completed {
            persist_at(&repository, path).await;
        }
        let new_paths = ["sec/a/01", "sec/a/02", "sec/b/00", "sec/b/06", "sec/c/01"];
        let writes = async {
            for path in new_paths {
                persist_at(&repository, path).await;
            }
        };
        let prefix = KeyPrefix::root();

        let expected_result = vec![
            key("sec/b/01"),
            key("sec/b/02"),
            key("sec/b/03"),
            key("sec/b/04"),
            key("sec/b/05"),
        ];

        let (listing, ()) = tokio::join!(collect(repository.list(prefix)), writes);
        let result: Vec<DocumentKey> = listing
            .into_iter()
            .filter_map(|item| item.ok().map(|entry| entry.key().clone()))
            .filter(|listed| completed.iter().any(|path| *listed == key(path)))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_the_modified_time_of_the_metadata_file_when_listing_a_document() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        persist_at(&repository, "sec/x.json").await;
        let modified_at = DateTime::<Utc>::from_timestamp(1_700_000_000, 0)
            .expect("Given a hardcoded timestamp in range, the time should always build");
        std::fs::File::options()
            .write(true)
            .open(root.path().join(format!("sec/x.json{METADATA_SUFFIX}")))
            .and_then(|file| file.set_modified(modified_at.into()))
            .expect(
                "Given a persisted metadata file, setting its modified time should always succeed",
            );
        let prefix = KeyPrefix::root();

        let expected_result = vec![Some(
            DateTime::<Utc>::from_timestamp(1_700_000_000, 0)
                .expect("Given a hardcoded timestamp in range, the time should always build"),
        )];

        let result: Vec<_> = collect(repository.list(prefix))
            .await
            .into_iter()
            .map(|item| item.ok().map(|entry| entry.metadata_modified_at()))
            .collect();

        assert_eq!(result, expected_result);
    }

    /// A read that finishes after the given count of polls, and yields the key it read.
    struct ReadAfterPolls {
        key: Option<DocumentKey>,
        polls_left: usize,
    }

    impl Future for ReadAfterPolls {
        type Output = ReadOutcome<DocumentKey>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.polls_left == 0 {
                return Poll::Ready(self.key.take().map(Ok));
            }
            self.polls_left -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }

    #[tokio::test]
    async fn should_yield_the_keys_in_walk_order_when_later_reads_finish_first() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        for path in ["sec/a.json", "sec/b.json", "sec/c.json"] {
            persist_at(&repository, path).await;
        }
        let walk = repository.walk_keys(&KeyPrefix::root());
        let mut polls_left = 3;
        let reads_in_reverse = move |walked: WalkedKey| -> BoxFuture<'static, _> {
            polls_left -= 1;
            Box::pin(ReadAfterPolls {
                key: Some(walked.key),
                polls_left,
            })
        };

        let expected_result = vec![
            Ok(key("sec/a.json")),
            Ok(key("sec/b.json")),
            Ok(key("sec/c.json")),
        ];

        let result = collect(Box::pin(OrderedReads::new(walk, reads_in_reverse))).await;

        assert_eq!(result, expected_result);
    }
}
