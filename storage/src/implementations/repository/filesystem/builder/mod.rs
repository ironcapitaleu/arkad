//! # Builder
//!
//! Provides [`FilesystemRepositoryBuilder`], which builds a
//! [`FilesystemRepository`] with a chosen access and root.
//!
//! Each step changes the builder's type. [`build`](FilesystemRepositoryBuilder::build) exists only
//! once the builder holds an access marker and a root, so a repository without either does not
//! compile. The access marker decides which ports the built repository implements.

use std::path::PathBuf;

use super::FilesystemRepository;
use super::access::{Access, Read, ReadWrite, Write};

/// Marks a builder that holds no access yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoAccess;

/// Marks a builder that holds no root yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoRoot;

/// Builds a [`FilesystemRepository`] with a chosen access and root.
///
/// Set the access with [`reader`](Self::reader), [`writer`](Self::writer), or
/// [`read_write`](Self::read_write), and the root with [`root`](Self::root), in any order. Then call
/// [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilesystemRepositoryBuilder<A, R> {
    access: A,
    root: R,
}

impl FilesystemRepositoryBuilder<NoAccess, NoRoot> {
    /// Creates a new [`FilesystemRepositoryBuilder`] that holds no access and no root.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    ///
    /// use storage::implementations::repository::filesystem::FilesystemRepositoryBuilder;
    ///
    /// let repository = FilesystemRepositoryBuilder::new()
    ///     .reader()
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
    pub const fn new() -> Self {
        Self {
            access: NoAccess,
            root: NoRoot,
        }
    }
}

impl Default for FilesystemRepositoryBuilder<NoAccess, NoRoot> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> FilesystemRepositoryBuilder<NoAccess, R> {
    /// Sets the access to [`Read`], so the built repository only reads.
    #[must_use]
    pub fn reader(self) -> FilesystemRepositoryBuilder<Read, R> {
        self.with_access(Read)
    }

    /// Sets the access to [`Write`], so the built repository only writes.
    #[must_use]
    pub fn writer(self) -> FilesystemRepositoryBuilder<Write, R> {
        self.with_access(Write)
    }

    /// Sets the access to [`ReadWrite`], so the built repository reads and writes.
    #[must_use]
    pub fn read_write(self) -> FilesystemRepositoryBuilder<ReadWrite, R> {
        self.with_access(ReadWrite)
    }

    /// Replaces the missing access with the given marker.
    fn with_access<A: Access>(self, access: A) -> FilesystemRepositoryBuilder<A, R> {
        FilesystemRepositoryBuilder {
            access,
            root: self.root,
        }
    }
}

impl<A> FilesystemRepositoryBuilder<A, NoRoot> {
    /// Sets the root directory that every key of the built repository resolves under.
    ///
    /// The builder checks nothing and touches no disk. Whether the root exists, is a directory,
    /// and allows reads or writes can change at any time. It can also differ for each document
    /// under the root. Each operation reports those failures for the document it touches.
    ///
    /// Pass an absolute root. A relative root resolves against the process's working directory,
    /// so the store moves with it.
    #[must_use]
    pub fn root(self, root: impl Into<PathBuf>) -> FilesystemRepositoryBuilder<A, PathBuf> {
        FilesystemRepositoryBuilder {
            access: self.access,
            root: root.into(),
        }
    }
}

impl<A: Access> FilesystemRepositoryBuilder<A, PathBuf> {
    /// Builds a [`FilesystemRepository`] with the access and the root this builder holds.
    ///
    /// # Examples
    ///
    /// A builder without a root has no `build` method, so this does not compile:
    ///
    /// ```compile_fail,E0599
    /// use storage::implementations::repository::filesystem::FilesystemRepositoryBuilder;
    ///
    /// let repository = FilesystemRepositoryBuilder::new().reader().build();
    /// ```
    ///
    /// A builder without an access has no `build` method either, so this does not compile:
    ///
    /// ```compile_fail,E0599
    /// use storage::implementations::repository::filesystem::FilesystemRepositoryBuilder;
    ///
    /// let repository = FilesystemRepositoryBuilder::new().root("/data/arkad").build();
    /// ```
    #[must_use]
    pub fn build(self) -> FilesystemRepository<A> {
        FilesystemRepository {
            root: self.root,
            access: self.access,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filesystem_repository_builder() {
        implements_auto_traits::<FilesystemRepositoryBuilder<NoAccess, NoRoot>>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_filesystem_repository_builder() {
        implements_value_traits::<FilesystemRepositoryBuilder<NoAccess, NoRoot>>();
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
    fn should_create_the_same_builder_when_using_default_and_new() {
        let expected_result = FilesystemRepositoryBuilder::default();

        let result = FilesystemRepositoryBuilder::new();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_read_access_when_choosing_reader() {
        let builder = FilesystemRepositoryBuilder::new().reader();

        let expected_result = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Read,
        };

        let result = builder.root("/data/arkad").build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_write_access_when_choosing_writer() {
        let builder = FilesystemRepositoryBuilder::new().writer();

        let expected_result = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Write,
        };

        let result = builder.root("/data/arkad").build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_a_repository_with_read_write_access_when_choosing_read_write() {
        let builder = FilesystemRepositoryBuilder::new().read_write();

        let expected_result = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: ReadWrite,
        };

        let result = builder.root("/data/arkad").build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_same_repository_when_setting_the_root_before_the_access() {
        let expected_result = FilesystemRepository {
            root: PathBuf::from("/data/arkad"),
            access: Read,
        };

        let result = FilesystemRepositoryBuilder::new()
            .root("/data/arkad")
            .reader()
            .build();

        assert_eq!(result, expected_result);
    }
}
