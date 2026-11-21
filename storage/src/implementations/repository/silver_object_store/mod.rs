//! # Silver Object Store
//!
//! Provides [`SilverObjectStore`], which stores each silver object once as a file under a root
//! directory, and the [`SilverObject`] and [`ListedObject`] it reads, writes, and lists.
//!
//! ## Usage
//!
//! ```rust
//! use storage::implementations::repository::filesystem::DocumentKey;
//! use storage::implementations::repository::silver_object_store::{
//!     SilverObject, SilverObjectStore,
//! };
//! use storage::{WriteError, WriteRepository};
//!
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() {
//! # let root = tempfile::TempDir::new().expect("Given a writable temp directory, it should build");
//! let store = SilverObjectStore::new(root.path());
//! let key = DocumentKey::new("notes/part-0.parquet")
//!     .expect("Given a valid hardcoded relative path, the key should always build");
//! let object = SilverObject::new(key.clone(), b"PAR1".as_slice());
//! store
//!     .persist(object.clone())
//!     .await
//!     .expect("Given an empty root, the first write of a key should always succeed");
//!
//! let expected_result = WriteError::conflicting_write("Object 'notes/part-0.parquet' exists");
//!
//! let result = store
//!     .persist(object)
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
use bytes::Bytes;
use futures_core::stream::{BoxStream, Stream};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::{BackendError, ReadError, WriteError};
use crate::implementations::repository::filesystem::implementations::key_walk::{
    KeyFiles, WalkedKey, key_text, walk_keys,
};
use crate::implementations::repository::filesystem::{
    DocumentKey, FailedFilesystemOperation, FilesystemOperation, KeyPrefix, check_root,
};
use crate::traits::repository::{ListRepository, ReadRepository, WriteRepository};

/// Stores silver objects as files under one root directory, and writes each object once.
///
/// An object lives at the root joined with its [`DocumentKey`]. `persist` creates the file only if
/// no file exists at that path. Otherwise it returns [`WriteError::ConflictingWrite`] and changes
/// nothing. The store writes no metadata file next to an object. `get` reads a whole object, and
/// [`ListRepository`] lists the objects under a [`KeyPrefix`].
///
/// A reader can see an object while its write runs. A silver object becomes visible to readers
/// only through a later control object, so a reader never reads it before the write ends.
/// If the process ends during a write, the partial file stays, and each later write of the key
/// fails with [`WriteError::ConflictingWrite`]. Silver writes each attempt of a task under its own
/// key, so a retry never meets that file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SilverObjectStore {
    root: PathBuf,
}

/// One silver object: its key and its bytes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SilverObject {
    /// The path of the object under the root.
    pub key: DocumentKey,
    /// The bytes of the object.
    pub bytes: Bytes,
}

/// One object that a listing found: its key and its size.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListedObject {
    /// The path of the object under the root.
    pub key: DocumentKey,
    /// The number of bytes the object holds.
    pub size_in_bytes: u64,
}

impl SilverObject {
    /// Creates a [`SilverObject`] from its key and its bytes.
    #[must_use]
    pub fn new(key: DocumentKey, bytes: impl Into<Bytes>) -> Self {
        Self {
            key,
            bytes: bytes.into(),
        }
    }
}

impl SilverObjectStore {
    /// Creates a [`SilverObjectStore`] under the root directory.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the root directory every key resolves under.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the path on disk of the object a key names.
    #[must_use]
    pub fn object_path(&self, key: &DocumentKey) -> PathBuf {
        self.root.join(key.as_path())
    }

    /// Creates the file of the object, writes its bytes, and syncs them.
    ///
    /// Returns `Ok(false)` if a file exists at the path. If the write fails after the file was
    /// created, it removes the file.
    async fn create_object(
        &self,
        object: &SilverObject,
    ) -> Result<bool, FailedFilesystemOperation> {
        check_root(&self.root).await?;
        let path = self.object_path(&object.key);
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
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return Ok(false),
            Err(error) => return Err(io_error(FilesystemOperation::CreateFile)(error)),
        };
        let written = async {
            file.write_all(&object.bytes)
                .await
                .map_err(io_error(FilesystemOperation::Write))?;
            file.sync_all()
                .await
                .map_err(io_error(FilesystemOperation::Sync))
        }
        .await;
        if written.is_err() {
            // The file holds a part of the bytes. Removing it lets a retry create it again. If the
            // removal fails, the write error still names the cause.
            let _ = fs::remove_file(&path).await;
        }
        written.map(|()| true)
    }

    /// Reads the whole object a key names, or `None` if no file exists at its path.
    async fn read_object(
        &self,
        key: DocumentKey,
    ) -> Result<Option<SilverObject>, FailedFilesystemOperation> {
        check_root(&self.root).await?;
        let path = self.object_path(&key);
        match fs::read(&path).await {
            Ok(bytes) => Ok(Some(SilverObject::new(key, bytes))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(FailedFilesystemOperation::from_io(
                FilesystemOperation::Read,
                &path,
                &error,
            )),
        }
    }

    /// Returns the size of the object a walked key names.
    async fn list_object(&self, walked: WalkedKey) -> Result<ListedObject, ReadError> {
        let path = self.object_path(&walked.key);
        let metadata = fs::metadata(&path).await.map_err(|error| {
            read_error(FailedFilesystemOperation::from_io(
                FilesystemOperation::Inspect,
                &path,
                &error,
            ))
        })?;
        Ok(ListedObject {
            key: walked.key,
            size_in_bytes: metadata.len(),
        })
    }
}

