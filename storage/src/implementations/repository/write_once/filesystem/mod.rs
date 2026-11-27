//! # Write-Once Filesystem Repository
//!
//! Provides [`WriteOnceFilesystemRepository`], which stores each blob once as a file under a root
//! directory.
//!
//! ## Usage
//!
//! ```rust
//! use storage::implementations::repository::filesystem::DocumentKey;
//! use storage::implementations::repository::write_once::{
//!     WriteOnceFilesystemRepository, Blob,
//! };
//! use storage::{WriteError, WriteRepository};
//!
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() {
//! # let root = tempfile::TempDir::new().expect("Given a writable temp directory, it should build");
//! let store = WriteOnceFilesystemRepository::new(root.path());
//! let key = DocumentKey::new("notes/part-0.parquet")
//!     .expect("Given a valid hardcoded relative path, the key should always build");
//! let blob = Blob::new(key.clone(), b"PAR1".as_slice());
//! store
//!     .persist(blob.clone())
//!     .await
//!     .expect("Given an empty root, the first write of a key should always succeed");
//!
//! let expected_result = WriteError::conflicting_write("Blob 'notes/part-0.parquet' exists");
//!
//! let result = store
//!     .persist(blob)
//!     .await
//!     .expect_err("A second write of one key should always fail");
//!
//! assert_eq!(result, expected_result);
//! # }
//! ```

use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use async_trait::async_trait;
use futures_core::stream::{BoxStream, Stream};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::{Blob, ListedBlob};
use crate::error::{BackendError, ReadError, WriteError};
use crate::implementations::repository::filesystem::implementations::key_walk::{
    KeyFiles, WalkedKey, key_text, walk_keys,
};
use crate::implementations::repository::filesystem::{
    DocumentKey, FailedFilesystemOperation, FilesystemOperation, KeyPrefix, check_root,
};
use crate::traits::repository::{ListRepository, ReadRepository, WriteRepository};

/// Stores blobs as files under one root directory, and writes each blob once.
///
/// A blob lives at the root joined with its [`DocumentKey`]. `persist` creates the file only if
/// no file exists at that path. Otherwise it returns [`WriteError::ConflictingWrite`] and changes
/// nothing. The repository writes no metadata file next to a blob. `get` reads a whole blob, and
/// [`ListRepository`] lists the blobs under a [`KeyPrefix`].
///
/// A reader that opens the file during the write reads only part of the bytes. A caller that
/// must not expose a partial blob publishes its key only after `persist` returns. `persist` syncs
/// the file but not its directory, so a crash of the machine can lose a blob that `persist`
/// reported as written. Tests and local runs use this backend.
///
/// If the process ends during a write, the partial file stays, and each later write of the key
/// fails with [`WriteError::ConflictingWrite`]. A caller that retries a write uses a new key, so
/// the retry never meets that file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WriteOnceFilesystemRepository {
    root: PathBuf,
}

impl WriteOnceFilesystemRepository {
    /// Creates a [`WriteOnceFilesystemRepository`] under the root directory.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the root directory every key resolves under.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the path on disk of the blob a key names.
    #[must_use]
    pub fn blob_path(&self, key: &DocumentKey) -> PathBuf {
        self.root.join(key.as_path())
    }

