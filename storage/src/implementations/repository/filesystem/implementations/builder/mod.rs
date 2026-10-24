//! # Builder
//!
//! Provides [`FilesystemRepositoryBuilder`], which builds a [`FilesystemRepository`] with a chosen
//! access, document type, and root.
//!
//! Each step changes the builder's type. [`build`](FilesystemRepositoryBuilder::build) exists only
//! once the builder holds an access marker, a document type, and a root, so a repository without
//! one of them does not compile. The access marker decides which ports the built repository
//! implements, and the document type decides which records it reads and writes.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::path::PathBuf;

use crate::implementations::repository::filesystem::FilesystemRepository;
use crate::implementations::repository::filesystem::shared::document_metadata::NoDocumentType;
use crate::implementations::repository::filesystem::traits::access::{
    Access, Read, ReadWrite, Write,
};
use crate::implementations::repository::filesystem::traits::document::Document;

/// Marks a builder that holds no access yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoAccess;

/// Marks a builder that holds no root yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoRoot;

/// Builds a [`FilesystemRepository`] with a chosen access, document type, and root.
///
/// Set the access with [`reader`](Self::reader), [`writer`](Self::writer), or
/// [`read_write`](Self::read_write), the document type with [`document_type`](Self::document_type),
/// and the root with [`root`](Self::root), in any order. Then call [`build`](Self::build).
// Deviation: the value traits below are written by hand. A derive would require `D` to implement
// each of them, although the builder holds no `D`.
pub struct FilesystemRepositoryBuilder<A, R, D> {
    access: A,
    root: R,
    document_type: PhantomData<fn() -> D>,
}

impl<A: fmt::Debug, R: fmt::Debug, D> fmt::Debug for FilesystemRepositoryBuilder<A, R, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilesystemRepositoryBuilder")
            .field("access", &self.access)
            .field("root", &self.root)
            .field("document_type", &self.document_type)
            .finish()
    }
}

impl<A: Clone, R: Clone, D> Clone for FilesystemRepositoryBuilder<A, R, D> {
    fn clone(&self) -> Self {
        Self {
            access: self.access.clone(),
            root: self.root.clone(),
            document_type: PhantomData,
        }
    }
}

impl<A: PartialEq, R: PartialEq, D> PartialEq for FilesystemRepositoryBuilder<A, R, D> {
    fn eq(&self, other: &Self) -> bool {
        (&self.access, &self.root) == (&other.access, &other.root)
    }
}

impl<A: Eq, R: Eq, D> Eq for FilesystemRepositoryBuilder<A, R, D> {}

impl<A: PartialOrd, R: PartialOrd, D> PartialOrd for FilesystemRepositoryBuilder<A, R, D> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (&self.access, &self.root).partial_cmp(&(&other.access, &other.root))
    }
}

impl<A: Ord, R: Ord, D> Ord for FilesystemRepositoryBuilder<A, R, D> {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.access, &self.root).cmp(&(&other.access, &other.root))
    }
}

impl<A: Hash, R: Hash, D> Hash for FilesystemRepositoryBuilder<A, R, D> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.access.hash(state);
        self.root.hash(state);
    }
}

impl FilesystemRepositoryBuilder<NoAccess, NoRoot, NoDocumentType> {
    /// Creates a new [`FilesystemRepositoryBuilder`] that holds no access, no root, and no document
    /// type.
    pub(crate) const fn new() -> Self {
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
            document_type: self.document_type,
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
    /// Builds a [`FilesystemRepository`] with the access, the document type, and the root this
    /// builder holds.
    ///
    /// # Examples
    ///
    /// A builder without a root has no `build` method, so this does not compile:
    ///
    /// ```compile_fail,E0599
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
    /// # use std::fmt;
    /// #
    /// # use serde::{Deserialize, Serialize};
    /// # use storage::implementations::repository::filesystem::{Document, DocumentKey, DocumentMetadata};
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
    ///
    /// let repository = FilesystemRepository::builder()
    ///     .reader()
    ///     .document_type::<Summary>()
    ///     .build();
    /// ```
    ///
    /// A builder without an access has no `build` method either, so this does not compile:
    ///
    /// ```compile_fail,E0599
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
    /// # use std::fmt;
    /// #
    /// # use serde::{Deserialize, Serialize};
    /// # use storage::implementations::repository::filesystem::{Document, DocumentKey, DocumentMetadata};
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
    ///
    /// let repository = FilesystemRepository::builder()
    ///     .document_type::<Summary>()
    ///     .root("/data/arkad")
    ///     .build();
    /// ```
    ///
    /// A builder without a document type has no `build` method either, so this does not compile:
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
    fn should_hold_no_access_no_root_and_no_document_type_when_starting_a_builder() {
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
        let builder = FilesystemRepository::builder()
            .document_type::<SampleDocument>()
            .reader();

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Read,
            document_type: PhantomData,
        };

        let result = builder.root("/data/arkad").build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_write_access_when_choosing_writer() {
        let builder = FilesystemRepository::builder()
            .document_type::<SampleDocument>()
            .writer();

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Write,
            document_type: PhantomData,
        };

        let result = builder.root("/data/arkad").build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_read_write_access_when_choosing_read_write() {
        let builder = FilesystemRepository::builder()
            .document_type::<SampleDocument>()
            .read_write();

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: ReadWrite,
            document_type: PhantomData,
        };

        let result = builder.root("/data/arkad").build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_read_access_when_setting_the_root_before_the_access() {
        let builder = FilesystemRepository::builder()
            .document_type::<SampleDocument>()
            .root("/data/arkad");

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Read,
            document_type: PhantomData,
        };

        let result = builder.reader().build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_write_access_when_setting_the_root_before_the_access() {
        let builder = FilesystemRepository::builder()
            .document_type::<SampleDocument>()
            .root("/data/arkad");

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Write,
            document_type: PhantomData,
        };

        let result = builder.writer().build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_read_write_access_when_setting_the_root_before_the_access() {
        let builder = FilesystemRepository::builder()
            .document_type::<SampleDocument>()
            .root("/data/arkad");

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: ReadWrite,
            document_type: PhantomData,
        };

        let result = builder.read_write().build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_when_setting_the_document_type_last() {
        let builder = FilesystemRepository::builder().reader().root("/data/arkad");

        let expected_result: FilesystemRepository<_, SampleDocument> = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Read,
            document_type: PhantomData,
        };

        let result = builder.document_type::<SampleDocument>().build();

        assert_eq!(result, expected_result);
    }
}
