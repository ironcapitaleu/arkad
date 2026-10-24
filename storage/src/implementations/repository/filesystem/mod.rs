//! # Filesystem Repository
//!
//! Provides [`FilesystemRepository`], which stores documents as files under a root
//! directory, and the [`FilesystemRepositoryBuilder`] that builds it.
//!
//! ## Record and Key
//!
//! [`ReadRepository`](crate::ReadRepository) and [`WriteRepository`](crate::WriteRepository) each
//! leave their associated types to the implementor. A `FilesystemRepository<A, D>` sets `Record`
//! to the [`Document`] type `D` in both, and `ReadRepository`'s `Key` to [`DocumentKey`].
//!
//! ## Errors
//!
//! [`DocumentKey::new`] reports an [`InvalidDocumentKey`] for a path the store rejects. The
//! repository takes only a valid [`DocumentKey`], so its own code reports a
//! [`FailedFilesystemOperation`].
//!
//! ## Modules
//!
//! - [`error`]: The [`FailedFilesystemOperation`] for an operation on a path that did not
//!   complete.
//! - [`implementations`]: The [`FilesystemRepositoryBuilder`], the constants that fix the file
//!   layout, such as [`METADATA_SUFFIX`], and the port implementations.
//! - [`shared`]: The value types that more than one module uses, such as the [`DocumentKey`] and
//!   the [`DocumentMetadata`].
//! - [`traits`]: The access markers, the traits that connect them to the ports, and the
//!   [`Document`] trait every stored document kind implements.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::error::BackendError;

pub mod error;
pub mod implementations;
pub mod shared;
pub mod traits;

pub use error::{FailedFilesystemOperation, FailedFilesystemOperationReason, FilesystemOperation};
pub use implementations::builder::{FilesystemRepositoryBuilder, NoAccess, NoRoot};
pub use implementations::constants::METADATA_SUFFIX;
pub use shared::document_key::{DocumentKey, InvalidDocumentKey};
pub use shared::document_metadata::{
    DocumentMetadata, DocumentMetadataBuilder, InputDocument, METADATA_VERSION, NoDocumentType,
    Producer,
};
pub use shared::sha256::{Sha256Digest, sha256_hex};
pub use traits::access::{Access, CanRead, CanWrite, Read, ReadWrite, Write};
pub use traits::document::Document;

/// Stores documents of one [`Document`] type `D` as files beneath one root directory, with the
/// access the marker `A` allows.
///
/// [`FilesystemRepository::builder`] starts the builder that builds it. `A` is [`Read`], [`Write`],
/// or [`ReadWrite`]. A read rejects a metadata file of another kind or kind version than `D`, and a
/// write rejects a record whose metadata holds another kind or kind version. Repositories for
/// different document types can share one root, as long as their keys differ.
///
/// The repository implements `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash`
/// with no bound on `D`, so a `State` context can hold it for every document type. Two repositories
/// compare by root and access.
///
/// A [`DocumentKey`] is a document's path under the root. A document lives at the root joined with
/// its key. On Linux and macOS, a root of `/data/arkad` and a key of `sec/CIK0000320193.json` name
/// `/data/arkad/sec/CIK0000320193.json`. On Windows, the same key under a root of `C:\data\arkad`
/// names `C:\data\arkad\sec\CIK0000320193.json`. The join uses the separator of the platform the
/// store runs on. [`FilesystemRepository::document_path`] performs that join.
///
/// Each document has a metadata file beside it. The metadata file name is the document's file name
/// with [`METADATA_SUFFIX`] appended, so `sec/CIK0000320193.json` keeps its metadata in
/// `sec/CIK0000320193.json<METADATA_SUFFIX>`. A key that ends in [`METADATA_SUFFIX`] names the
/// metadata file of another key. `get` and `persist` reject such a key with
/// [`FailedFilesystemOperationReason::InvalidKeySuffix`] before they read or write a file.
///
/// The metadata names the size and the SHA-256 digest of its document. `get` and `persist` check
/// both. If a new record differs from the stored pair only in valid `produced_at` and
/// `execution_id` values, `persist` writes no file, so `produced_at` keeps naming the run that
/// produced the bytes.
///
/// A write first writes both files to synced temporary files in the same directory. It then
/// renames the metadata over its target, and the document over its target. A reader sees the old
/// file or the new file, never a partial one. A document on disk always has its metadata. Between
/// the two renames, a reader can see new metadata beside the old document. If the document rename
/// fails, or the caller drops the write between the renames, the new metadata stays beside the old
/// document until the next successful write under that key. If the caller drops the write before
/// the renames, its temporary files stay in the directory.
// The value traits below are written by hand. A derive would require `D` to implement each of
// them, although the repository holds no `D`.
pub struct FilesystemRepository<A, D> {
    root: PathBuf,
    access: A,
    // `fn() -> D` instead of `D`: the repository holds no `D`, so it stays `Send`, `Sync`, and
    // `Unpin` for every `D`.
    document_type: PhantomData<fn() -> D>,
}