#[async_trait]
impl WriteRepository for SilverObjectStore {
    type Record = SilverObject;

    /// Writes the object if no object exists at its key.
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`WriteError::ConflictingWrite`] if an object exists at the key. The write changes nothing.
    /// - [`WriteError::Backend`] that holds a [`FailedFilesystemOperation`] if the root does not
    ///   exist or is not a directory, or the filesystem cannot create, write, or sync the file.
    async fn persist(&self, record: SilverObject) -> Result<(), WriteError> {
        match self.create_object(&record).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(WriteError::conflicting_write(format!(
                "Object '{}' exists",
                key_text(record.key.as_path())
            ))),
            Err(error) => Err(WriteError::Backend(BackendError::from(error))),
        }
    }
}

#[async_trait]
impl ReadRepository for SilverObjectStore {
    type Record = SilverObject;
    type Key = DocumentKey;

    /// Reads the whole object at the key. Returns `Ok(None)` if no object exists there.
    ///
    /// # Errors
    ///
    /// Returns [`ReadError::Backend`] that holds a [`FailedFilesystemOperation`] if the root does
    /// not exist or is not a directory, or the filesystem cannot read the file.
    async fn get(&self, key: DocumentKey) -> Result<Option<SilverObject>, ReadError> {
        self.read_object(key).await.map_err(read_error)
    }
}

impl ListRepository for SilverObjectStore {
    type Key = DocumentKey;
    type Entry = ListedObject;
    type Filter = KeyPrefix;

    /// Lists the key of each object under the prefix, sorted by the bytes of the key text.
    ///
    /// Each regular file is an object. If the filter carries a cursor, it yields only the keys that
    /// sort after it.
    ///
    /// # Errors
    ///
    /// Yields a [`ReadError::Backend`] item for each error of the walk: the root does not exist or
    /// is not a directory, or the filesystem cannot list a directory. An error about a directory
    /// below the prefix does not end the stream.
    fn list_keys(&self, filter: KeyPrefix) -> BoxStream<'_, Result<DocumentKey, ReadError>> {
        Box::pin(WalkedObjects::keys(self, &filter))
    }

    /// Lists each object under the prefix with its size, in the order of
    /// [`list_keys`](ListRepository::list_keys). Reads no bytes of an object.
    ///
    /// # Errors
    ///
    /// Yields a [`ReadError::Backend`] item for each error of
    /// [`list_keys`](ListRepository::list_keys), and for an object the filesystem cannot inspect.
    /// The stream continues after an item about one object.
    fn list(&self, filter: KeyPrefix) -> BoxStream<'_, Result<ListedObject, ReadError>> {
        Box::pin(WalkedObjects::entries(self, &filter))
    }
}

/// A future that turns one walked key into an item of a listing.
type ToItem<'a, T> = Pin<Box<dyn Future<Output = Result<T, ReadError>> + Send + 'a>>;

/// The objects of a walk, one at a time, as keys or as entries with their size.
struct WalkedObjects<'a, T> {
    store: &'a SilverObjectStore,
    walk: BoxStream<'static, Result<WalkedKey, ReadError>>,
    pending: Option<ToItem<'a, T>>,
    to_item: fn(&'a SilverObjectStore, WalkedKey) -> ToItem<'a, T>,
}

impl<'a> WalkedObjects<'a, DocumentKey> {
    fn keys(store: &'a SilverObjectStore, filter: &KeyPrefix) -> Self {
        Self::new(store, filter, |_, walked| {
            Box::pin(async move { Ok(walked.key) })
        })
    }
}

impl<'a> WalkedObjects<'a, ListedObject> {
    fn entries(store: &'a SilverObjectStore, filter: &KeyPrefix) -> Self {
        Self::new(store, filter, |store, walked| {
            Box::pin(store.list_object(walked))
        })
    }
}

