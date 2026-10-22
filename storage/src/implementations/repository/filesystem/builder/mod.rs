//! # Builder
//!
//! Provides [`FilesystemRepositoryBuilder`], which builds a
//! [`FilesystemRepository`] with a chosen access, document type, and root.
//!
//! Each step changes the builder's type. [`build`](FilesystemRepositoryBuilder::build) exists only
//! once the builder holds an access marker, a document type, and a root, so a repository without
//! one of them does not compile. The access marker decides which ports the built repository
//! implements, and the document type decides which records it reads and writes.

use std::marker::PhantomData;
use std::path::PathBuf;

use super::FilesystemRepository;
use super::access::{Access, Read, ReadWrite, Write};
use crate::traits::document::Document;

/// Marks a builder that holds no access yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoAccess;

/// Marks a builder that holds no root yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoRoot;

/// Marks a builder that holds no document type yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoDocumentType;

/// Builds a [`FilesystemRepository`] with a chosen access, document type, and root.
///
/// Set the access with [`reader`](Self::reader), [`writer`](Self::writer), or
/// [`read_write`](Self::read_write), the document type with [`document_type`](Self::document_type),
/// and the root with [`root`](Self::root), in any order. Then call [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilesystemRepositoryBuilder<A, R, D> {
    access: A,
    root: R,
    document_type: PhantomData<fn() -> D>,
}

impl FilesystemRepositoryBuilder<NoAccess, NoRoot, NoDocumentType> {
    /// Creates a new [`FilesystemRepositoryBuilder`] that holds no access, root, or document type.
    pub(super) const fn new() -> Self {
        Self {
            access: NoAccess,
            root: NoRoot,
            document_type: PhantomData,
        }
    }
}

impl<R, D> FilesystemRepositoryBuilder<NoAccess, R, D> {
    /// Sets the access to [`Read`], so the built repository only reads.
    #[must_use]
    pub fn reader(self) -> FilesystemRepositoryBuilder<Read, R, D> {
        self.with_access(Read)
    }

    /// Sets the access to [`Write`], so the built repository only writes.
    #[must_use]
    pub fn writer(self) -> FilesystemRepositoryBuilder<Write, R, D> {
        self.with_access(Write)
    }

    /// Sets the access to [`ReadWrite`], so the built repository reads and writes.
    #[must_use]
    pub fn read_write(self) -> FilesystemRepositoryBuilder<ReadWrite, R, D> {
        self.with_access(ReadWrite)
    }

    /// Replaces the missing access with the given marker.
    fn with_access<A: Access>(self, access: A) -> FilesystemRepositoryBuilder<A, R, D> {
        FilesystemRepositoryBuilder {
            access,
            root: self.root,
            document_type: PhantomData,
        }
    }
}

impl<A, R> FilesystemRepositoryBuilder<A, R, NoDocumentType> {
    /// Sets the [`Document`] type that the built repository reads and writes.
    #[must_use]
    pub fn document_type<D: Document>(self) -> FilesystemRepositoryBuilder<A, R, D> {
        FilesystemRepositoryBuilder {
            access: self.access,
            root: self.root,
            document_type: PhantomData,
        }
    }
}

impl<A, D> FilesystemRepositoryBuilder<A, NoRoot, D> {
    /// Sets the root directory that every key of the built repository resolves under.
    ///
    /// The builder checks nothing and touches no disk. Whether the root exists, is a directory,
    /// and allows reads or writes can change at any time. It can also differ for each document
    /// under the root. Each operation reports those failures for the document it touches.
    ///
    /// Pass an absolute root. A relative root resolves against the process's working directory,
    /// so the store moves with it.
    #[must_use]
    pub fn root(self, root: impl Into<PathBuf>) -> FilesystemRepositoryBuilder<A, PathBuf, D> {
        FilesystemRepositoryBuilder {
            access: self.access,
            root: root.into(),
            document_type: self.document_type,
        }
    }
}