impl<A: fmt::Debug, D> fmt::Debug for FilesystemRepository<A, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilesystemRepository")
            .field("root", &self.root)
            .field("access", &self.access)
            .field("document_type", &self.document_type)
            .finish()
    }
}

impl<A: Clone, D> Clone for FilesystemRepository<A, D> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            access: self.access.clone(),
            document_type: PhantomData,
        }
    }
}

impl<A: PartialEq, D> PartialEq for FilesystemRepository<A, D> {
    fn eq(&self, other: &Self) -> bool {
        (&self.root, &self.access) == (&other.root, &other.access)
    }
}

impl<A: Eq, D> Eq for FilesystemRepository<A, D> {}

impl<A: PartialOrd, D> PartialOrd for FilesystemRepository<A, D> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (&self.root, &self.access).partial_cmp(&(&other.root, &other.access))
    }
}

impl<A: Ord, D> Ord for FilesystemRepository<A, D> {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.root, &self.access).cmp(&(&other.root, &other.access))
    }
}

impl<A: Hash, D> Hash for FilesystemRepository<A, D> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.root.hash(state);
        self.access.hash(state);
    }
}

impl FilesystemRepository<NoAccess, NoDocumentType> {
    /// Starts a [`FilesystemRepositoryBuilder`], which is the only way to build a
    /// [`FilesystemRepository`].
    ///
    /// The impl uses `NoAccess` and `NoDocumentType` so that a call needs no type annotation. No
    /// built repository has either.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    ///
    /// use storage::implementations::repository::filesystem::{DocumentKey, FilesystemRepository};
    /// # use std::fmt;
    /// #
    /// # use serde::{Deserialize, Serialize};
    /// # use storage::implementations::repository::filesystem::{Document, DocumentMetadata};
    /// #
    /// # #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    /// # enum ReportKind {
    /// #     Summary,
    /// # }
    /// #
    /// # impl fmt::Display for ReportKind {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "summary")
    /// #     }
    /// # }
    /// #
    /// # #[derive(Serialize, Deserialize)]
    /// # struct SummaryContext;
    /// #
    /// # struct Summary {
    /// #     key: DocumentKey,
    /// #     bytes: Vec<u8>,
    /// #     metadata: DocumentMetadata<SummaryContext>,
    /// # }
    /// #
    /// # impl Document for Summary {
    /// #     type Kind = ReportKind;
    /// #     const KIND: ReportKind = ReportKind::Summary;
    /// #     const KIND_VERSION: u32 = 1;
    /// #     type Context = SummaryContext;
    /// #     fn key(&self) -> &DocumentKey { &self.key }
    /// #     fn bytes(&self) -> &[u8] { &self.bytes }
    /// #     fn metadata(&self) -> &DocumentMetadata<SummaryContext> { &self.metadata }
    /// #     fn from_parts(
    /// #         key: DocumentKey,
    /// #         bytes: Vec<u8>,
    /// #         metadata: DocumentMetadata<SummaryContext>,
    /// #     ) -> Self {
    /// #         Self { key, bytes, metadata }
    /// #     }
    /// # }
    /// #
    /// let repository = FilesystemRepository::builder()
    ///     .reader()
    ///     .document_type::<Summary>()
    ///     .root("/data/arkad")
    ///     .build();
    ///
    /// let expected_result = Path::new("/data/arkad");
    ///
    /// let result = repository.root();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn builder() -> FilesystemRepositoryBuilder<NoAccess, NoRoot, NoDocumentType> {
        FilesystemRepositoryBuilder::new()
    }
}