impl<'a, T> WalkedObjects<'a, T> {
    fn new(
        store: &'a SilverObjectStore,
        filter: &KeyPrefix,
        to_item: fn(&'a SilverObjectStore, WalkedKey) -> ToItem<'a, T>,
    ) -> Self {
        Self {
            store,
            walk: walk_keys(&store.root, filter, KeyFiles::EveryFile),
            pending: None,
            to_item,
        }
    }
}

impl<T> Stream for WalkedObjects<'_, T> {
    type Item = Result<T, ReadError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
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
        Poll::Ready(Some(item))
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

    fn object(path: &str, bytes: &'static [u8]) -> SilverObject {
        SilverObject::new(key(path), bytes)
    }

    async fn store_with(root: &TempDir, objects: Vec<SilverObject>) -> SilverObjectStore {
        let store = SilverObjectStore::new(root.path());
        for object in objects {
            store
                .persist(object)
                .await
                .expect("Given distinct keys under an existing root, each write should succeed");
        }
        store
    }

    const fn implements_read_write_and_list<T: ReadWriteRepository + ReadListRepository>() {}
    #[test]
    const fn should_implement_read_write_and_read_list_repository_for_silver_object_store() {
        implements_read_write_and_list::<SilverObjectStore>();
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_silver_object_store() {
        implements_auto_traits::<SilverObjectStore>();
    }

    #[tokio::test]
    async fn should_read_the_whole_object_when_a_write_created_it() {
        let root = temporary_root();
        let store = store_with(&root, vec![object("notes/part-0.parquet", b"PAR1")]).await;
        let key = key("notes/part-0.parquet");

        let expected_result = Some(object("notes/part-0.parquet", b"PAR1"));

        let result = store
            .get(key)
            .await
            .expect("Given an object the store wrote, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_conflicting_write_when_the_object_exists() {
        let root = temporary_root();
        let store = store_with(&root, vec![object("notes/part-0.parquet", b"PAR1")]).await;
        let second = object("notes/part-0.parquet", b"OTHER");

        let expected_result = Err(WriteError::conflicting_write(
            "Object 'notes/part-0.parquet' exists",
        ));

        let result = store.persist(second).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_first_bytes_when_a_second_write_of_the_key_fails() {
        let root = temporary_root();
        let store = store_with(&root, vec![object("notes/part-0.parquet", b"PAR1")]).await;
        let _conflict = store
            .persist(object("notes/part-0.parquet", b"OTHER"))
            .await;

        let expected_result = b"PAR1".to_vec();

        let result = std::fs::read(root.path().join("notes/part-0.parquet"))
            .expect("Given an object the store wrote, the file should always be readable");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_no_object_exists_at_the_key() {
        let root = temporary_root();
        let store = SilverObjectStore::new(root.path());
        let key = key("notes/part-0.parquet");

        let expected_result = None;

        let result = store
            .get(key)
            .await
            .expect("Given an existing root, the read of an absent key should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_no_file_beside_the_object_when_persisting() {
        let root = temporary_root();
        store_with(&root, vec![object("notes/part-0.parquet", b"PAR1")]).await;

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
        let store = SilverObjectStore::new(&root);
        let record = object("notes/part-0.parquet", b"PAR1");

        let expected_result = Err(WriteError::Backend(BackendError::from(
            FailedFilesystemOperation::new(FailedFilesystemOperationReason::MissingRoot, &root),
        )));

        let result = store.persist(record).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_list_the_keys_under_the_prefix_in_byte_order() {
        let root = temporary_root();
        let store = store_with(
            &root,
            vec![
                object("notes/b.parquet", b"B"),
                object("notes/a/x.parquet", b"X"),
                object("daily/c.parquet", b"C"),
            ],
        )
        .await;
        let prefix = KeyPrefix::from(key("notes"));

        let expected_result = vec![Ok(key("notes/a/x.parquet")), Ok(key("notes/b.parquet"))];

        let result = collect(store.list_keys(prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_list_each_object_with_its_size() {
        let root = temporary_root();
        let store = store_with(
            &root,
            vec![
                object("notes/a.parquet", b"PAR1"),
                object("notes/b.parquet", b"PA"),
            ],
        )
        .await;
        let prefix = KeyPrefix::root();

        let expected_result = vec![
            Ok(ListedObject {
                key: key("notes/a.parquet"),
                size_in_bytes: 4,
            }),
            Ok(ListedObject {
                key: key("notes/b.parquet"),
                size_in_bytes: 2,
            }),
        ];

        let result = collect(store.list(prefix)).await;

        assert_eq!(result, expected_result);
    }
}