impl<A: Access, D: Document> FilesystemRepositoryBuilder<A, PathBuf, D> {
    /// Builds a [`FilesystemRepository`] with the access, the root, and the document type this
    /// builder holds.
    ///
    /// # Examples
    ///
    /// A builder without a root has no `build` method, so this does not compile:
    ///
    /// ```compile_fail,E0599
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
    /// # use serde::{Deserialize, Serialize};
    /// # use storage::{Document, DocumentKey, DocumentMetadata, MetadataContext};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct FetchContext;
    /// # impl MetadataContext for FetchContext { const KIND: &'static str = "fetch.v1"; }
    /// # struct Fetch { key: DocumentKey, bytes: Vec<u8>, metadata: DocumentMetadata<FetchContext> }
    /// # impl Document for Fetch {
    /// #     type Context = FetchContext;
    /// #     fn key(&self) -> &DocumentKey { &self.key }
    /// #     fn bytes(&self) -> &[u8] { &self.bytes }
    /// #     fn metadata(&self) -> &DocumentMetadata<FetchContext> { &self.metadata }
    /// #     fn from_parts(key: DocumentKey, bytes: Vec<u8>, metadata: DocumentMetadata<FetchContext>) -> Self {
    /// #         Self { key, bytes, metadata }
    /// #     }
    /// # }
    /// let repository = FilesystemRepository::builder()
    ///     .reader()
    ///     .document_type::<Fetch>()
    ///     .build();
    /// ```
    ///
    /// A builder without an access has no `build` method either:
    ///
    /// ```compile_fail,E0599
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
    /// # use serde::{Deserialize, Serialize};
    /// # use storage::{Document, DocumentKey, DocumentMetadata, MetadataContext};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct FetchContext;
    /// # impl MetadataContext for FetchContext { const KIND: &'static str = "fetch.v1"; }
    /// # struct Fetch { key: DocumentKey, bytes: Vec<u8>, metadata: DocumentMetadata<FetchContext> }
    /// # impl Document for Fetch {
    /// #     type Context = FetchContext;
    /// #     fn key(&self) -> &DocumentKey { &self.key }
    /// #     fn bytes(&self) -> &[u8] { &self.bytes }
    /// #     fn metadata(&self) -> &DocumentMetadata<FetchContext> { &self.metadata }
    /// #     fn from_parts(key: DocumentKey, bytes: Vec<u8>, metadata: DocumentMetadata<FetchContext>) -> Self {
    /// #         Self { key, bytes, metadata }
    /// #     }
    /// # }
    /// let repository = FilesystemRepository::builder()
    ///     .document_type::<Fetch>()
    ///     .root("/data/arkad")
    ///     .build();
    /// ```
    ///
    /// A builder without a document type has no `build` method either:
    ///
    /// ```compile_fail,E0599
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
    ///
    /// let repository = FilesystemRepository::builder()
    ///     .reader()
    ///     .root("/data/arkad")
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> FilesystemRepository<A, D> {
        FilesystemRepository {
            root: self.root,
            access: self.access,
            document_type: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::sample_document::SampleDocument;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filesystem_repository_builder() {
        implements_auto_traits::<FilesystemRepositoryBuilder<NoAccess, NoRoot, NoDocumentType>>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_filesystem_repository_builder() {
        implements_value_traits::<FilesystemRepositoryBuilder<NoAccess, NoRoot, NoDocumentType>>();
    }

    #[test]
    const fn should_implement_value_traits_for_no_access() {
        implements_value_traits::<NoAccess>();
    }

    #[test]
    const fn should_implement_value_traits_for_no_root() {
        implements_value_traits::<NoRoot>();
    }

    #[test]
    const fn should_implement_value_traits_for_no_document_type() {
        implements_value_traits::<NoDocumentType>();
    }

    #[test]
    fn should_hold_no_access_root_or_document_type_when_starting_a_builder() {
        let expected_result = FilesystemRepositoryBuilder {
            access: NoAccess,
            root: NoRoot,
            document_type: PhantomData,
        };

        let result = FilesystemRepository::builder();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_read_access_when_choosing_reader() {
        let builder = FilesystemRepository::builder().reader();

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Read,
            document_type: PhantomData,
        };

        let result = builder
            .document_type::<SampleDocument>()
            .root("/data/arkad")
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_write_access_when_choosing_writer() {
        let builder = FilesystemRepository::builder().writer();

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Write,
            document_type: PhantomData,
        };

        let result = builder
            .document_type::<SampleDocument>()
            .root("/data/arkad")
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_read_write_access_when_choosing_read_write() {
        let builder = FilesystemRepository::builder().read_write();

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: ReadWrite,
            document_type: PhantomData,
        };

        let result = builder
            .document_type::<SampleDocument>()
            .root("/data/arkad")
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_read_access_when_setting_the_root_before_the_access() {
        let builder = FilesystemRepository::builder().root("/data/arkad");

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Read,
            document_type: PhantomData,
        };

        let result = builder.reader().document_type::<SampleDocument>().build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_write_access_when_setting_the_root_before_the_access() {
        let builder = FilesystemRepository::builder().root("/data/arkad");

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Write,
            document_type: PhantomData,
        };

        let result = builder.writer().document_type::<SampleDocument>().build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_read_write_access_when_setting_the_root_before_the_access() {
        let builder = FilesystemRepository::builder().root("/data/arkad");

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: ReadWrite,
            document_type: PhantomData,
        };

        let result = builder
            .read_write()
            .document_type::<SampleDocument>()
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_when_setting_the_document_type_before_the_access_and_root() {
        let builder = FilesystemRepository::builder().document_type::<SampleDocument>();

        let expected_result: FilesystemRepository<Read, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Read,
            document_type: PhantomData,
        };

        let result = builder.root("/data/arkad").reader().build();

        assert_eq!(result, expected_result);
    }
}
