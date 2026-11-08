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
    /// repository of that type lists it. It reads up to 16 metadata files at once, and yields the
    /// entries in the order of [`list_keys`](ListRepository::list_keys).
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
        Box::pin(MetadataReads {
            repository: self,
            walk: Some(self.walk_keys(&filter)),
            in_flight: VecDeque::with_capacity(METADATA_READS_IN_FLIGHT),
        })
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

/// The outcome of reading one walked key: an entry, an error, or `None` for metadata of another
/// document type.
type ReadOutcome<C> = Option<Result<ListedDocument<C>, ReadError>>;

/// One metadata read in walk order, still running or done.
enum MetadataRead<'a, C> {
    Running(BoxFuture<'a, ReadOutcome<C>>),
    // Boxed, so a running read takes no more room than its future.
    Done(Box<ReadOutcome<C>>),
}

/// The entries of a walk, read up to [`METADATA_READS_IN_FLIGHT`] at once, in walk order.
struct MetadataReads<'a, A, D: Document> {
    repository: &'a FilesystemRepository<A, D>,
    walk: Option<BoxStream<'static, Result<WalkedKey, ReadError>>>,
    in_flight: VecDeque<MetadataRead<'a, D::Context>>,
}

impl<A: CanRead, D: Document> Stream for MetadataReads<'_, A, D>
where
    D::Context: Send,
{
    type Item = Result<ListedDocument<D::Context>, ReadError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            this.start_reads(cx);
            for read in &mut this.in_flight {
                if let MetadataRead::Running(future) = read
                    && let Poll::Ready(outcome) = future.as_mut().poll(cx)
                {
                    *read = MetadataRead::Done(Box::new(outcome));
                }
            }
            match this.in_flight.front() {
                None if this.walk.is_none() => return Poll::Ready(None),
                None | Some(MetadataRead::Running(_)) => return Poll::Pending,
                Some(MetadataRead::Done(_)) => {}
            }
            if let Some(MetadataRead::Done(outcome)) = this.in_flight.pop_front()
                && let Some(item) = *outcome
            {
                return Poll::Ready(Some(item));
            }
        }
    }
}

// The stream never pins a field: each running read sits in its own `Box`. So moving the stream
// moves no pinned value, and it is `Unpin` for every metadata context.
impl<A, D: Document> Unpin for MetadataReads<'_, A, D> {}

impl<A: CanRead, D: Document> MetadataReads<'_, A, D>
where
    D::Context: Send,
{
    /// Takes keys from the walk and starts their reads, until the reads in flight reach the bound
    /// or the walk has no key ready.
    fn start_reads(&mut self, cx: &mut Context<'_>) {
        while self.in_flight.len() < METADATA_READS_IN_FLIGHT {
            let Some(walk) = self.walk.as_mut() else {
                return;
            };
            let read = match walk.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(walked))) => {
                    MetadataRead::Running(Box::pin(read_entry(self.repository, walked)))
                }
                Poll::Ready(Some(Err(error))) => MetadataRead::Done(Box::new(Some(Err(error)))),
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

/// Reads the entry of one walked key, and converts its error into a [`ReadError`].
async fn read_entry<A: CanRead, D: Document>(
    repository: &FilesystemRepository<A, D>,
    walked: WalkedKey,
) -> ReadOutcome<D::Context> {
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
        // Parses last, because the future cannot hold a metadata context that is not `Send`
        // across an `await`.
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