impl<A, D> FilesystemRepository<A, D> {
    /// Returns the root directory every key resolves under.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the path on disk of the document a key names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    ///
    /// use storage::implementations::repository::filesystem::{DocumentKey, FilesystemRepository};
    /// # use std::fmt;
    /// #
    /// # use serde::{Deserialize, Serialize};
    /// # use storage::implementations::repository::filesystem::{Document, DocumentMetadata};
    /// #
    /// # #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    /// # enum ReportKind {
    /// #     Summary,
    /// # }
    /// #
    /// # impl fmt::Display for ReportKind {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "summary")
    /// #     }
    /// # }
    /// #
    /// # #[derive(Serialize, Deserialize)]
    /// # struct SummaryContext;
    /// #
    /// # struct Summary {
    /// #     key: DocumentKey,
    /// #     bytes: Vec<u8>,
    /// #     metadata: DocumentMetadata<SummaryContext>,
    /// # }
    /// #
    /// # impl Document for Summary {
    /// #     type Kind = ReportKind;
    /// #     const KIND: ReportKind = ReportKind::Summary;
    /// #     const KIND_VERSION: u32 = 1;
    /// #     type Context = SummaryContext;
    /// #     fn key(&self) -> &DocumentKey { &self.key }
    /// #     fn bytes(&self) -> &[u8] { &self.bytes }
    /// #     fn metadata(&self) -> &DocumentMetadata<SummaryContext> { &self.metadata }
    /// #     fn from_parts(
    /// #         key: DocumentKey,
    /// #         bytes: Vec<u8>,
    /// #         metadata: DocumentMetadata<SummaryContext>,
    /// #     ) -> Self {
    /// #         Self { key, bytes, metadata }
    /// #     }
    /// # }
    /// #
    /// let repository = FilesystemRepository::builder()
    ///     .reader()
    ///     .document_type::<Summary>()
    ///     .root("/data/arkad")
    ///     .build();
    /// let key = DocumentKey::new("sec/CIK0000320193.json")
    ///     .expect("Given a valid relative path, the key should always build");
    ///
    /// let expected_result = Path::new("/data/arkad/sec/CIK0000320193.json");
    ///
    /// let result = repository.document_path(&key);
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn document_path(&self, key: &DocumentKey) -> PathBuf {
        self.root.join(key.as_path())
    }

    /// Returns the path on disk of the metadata file beside a key's document.
    fn metadata_path(&self, key: &DocumentKey) -> PathBuf {
        let mut path = self.document_path(key).into_os_string();
        path.push(METADATA_SUFFIX);
        PathBuf::from(path)
    }

    /// Checks that the key does not end in [`METADATA_SUFFIX`].
    ///
    /// # Errors
    ///
    /// Returns a [`FailedFilesystemOperation`] with
    /// [`FailedFilesystemOperationReason::InvalidKeySuffix`] if the key ends in [`METADATA_SUFFIX`].
    fn check_key(&self, key: &DocumentKey) -> Result<(), FailedFilesystemOperation> {
        let key_bytes = key.as_path().as_os_str().as_encoded_bytes();
        if key_bytes.ends_with(METADATA_SUFFIX.as_bytes()) {
            return Err(FailedFilesystemOperation::new(
                FailedFilesystemOperationReason::InvalidKeySuffix,
                self.document_path(key),
            ));
        }
        Ok(())
    }
}

