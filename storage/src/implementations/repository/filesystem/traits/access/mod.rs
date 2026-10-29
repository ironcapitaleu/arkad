//! # Access
//!
//! Provides the access markers [`Read`], [`Write`], and [`ReadWrite`], which set what a
//! [`FilesystemRepository`] can do.
//!
//! A repository implements only the ports its marker allows. Code that holds a read handle has no
//! `persist` method, so a write through it does not compile. [`CanRead`] and [`CanWrite`] connect
//! the markers to the ports, and only the markers in this module implement them.
//!
//! [`FilesystemRepository`]: crate::implementations::repository::filesystem::FilesystemRepository

/// Keeps [`Access`] closed, so no type outside this module becomes an access marker.
mod sealed {
    pub trait Sealed {}
}

/// Marks a repository that reads documents and cannot write them.
///
/// # Examples
///
/// A read handle has no `persist` method, so this does not compile:
///
/// ```compile_fail,E0599
/// use storage::WriteRepository;
/// use storage::implementations::repository::filesystem::FilesystemRepository;
/// # use std::fmt;
/// #
/// # use serde::{Deserialize, Serialize};
/// # use storage::implementations::repository::filesystem::{Document, DocumentKey, DocumentMetadata};
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// # enum ReportType {
/// #     Summary,
/// # }
/// #
/// # impl fmt::Display for ReportType {
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
/// #     type DocumentType = ReportType;
/// #     type IngestionMode = &'static str;
/// #     const DOCUMENT_TYPE: ReportType = ReportType::Summary;
/// #     const DOCUMENT_TYPE_VERSION: u32 = 1;
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
/// async fn write_through_a_reader(document: Summary) {
///     let reader = FilesystemRepository::builder()
///         .reader()
///         .document_type::<Summary>()
///         .root("/data/arkad")
///         .build();
///
///     let _ = reader.persist(document).await;
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Read;

/// Marks a repository that writes documents and cannot read them.
///
/// # Examples
///
/// A write handle has no `get` method, so this does not compile:
///
/// ```compile_fail,E0599
/// use storage::ReadRepository;
/// use storage::implementations::repository::filesystem::FilesystemRepository;
/// # use std::fmt;
/// #
/// # use serde::{Deserialize, Serialize};
/// # use storage::implementations::repository::filesystem::{Document, DocumentKey, DocumentMetadata};
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// # enum ReportType {
/// #     Summary,
/// # }
/// #
/// # impl fmt::Display for ReportType {
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
/// #     type DocumentType = ReportType;
/// #     type IngestionMode = &'static str;
/// #     const DOCUMENT_TYPE: ReportType = ReportType::Summary;
/// #     const DOCUMENT_TYPE_VERSION: u32 = 1;
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
/// async fn read_through_a_writer(key: DocumentKey) {
///     let writer = FilesystemRepository::builder()
///         .writer()
///         .document_type::<Summary>()
///         .root("/data/arkad")
///         .build();
///
///     let _ = writer.get(key).await;
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Write;

/// Marks a repository that reads and writes documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadWrite;

/// Defines the access of a [`FilesystemRepository`].
///
/// Only [`Read`], [`Write`], and [`ReadWrite`] implement it. A builder can build a repository only
/// after it holds one of them.
///
/// [`FilesystemRepository`]: crate::implementations::repository::filesystem::FilesystemRepository
pub trait Access: sealed::Sealed + Send + Sync {
    /// The name of the access, such as `ReadWrite`.
    const NAME: &'static str;
}

/// Defines an access that allows reads, so the repository implements
/// [`ReadRepository`](crate::ReadRepository).
#[diagnostic::on_unimplemented(
    message = "a repository with `{Self}` access cannot read documents",
    note = "build the repository with `.reader()` or `.read_write()` to read through it"
)]
pub trait CanRead: Access {}

/// Defines an access that allows writes, so the repository implements
/// [`WriteRepository`](crate::WriteRepository).
///
/// # Examples
///
/// A read handle passed where a [`WriteRepository`](crate::WriteRepository) is required does
/// not compile. The compiler reports this trait's message:
///
/// ```compile_fail,E0277
/// use storage::WriteRepository;
/// use storage::implementations::repository::filesystem::FilesystemRepository;
/// # use std::fmt;
/// #
/// # use serde::{Deserialize, Serialize};
/// # use storage::implementations::repository::filesystem::{Document, DocumentKey, DocumentMetadata};
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// # enum ReportType {
/// #     Summary,
/// # }
/// #
/// # impl fmt::Display for ReportType {
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
/// #     type DocumentType = ReportType;
/// #     type IngestionMode = &'static str;
/// #     const DOCUMENT_TYPE: ReportType = ReportType::Summary;
/// #     const DOCUMENT_TYPE_VERSION: u32 = 1;
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
/// fn needs_a_writer<W: WriteRepository>(_writer: W) {}
///
/// needs_a_writer(
///     FilesystemRepository::builder()
///         .reader()
///         .document_type::<Summary>()
///         .root("/data/arkad")
///         .build(),
/// );
/// ```
#[diagnostic::on_unimplemented(
    message = "a repository with `{Self}` access cannot write documents",
    note = "build the repository with `.writer()` or `.read_write()` to write through it"
)]
pub trait CanWrite: Access {}

impl sealed::Sealed for Read {}
impl sealed::Sealed for Write {}
impl sealed::Sealed for ReadWrite {}

impl Access for Read {
    const NAME: &'static str = "Read";
}

impl Access for Write {
    const NAME: &'static str = "Write";
}

impl Access for ReadWrite {
    const NAME: &'static str = "ReadWrite";
}

impl CanRead for Read {}
impl CanRead for ReadWrite {}

impl CanWrite for Write {}
impl CanWrite for ReadWrite {}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_marker_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash + Send + Sync + Unpin,
    >() {
    }

    #[test]
    const fn should_implement_marker_traits_for_read() {
        implements_marker_traits::<Read>();
    }

    #[test]
    const fn should_implement_marker_traits_for_write() {
        implements_marker_traits::<Write>();
    }

    #[test]
    const fn should_implement_marker_traits_for_read_write() {
        implements_marker_traits::<ReadWrite>();
    }

    #[test]
    fn should_name_read_access_for_read() {
        let expected_result = "Read";

        let result = Read::NAME;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_write_access_for_write() {
        let expected_result = "Write";

        let result = Write::NAME;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_read_write_access_for_read_write() {
        let expected_result = "ReadWrite";

        let result = ReadWrite::NAME;

        assert_eq!(result, expected_result);
    }

    const fn implements_can_read<T: CanRead>() {}
    const fn implements_can_write<T: CanWrite>() {}

    #[test]
    const fn should_allow_reads_for_read() {
        implements_can_read::<Read>();
    }

    #[test]
    const fn should_allow_writes_for_write() {
        implements_can_write::<Write>();
    }

    #[test]
    const fn should_allow_reads_for_read_write() {
        implements_can_read::<ReadWrite>();
    }

    #[test]
    const fn should_allow_writes_for_read_write() {
        implements_can_write::<ReadWrite>();
    }
}