    /// Creates the file of the blob, writes its bytes, and syncs them.
    ///
    /// Returns `Ok(false)` if a file exists at the path. If the write fails after the file was
    /// created, it removes the file.
    async fn create_blob(&self, blob: &Blob) -> Result<bool, FailedFilesystemOperation> {
        check_root(&self.root).await?;
        let path = self.blob_path(&blob.key);
        let io_error = |operation| {
            let path = path.clone();
            move |error: io::Error| FailedFilesystemOperation::from_io(operation, path, &error)
        };
        if let Some(directory) = path.parent() {
            fs::create_dir_all(directory)
                .await
                .map_err(io_error(FilesystemOperation::CreateDirectory))?;
        }
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                // A directory or a symlink at the path also fails with `AlreadyExists`, but neither is
                // a blob.
                return match fs::symlink_metadata(&path).await {
                    Ok(metadata) if metadata.is_file() => Ok(false),
                    _ => Err(io_error(FilesystemOperation::CreateFile)(error)),
                };
            }
            Err(error) => return Err(io_error(FilesystemOperation::CreateFile)(error)),
        };
        let written = async {
            file.write_all(&blob.bytes)
                .await
                .map_err(io_error(FilesystemOperation::Write))?;
            file.sync_all()
                .await
                .map_err(io_error(FilesystemOperation::Sync))
        }
        .await;
        if written.is_err() {
            // The file holds a part of the bytes. Removing it lets a retry create it again. If the
            // removal fails, the write error still names the cause. No test covers this branch:
            // a write to a created file fails only on a full disk or an I/O error, and a unit test
            // can cause neither.
            let _ = fs::remove_file(&path).await;
        }
        written.map(|()| true)
    }

    /// Reads the whole blob a key names, or `None` if no file exists at its path.
    async fn read_blob(&self, key: DocumentKey) -> Result<Option<Blob>, FailedFilesystemOperation> {
        check_root(&self.root).await?;
        let path = self.blob_path(&key);
        match fs::read(&path).await {
            Ok(bytes) => Ok(Some(Blob::new(key, bytes))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(FailedFilesystemOperation::from_io(
                FilesystemOperation::Read,
                &path,
                &error,
            )),
        }
    }

    /// Returns the size of the blob a walked key names, or `None` if the file disappeared after
    /// the walk, as the file of a failed write does.
    async fn list_blob(&self, walked: WalkedKey) -> Option<Result<ListedBlob, ReadError>> {
        let path = self.blob_path(&walked.key);
        match fs::metadata(&path).await {
            Ok(metadata) => Some(Ok(ListedBlob {
                key: walked.key,
                size_in_bytes: metadata.len(),
            })),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => Some(Err(read_error(FailedFilesystemOperation::from_io(
                FilesystemOperation::Inspect,
                &path,
                &error,
            )))),
        }
    }
}

#[async_trait]
impl WriteRepository for WriteOnceFilesystemRepository {
    type Record = Blob;

    /// Writes the blob if no blob exists at its key.
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`WriteError::ConflictingWrite`] if a blob exists at the key. The write changes nothing.
    /// - [`WriteError::Backend`] that holds a [`FailedFilesystemOperation`] if the root does not
    ///   exist or is not a directory, a directory sits at the key, or the filesystem cannot create,
    ///   write, or sync the file.
    async fn persist(&self, record: Blob) -> Result<(), WriteError> {
        match self.create_blob(&record).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(WriteError::conflicting_write(format!(
                "Blob '{}' exists",
                key_text(record.key.as_path())
            ))),
            Err(error) => Err(WriteError::Backend(BackendError::from(error))),
        }
    }
}

#[async_trait]
impl ReadRepository for WriteOnceFilesystemRepository {
    type Record = Blob;
    type Key = DocumentKey;

    /// Reads the whole blob at the key. Returns `Ok(None)` if no blob exists there.
    ///
    /// # Errors
    ///
    /// Returns [`ReadError::Backend`] that holds a [`FailedFilesystemOperation`] if the root does
    /// not exist or is not a directory, or the filesystem cannot read the file.
    async fn get(&self, key: DocumentKey) -> Result<Option<Blob>, ReadError> {
        self.read_blob(key).await.map_err(read_error)
    }
}

impl ListRepository for WriteOnceFilesystemRepository {
    type Key = DocumentKey;
    type Entry = ListedBlob;
    type Filter = KeyPrefix;

    /// Lists the key of each blob under the prefix, sorted by the bytes of the key text.
    ///
    /// Each regular file is a blob. If the filter carries a cursor, it yields only the keys that
    /// sort after it.
    ///
    /// # Errors
    ///
    /// Yields a [`ReadError::Backend`] item for each error of the walk: the root does not exist or
    /// is not a directory, or the filesystem cannot list a directory. An error about a directory
    /// below the prefix does not end the stream.
    fn list_keys(&self, filter: KeyPrefix) -> BoxStream<'_, Result<DocumentKey, ReadError>> {
        Box::pin(WalkedBlobs::keys(self, &filter))
    }

    /// Lists each blob under the prefix with its size, in the order of
    /// [`list_keys`](ListRepository::list_keys). Reads no bytes of a blob. Skips a blob
    /// whose file disappears after the walk found it.
    ///
    /// # Errors
    ///
    /// Yields a [`ReadError::Backend`] item for each error of
    /// [`list_keys`](ListRepository::list_keys), and for a blob the filesystem cannot inspect.
    /// The stream continues after an item about one blob.
    fn list(&self, filter: KeyPrefix) -> BoxStream<'_, Result<ListedBlob, ReadError>> {
        Box::pin(WalkedBlobs::entries(self, &filter))
    }
}