/// Checks that the bytes have the size and the SHA-256 digest the metadata names.
///
/// # Errors
///
/// Returns [`FailedFilesystemOperationReason::UnexpectedDocumentSize`] if the size differs, or
/// [`FailedFilesystemOperationReason::UnexpectedDocumentDigest`] if the digest differs.
fn check_content<C>(
    bytes: &[u8],
    metadata: &DocumentMetadata<C>,
) -> Result<(), FailedFilesystemOperationReason> {
    // A `usize` always fits in a `u64` on the platforms Rust supports.
    let found_size = bytes.len() as u64;
    if found_size != metadata.size_in_bytes() {
        return Err(FailedFilesystemOperationReason::UnexpectedDocumentSize {
            found_size,
            expected_size: metadata.size_in_bytes(),
        });
    }
    let found_digest = sha256_hex(bytes);
    if found_digest != metadata.sha256() {
        return Err(FailedFilesystemOperationReason::UnexpectedDocumentDigest {
            found_digest,
            expected_digest: metadata.sha256().to_owned(),
        });
    }
    Ok(())
}

// `A: Access` makes `A` `Sync`, so the future of `ensure_root_is_reachable` is `Send`. Without the
// bound, clippy's `future_not_send` lint rejects the method. `PhantomData<fn() -> D>` is `Sync` for
// every `D`, so `D` needs no bound.
impl<A: Access, D> FilesystemRepository<A, D> {
    /// Checks that the root exists and is a directory.
    ///
    /// # Errors
    ///
    /// Returns a [`FailedFilesystemOperation`] with:
    /// - [`FailedFilesystemOperationReason::MissingRoot`] if the root does not exist, or a file
    ///   stands in its path.
    /// - [`FailedFilesystemOperationReason::RootIsNotADirectory`] if the root is a file.
    /// - [`FailedFilesystemOperationReason::FailedIo`] if the filesystem cannot inspect the root.
    async fn ensure_root_is_reachable(&self) -> Result<(), FailedFilesystemOperation> {
        match tokio::fs::metadata(&self.root).await {
            Ok(metadata) if metadata.is_dir() => Ok(()),
            Ok(_) => Err(FailedFilesystemOperation::new(
                FailedFilesystemOperationReason::RootIsNotADirectory,
                &self.root,
            )),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) =>
            {
                Err(FailedFilesystemOperation::new(
                    FailedFilesystemOperationReason::MissingRoot,
                    &self.root,
                ))
            }
            Err(error) => Err(FailedFilesystemOperation::from_io(
                FilesystemOperation::Inspect,
                &self.root,
                &error,
            )),
        }
    }
}

