//! # Access
//!
//! Provides the access markers [`Read`], [`Write`], and [`ReadWrite`], which set what a
//! [`FilesystemRepository`](super::FilesystemRepository) can do.
//!
//! A repository implements only the ports its marker allows. Code that holds a read handle has no
//! `persist` method, so a write through it does not compile. [`CanRead`] and [`CanWrite`] connect
//! the markers to the ports, and only the markers in this module implement them.

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
/// use storage::implementations::repository::filesystem::FilesystemRepositoryBuilder;
/// # use storage::implementations::repository::filesystem::RawDocument;
///
/// async fn write_through_a_reader(document: RawDocument) {
///     let reader = FilesystemRepositoryBuilder::new()
///         .reader()
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
/// use storage::implementations::repository::filesystem::{DocumentKey, FilesystemRepositoryBuilder};
///
/// async fn read_through_a_writer(key: DocumentKey) {
///     let writer = FilesystemRepositoryBuilder::new()
///         .writer()
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

/// Defines the access of a [`FilesystemRepository`](super::FilesystemRepository).
///
/// Only [`Read`], [`Write`], and [`ReadWrite`] implement it. A builder can build a repository only
/// after it holds one of them.
pub trait Access: sealed::Sealed + Send + Sync {}

/// Defines an access that allows reads.
#[diagnostic::on_unimplemented(
    message = "a repository with `{Self}` access cannot read documents",
    note = "build the repository with `.reader()` or `.read_write()` to read through it"
)]
pub trait CanRead: Access {}

/// Defines an access that allows writes.
#[diagnostic::on_unimplemented(
    message = "a repository with `{Self}` access cannot write documents",
    note = "build the repository with `.writer()` or `.read_write()` to write through it"
)]
pub trait CanWrite: Access {}

impl sealed::Sealed for Read {}
impl sealed::Sealed for Write {}
impl sealed::Sealed for ReadWrite {}

impl Access for Read {}
impl Access for Write {}
impl Access for ReadWrite {}

impl CanRead for Read {}
impl CanRead for ReadWrite {}

impl CanWrite for Write {}
impl CanWrite for ReadWrite {}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

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