/// A future that turns one walked key into an item of a listing, or `None` for a key to skip.
type ToItem<'a, T> = Pin<Box<dyn Future<Output = Option<Result<T, ReadError>>> + Send + 'a>>;

/// The blobs of a walk, one at a time, as keys or as entries with their size.
struct WalkedBlobs<'a, T> {
    store: &'a WriteOnceFilesystemRepository,
    walk: BoxStream<'static, Result<WalkedKey, ReadError>>,
    pending: Option<ToItem<'a, T>>,
    to_item: fn(&'a WriteOnceFilesystemRepository, WalkedKey) -> ToItem<'a, T>,
}

impl<'a> WalkedBlobs<'a, DocumentKey> {
    fn keys(store: &'a WriteOnceFilesystemRepository, filter: &KeyPrefix) -> Self {
        Self::new(store, filter, |_, walked| {
            Box::pin(async move { Some(Ok(walked.key)) })
        })
    }
}

impl<'a> WalkedBlobs<'a, ListedBlob> {
    fn entries(store: &'a WriteOnceFilesystemRepository, filter: &KeyPrefix) -> Self {
        Self::new(store, filter, |store, walked| {
            Box::pin(store.list_blob(walked))
        })
    }
}

impl<'a, T> WalkedBlobs<'a, T> {
    fn new(
        store: &'a WriteOnceFilesystemRepository,
        filter: &KeyPrefix,
        to_item: fn(&'a WriteOnceFilesystemRepository, WalkedKey) -> ToItem<'a, T>,
    ) -> Self {
        Self {
            store,
            walk: walk_keys(&store.root, filter, KeyFiles::EveryFile),
            pending: None,
            to_item,
        }
    }
}

impl<T> Stream for WalkedBlobs<'_, T> {
    type Item = Result<T, ReadError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            if this.pending.is_none() {
                match ready!(this.walk.as_mut().poll_next(cx)) {
                    Some(Ok(walked)) => this.pending = Some((this.to_item)(this.store, walked)),
                    Some(Err(error)) => return Poll::Ready(Some(Err(error))),
                    None => return Poll::Ready(None),
                }
            }
            let pending = this
                .pending
                .as_mut()
                .expect("Given a walked key, the stream should always hold its pending item");
            let item = ready!(pending.as_mut().poll(cx));
            this.pending = None;
            if item.is_some() {
                return Poll::Ready(item);
            }
        }
    }
}