impl From<FailedFilesystemOperation> for BackendError {
    /// Converts a [`FailedFilesystemOperation`] into the [`BackendError`] variant for its reason.
    ///
    /// A missing root, a root that is not a directory, and a path that vanished become
    /// [`BackendError::UnreachableStorage`]. A denied permission becomes
    /// [`BackendError::UnauthorizedAccess`]. Every other reason becomes
    /// [`BackendError::FailedOperation`]. The [`FailedFilesystemOperation`] message becomes the
    /// reason.
    fn from(error: FailedFilesystemOperation) -> Self {
        let reason = error.to_string();
        match error.reason {
            FailedFilesystemOperationReason::MissingRoot
            | FailedFilesystemOperationReason::RootIsNotADirectory
            | FailedFilesystemOperationReason::FailedIo {
                kind: io::ErrorKind::NotFound,
                ..
            } => Self::unreachable_storage(reason),
            FailedFilesystemOperationReason::FailedIo {
                kind: io::ErrorKind::PermissionDenied,
                ..
            } => Self::unauthorized_access(reason),
            FailedFilesystemOperationReason::MissingMetadataFile
            | FailedFilesystemOperationReason::InvalidMetadata { .. }
            | FailedFilesystemOperationReason::UnexpectedMetadataKind { .. }
            | FailedFilesystemOperationReason::UnserializableMetadata
            | FailedFilesystemOperationReason::UnexpectedDocumentSize { .. }
            | FailedFilesystemOperationReason::UnexpectedDocumentDigest { .. }
            | FailedFilesystemOperationReason::InvalidKeySuffix
            | FailedFilesystemOperationReason::FailedIo { .. } => Self::failed_operation(reason),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::{assert_eq, assert_ne};
    use tempfile::TempDir;

    use super::*;
    use crate::tests::fixtures::sample_document::{SampleDocument, SampleDocumentKind};
    use crate::traits::repository::ReadWriteRepository;

    /// Builds a repository with read and write access under the given root.
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

    /// A document type that derives no trait, so a test can show that the repository needs none.
    struct BareDocument {
        key: DocumentKey,
        bytes: Vec<u8>,
        metadata: DocumentMetadata<()>,
    }

    impl Document for BareDocument {
        type Kind = SampleDocumentKind;
        const KIND: SampleDocumentKind = SampleDocumentKind::SampleDocument;
        const KIND_VERSION: u32 = 1;
        type Context = ();

        fn key(&self) -> &DocumentKey {
            &self.key
        }

        fn bytes(&self) -> &[u8] {
            &self.bytes
        }

        fn metadata(&self) -> &DocumentMetadata<()> {
            &self.metadata
        }

        fn from_parts(key: DocumentKey, bytes: Vec<u8>, metadata: DocumentMetadata<()>) -> Self {
            Self {
                key,
                bytes,
                metadata,
            }
        }
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_when_the_document_type_derives_nothing() {
        implements_value_traits::<FilesystemRepository<ReadWrite, BareDocument>>();
    }

    #[test]
    fn should_order_repositories_by_root_when_the_roots_differ() {
        let expected_result = Ordering::Less;

        let result = repository_at("/data/a").cmp(&repository_at("/data/b"));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_compare_unequal_when_two_repositories_have_different_roots() {
        let expected_result = repository_at("/data/other");

        let result = repository_at("/data/arkad");

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_compare_equal_when_two_repositories_share_the_root_and_the_access() {
        let expected_result = repository_at("/data/arkad");

        let result = repository_at("/data/arkad");

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filesystem_repository() {
        implements_auto_traits::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_filesystem_repository() {
        implements_send::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    #[test]
    const fn should_implement_sync_for_filesystem_repository() {
        implements_sync::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_filesystem_repository() {
        implements_debug::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_filesystem_repository() {
        implements_clone::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_filesystem_repository() {
        implements_hash::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_filesystem_repository() {
        implements_eq::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_filesystem_repository() {
        implements_ord::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_filesystem_repository() {
        implements_sized::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_filesystem_repository() {
        implements_partial_eq::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_filesystem_repository() {
        implements_partial_ord::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_filesystem_repository() {
        implements_unpin::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    const fn implements_read_write_repository<T: ReadWriteRepository>() {}
    #[test]
    const fn should_implement_read_write_repository_for_filesystem_repository() {
        implements_read_write_repository::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    #[test]
    fn should_hold_the_root_when_building_a_repository() {
        let root = "/data/arkad";

        let expected_result = Path::new("/data/arkad");

        let repository = repository_at(root);
        let result = repository.root();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_join_the_root_and_the_key_when_building_a_document_path() {
        let repository = repository_at("/data/arkad");
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        let expected_result = Path::new("/data/arkad/sec/companyfacts/CIK0000320193.json");

        let result = repository.document_path(&key);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_append_the_metadata_suffix_for_a_document_key() {
        let repository = repository_at("/data/arkad");
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        let expected_result =
            Path::new("/data/arkad/sec/companyfacts/CIK0000320193.json.meta.json");

        let result = repository.metadata_path(&key);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unreachable_storage_when_converting_a_missing_root() {
        let error = FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::MissingRoot,
            "/data/arkad",
        );

        let expected_result = BackendError::unreachable_storage(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Root directory does not exist', Path: '/data/arkad'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unreachable_storage_when_converting_a_root_that_is_not_a_directory() {
        let error = FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::RootIsNotADirectory,
            "/data/arkad",
        );

        let expected_result = BackendError::unreachable_storage(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Root is not a directory', Path: '/data/arkad'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unreachable_storage_when_converting_a_path_that_vanished() {
        let io_error = io::Error::from(io::ErrorKind::NotFound);
        let error = FailedFilesystemOperation::from_io(
            FilesystemOperation::CreateFile,
            "/data/arkad/x.json",
            &io_error,
        );

        let expected_result = BackendError::unreachable_storage(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Failed to create file, entity not found', Path: '/data/arkad/x.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unauthorized_access_when_converting_a_denied_permission() {
        let io_error = io::Error::from(io::ErrorKind::PermissionDenied);
        let error = FailedFilesystemOperation::from_io(
            FilesystemOperation::Read,
            "/data/arkad/x.json",
            &io_error,
        );

        let expected_result = BackendError::unauthorized_access(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Failed to read, permission denied', Path: '/data/arkad/x.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_operation_when_converting_a_full_disk() {
        let io_error = io::Error::from(io::ErrorKind::StorageFull);
        let error = FailedFilesystemOperation::from_io(
            FilesystemOperation::Write,
            "/data/arkad/x.json",
            &io_error,
        );

        let expected_result = BackendError::failed_operation(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Failed to write, no storage space', Path: '/data/arkad/x.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_operation_when_converting_a_missing_metadata_file() {
        let error = FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::MissingMetadataFile,
            "/data/arkad/x.json.meta.json",
        );

        let expected_result = BackendError::failed_operation(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Metadata file does not exist', Path: '/data/arkad/x.json.meta.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_operation_when_converting_an_invalid_key_suffix() {
        let error = FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::InvalidKeySuffix,
            "/data/arkad/x.json.meta.json",
        );

        let expected_result = BackendError::failed_operation(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Key ends in '.meta.json', the suffix of a metadata file', \
            Path: '/data/arkad/x.json.meta.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_the_key_when_the_metadata_suffix_is_not_at_its_end() {
        let repository = repository_at("/data/arkad");
        let key = DocumentKey::new("sec/x.meta.jsonl")
            .expect("Given a valid relative path, the key should always build");

        let expected_result = Ok(());

        let result = repository.check_key(&key);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_accept_the_root_when_the_root_is_an_existing_directory() {
        let root = temporary_root();
        let repository = repository_at(root.path());

        let expected_result = Ok(());

        let result = repository.ensure_root_is_reachable().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_root_when_the_root_does_not_exist() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let repository = repository_at(&root);

        let expected_result = Err(FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::MissingRoot,
            &root,
        ));

        let result = repository.ensure_root_is_reachable().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_root_when_the_root_has_a_file_in_its_path() {
        let parent = temporary_root();
        let file = parent.path().join("file");
        std::fs::write(&file, b"").expect(
            "Given a writable temp directory, writing a file into it should always succeed",
        );
        let root = file.join("data");
        let repository = repository_at(&root);

        let expected_result = Err(FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::MissingRoot,
            &root,
        ));

        let result = repository.ensure_root_is_reachable().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_root_is_not_a_directory_when_the_root_is_a_file() {
        let parent = temporary_root();
        let root = parent.path().join("file");
        std::fs::write(&root, b"").expect(
            "Given a writable temp directory, writing a file into it should always succeed",
        );
        let repository = repository_at(&root);

        let expected_result = Err(FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::RootIsNotADirectory,
            &root,
        ));

        let result = repository.ensure_root_is_reachable().await;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_operation_when_converting_an_unexpected_metadata_kind() {
        let error = FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::UnexpectedMetadataKind {
                found_kind: "other_document".to_owned(),
                found_kind_version: 1,
                expected_kind: "sample_document".to_owned(),
                expected_kind_version: 1,
            },
            "/data/arkad/x.json.meta.json",
        );

        let expected_result = BackendError::failed_operation(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Metadata holds kind 'other_document' version 1, expected kind \
            'sample_document' version 1', Path: '/data/arkad/x.json.meta.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }
}