fn read_error(error: FailedFilesystemOperation) -> ReadError {
    ReadError::Backend(BackendError::from(error))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::repository::filesystem::FailedFilesystemOperationReason;
    use crate::tests::fixtures::fake_list_repository::collect;
    use crate::traits::repository::{ReadListRepository, ReadWriteRepository};

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn key(path: &str) -> DocumentKey {
        DocumentKey::new(path)
            .expect("Given a valid hardcoded relative path, the key should always build")
    }

    fn blob(path: &str, bytes: &'static [u8]) -> Blob {
        Blob::new(key(path), bytes)
    }

    async fn store_with(root: &TempDir, blobs: Vec<Blob>) -> WriteOnceFilesystemRepository {
        let store = WriteOnceFilesystemRepository::new(root.path());
        for blob in blobs {
            store
                .persist(blob)
                .await
                .expect("Given distinct keys under an existing root, each write should succeed");
        }
        store
    }

    const fn implements_read_write_and_list<T: ReadWriteRepository + ReadListRepository>() {}
    #[test]
    const fn should_implement_read_write_and_read_list_repository_for_write_once_filesystem_repository()
     {
        implements_read_write_and_list::<WriteOnceFilesystemRepository>();
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_write_once_filesystem_repository() {
        implements_auto_traits::<WriteOnceFilesystemRepository>();
    }

    #[tokio::test]
    async fn should_read_the_whole_blob_when_a_write_created_it() {
        let root = temporary_root();
        let store = store_with(&root, vec![blob("notes/part-0.parquet", b"PAR1")]).await;
        let key = key("notes/part-0.parquet");

        let expected_result = Some(blob("notes/part-0.parquet", b"PAR1"));

        let result = store
            .get(key)
            .await
            .expect("Given a blob the store wrote, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_conflicting_write_when_the_blob_exists() {
        let root = temporary_root();
        let store = store_with(&root, vec![blob("notes/part-0.parquet", b"PAR1")]).await;
        let second = blob("notes/part-0.parquet", b"OTHER");

        let expected_result = Err(WriteError::conflicting_write(
            "Blob 'notes/part-0.parquet' exists",
        ));

        let result = store.persist(second).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_first_bytes_when_a_second_write_of_the_key_fails() {
        let root = temporary_root();
        let store = store_with(&root, vec![blob("notes/part-0.parquet", b"PAR1")]).await;
        let _conflict = store.persist(blob("notes/part-0.parquet", b"OTHER")).await;

        let expected_result = b"PAR1".to_vec();

        let result = std::fs::read(root.path().join("notes/part-0.parquet"))
            .expect("Given a blob the store wrote, the file should always be readable");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_no_blob_exists_at_the_key() {
        let root = temporary_root();
        let store = WriteOnceFilesystemRepository::new(root.path());
        let key = key("notes/part-0.parquet");

        let expected_result = None;

        let result = store
            .get(key)
            .await
            .expect("Given an existing root, the read of an absent key should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_no_file_beside_the_blob_when_persisting() {
        let root = temporary_root();
        store_with(&root, vec![blob("notes/part-0.parquet", b"PAR1")]).await;

        let expected_result = vec!["part-0.parquet".to_owned()];

        let result: Vec<String> = std::fs::read_dir(root.path().join("notes"))
            .expect("Given a directory the store created, it should always be listable")
            .map(|entry| {
                entry
                    .expect("Given a listable directory, each entry should always be readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_root_when_persisting_under_a_missing_root() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let store = WriteOnceFilesystemRepository::new(&root);
        let record = blob("notes/part-0.parquet", b"PAR1");

        let expected_result = Err(WriteError::Backend(BackendError::from(
            FailedFilesystemOperation::new(FailedFilesystemOperationReason::MissingRoot, &root),
        )));

        let result = store.persist(record).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_a_create_file_error_when_a_directory_sits_at_the_key() {
        let root = temporary_root();
        let store = store_with(&root, vec![blob("notes/a/x.parquet", b"X")]).await;
        let record = blob("notes/a", b"A");
        let path = root.path().join("notes/a");

        let expected_result = Err(WriteError::Backend(BackendError::from(
            FailedFilesystemOperation::from_io(
                FilesystemOperation::CreateFile,
                &path,
                &io::Error::from(io::ErrorKind::AlreadyExists),
            ),
        )));

        let result = store.persist(record).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_root_when_reading_under_a_missing_root() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let store = WriteOnceFilesystemRepository::new(&root);
        let key = key("notes/part-0.parquet");

        let expected_result = Err(ReadError::Backend(BackendError::from(
            FailedFilesystemOperation::new(FailedFilesystemOperationReason::MissingRoot, &root),
        )));

        let result = store.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_root_when_listing_under_a_missing_root() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let store = WriteOnceFilesystemRepository::new(&root);
        let prefix = KeyPrefix::root();

        let expected_result = vec![Err(ReadError::Backend(BackendError::from(
            FailedFilesystemOperation::new(FailedFilesystemOperationReason::MissingRoot, &root),
        )))];

        let result = collect(store.list_keys(prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_list_the_keys_in_byte_order_for_blobs_under_a_prefix() {
        let root = temporary_root();
        let store = store_with(
            &root,
            vec![
                blob("notes/b.parquet", b"B"),
                blob("notes/a/x.parquet", b"X"),
                blob("daily/c.parquet", b"C"),
            ],
        )
        .await;
        let prefix = KeyPrefix::from(key("notes"));

        let expected_result = vec![Ok(key("notes/a/x.parquet")), Ok(key("notes/b.parquet"))];

        let result = collect(store.list_keys(prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_list_the_size_for_each_blob_under_a_prefix() {
        let root = temporary_root();
        let store = store_with(
            &root,
            vec![
                blob("notes/a.parquet", b"PAR1"),
                blob("notes/b.parquet", b"PA"),
            ],
        )
        .await;
        let prefix = KeyPrefix::root();

        let expected_result = vec![
            Ok(ListedBlob {
                key: key("notes/a.parquet"),
                size_in_bytes: 4,
            }),
            Ok(ListedBlob {
                key: key("notes/b.parquet"),
                size_in_bytes: 2,
            }),
        ];

        let result = collect(store.list(prefix)).await;

        assert_eq!(result, expected_result);
    }
}
