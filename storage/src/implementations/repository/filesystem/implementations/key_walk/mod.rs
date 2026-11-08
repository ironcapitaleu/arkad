//! # Key Walk
//!
//! Provides the sorted walk over the directories under a [`KeyPrefix`], which yields each key that
//! has a metadata file.

// The `ListRepository` impl is the only caller outside the tests, and it lands in a later change.
// Until then, the walk is dead code in a build without tests.
#![cfg_attr(not(test), expect(dead_code))]

use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::future::BoxFuture;
use futures_core::stream::{BoxStream, Stream};
use tokio::fs;

use crate::error::{BackendError, ReadError};
use crate::implementations::repository::filesystem::error::{
    FailedFilesystemOperation, FailedFilesystemOperationReason, FilesystemOperation,
};
use crate::implementations::repository::filesystem::implementations::constants::METADATA_SUFFIX;
use crate::implementations::repository::filesystem::shared::document_key::{
    DocumentKey, NON_PORTABLE_CHARACTERS,
};
use crate::implementations::repository::filesystem::shared::key_prefix::KeyPrefix;
use crate::implementations::repository::filesystem::{FilesystemRepository, check_root};

/// One key the walk found: a metadata file, and whether the document file is beside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkedKey {
    /// The key the metadata file belongs to.
    pub key: DocumentKey,
    /// Whether a document file sits beside the metadata file.
    pub has_document: bool,
}

impl<A, D> FilesystemRepository<A, D> {
    /// Walks the directories under the prefix and yields each key that has a metadata file.
    ///
    /// The walk goes depth first and reads one directory at a time. It sorts the names of each
    /// directory by their UTF-8 bytes, and compares a directory name as the name plus `/`. So the
    /// keys come out sorted by the bytes of the key text, each key once. The walk reads no file.
    ///
    /// A key comes from a file whose name ends in [`METADATA_SUFFIX`]. So the walk skips a document
    /// without metadata and a hidden `.tmp` file of a write. It skips a symbolic link under the
    /// prefix, a name that disappears while the walk reads its directory, and a name that no
    /// [`DocumentKey`] can hold, such as a name that is not valid UTF-8. The walk follows a
    /// symbolic link in the path of the prefix itself.
    ///
    /// # Errors
    ///
    /// Yields a [`ReadError::Backend`] item that holds a [`FailedFilesystemOperation`] if:
    /// - the root does not exist or is not a directory. The stream ends after this item.
    /// - the filesystem cannot list the directory the prefix names, or inspect a name in it. The
    ///   stream ends after this item.
    /// - the filesystem cannot list a directory below it, or inspect a name in it. The walk skips
    ///   that directory and continues with the next name of its parent.
    ///
    /// A prefix that names no directory under an existing root yields no item.
    pub(crate) fn walk_keys(
        &self,
        prefix: &KeyPrefix,
    ) -> BoxStream<'static, Result<WalkedKey, ReadError>> {
        let walk = KeyWalk {
            root: self.root().to_path_buf(),
            prefix: prefix.as_path().to_path_buf(),
            open_directories: Vec::new(),
            started: false,
        };
        Box::pin(Unfold {
            step: |walk: KeyWalk| Box::pin(walk.next()),
            pending: Some(Box::pin(walk.next())),
        })
    }
}

/// The state of one walk: the directories on the path to the current one, each with the names it
/// has left.
struct KeyWalk {
    root: PathBuf,
    prefix: PathBuf,
    open_directories: Vec<OpenDirectory>,
    started: bool,
}

/// A directory the walk has read, as its path relative to the root and its sorted names.
struct OpenDirectory {
    relative_path: PathBuf,
    names: std::vec::IntoIter<Name>,
}

/// One name of a directory that the walk visits.
#[derive(Debug, PartialEq, Eq)]
enum Name {
    Directory(String),
    Key { name: String, has_document: bool },
}

impl Name {
    fn text(&self) -> &str {
        match self {
            Self::Directory(name) | Self::Key { name, .. } => name,
        }
    }

    /// The bytes the walk sorts by: a directory name compares as the name plus `/`.
    fn sort_bytes(&self) -> impl Iterator<Item = u8> + '_ {
        let separator: &[u8] = match self {
            Self::Directory(_) => b"/",
            Self::Key { .. } => b"",
        };
        self.text().bytes().chain(separator.iter().copied())
    }
}

impl KeyWalk {
    /// Returns the next item of the walk and the walk that continues after it, or `None` at the
    /// end.
    async fn next(mut self) -> Option<(Result<WalkedKey, ReadError>, Self)> {
        if !self.started {
            self.started = true;
            if let Some(error) = self.open_start_directory().await {
                return Some((Err(read_error(error)), self));
            }
        }
        loop {
            let directory = self.open_directories.last_mut()?;
            let Some(name) = directory.names.next() else {
                self.open_directories.pop();
                continue;
            };
            let relative_path = directory.relative_path.join(name.text());
            match name {
                Name::Directory(_) => {
                    if let Err(error) = self.open(relative_path).await {
                        return Some((Err(read_error(error)), self));
                    }
                }
                Name::Key { has_document, .. } => {
                    let key = DocumentKey::new(relative_path).expect(
                        "Given a path joined from UTF-8 names other than '.' and '..', without '\\' or ':', the key should always build",
                    );
                    return Some((Ok(WalkedKey { key, has_document }), self));
                }
            }
        }
    }

    /// Opens the directory the prefix names, and returns the error that ends the walk, if any.
    ///
    /// Returns no error if the root exists and the prefix names no directory. The walk then has no
    /// open directory, so it ends without an item.
    async fn open_start_directory(&mut self) -> Option<FailedFilesystemOperation> {
        let error = self.open(self.prefix.clone()).await.err()?;
        if names_no_directory(&error) {
            return check_root(&self.root).await.err();
        }
        Some(error)
    }

    /// Reads and sorts the names of a directory, and puts it on top of the open directories.
    async fn open(&mut self, relative_path: PathBuf) -> Result<(), FailedFilesystemOperation> {
        let names = read_sorted_names(self.root.join(&relative_path)).await?;
        self.open_directories.push(OpenDirectory {
            relative_path,
            names: names.into_iter(),
        });
        Ok(())
    }
}

/// Names that refer to a directory, not to a document: the current directory and its parent. No
/// key can hold them.
const DIRECTORY_REFERENCES: [&str; 2] = [".", ".."];

/// Whether the error says that the directory to list does not exist, or that a file stands in its
/// path.
const fn names_no_directory(error: &FailedFilesystemOperation) -> bool {
    matches!(
        error.reason,
        FailedFilesystemOperationReason::FailedIo {
            operation: FilesystemOperation::ListDirectory,
            kind: io::ErrorKind::NotFound | io::ErrorKind::NotADirectory,
        }
    )
}

/// Reads the names of a directory and returns its subdirectories and keys, sorted.
async fn read_sorted_names(directory: PathBuf) -> Result<Vec<Name>, FailedFilesystemOperation> {
    let list_error = |error: io::Error| {
        FailedFilesystemOperation::from_io(FilesystemOperation::ListDirectory, &directory, &error)
    };
    let mut entries = fs::read_dir(&directory).await.map_err(list_error)?;
    let mut directories = Vec::new();
    let mut files = HashSet::new();
    while let Some(entry) = entries.next_entry().await.map_err(list_error)? {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if name.contains(NON_PORTABLE_CHARACTERS) {
            continue;
        }
        let file_type = match entry.file_type().await {
            Ok(file_type) => file_type,
            // The name disappeared after the directory read, for example a `.tmp` file that a
            // write renamed.
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(FailedFilesystemOperation::from_io(
                    FilesystemOperation::Inspect,
                    entry.path(),
                    &error,
                ));
            }
        };
        if file_type.is_dir() {
            directories.push(name);
        } else if file_type.is_file() {
            files.insert(name);
        }
    }
    Ok(sorted_names(directories, &files))
}

/// Pairs each metadata file with its document file, and sorts the keys with the subdirectories.
fn sorted_names(directories: Vec<String>, files: &HashSet<String>) -> Vec<Name> {
    let keys = files.iter().filter_map(|file| {
        let name = file.strip_suffix(METADATA_SUFFIX)?;
        let holds_a_key = !name.is_empty() && !DIRECTORY_REFERENCES.contains(&name);
        holds_a_key.then(|| Name::Key {
            name: name.to_owned(),
            has_document: files.contains(name),
        })
    });
    let mut names: Vec<Name> = directories
        .into_iter()
        .map(Name::Directory)
        .chain(keys)
        .collect();
    names.sort_unstable_by(|left, right| left.sort_bytes().cmp(right.sort_bytes()));
    names
}

fn read_error(error: FailedFilesystemOperation) -> ReadError {
    ReadError::Backend(BackendError::from(error))
}

/// One step of an [`Unfold`]: a future that returns the next item and the state after it, or
/// `None` at the end.
type Step<S, T> = BoxFuture<'static, Option<(T, S)>>;

/// A stream that runs one future per item. Each future returns the item and the state that the
/// next future starts from.
struct Unfold<S, T> {
    step: fn(S) -> Step<S, T>,
    pending: Option<Step<S, T>>,
}

impl<S, T> Stream for Unfold<S, T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let Some(pending) = self.pending.as_mut() else {
            return Poll::Ready(None);
        };
        match pending.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => {
                self.pending = None;
                Poll::Ready(None)
            }
            Poll::Ready(Some((item, state))) => {
                self.pending = Some((self.step)(state));
                Poll::Ready(Some(item))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::repository::filesystem::Read;
    use crate::tests::fixtures::fake_list_repository::collect;
    use crate::tests::fixtures::sample_document::SampleDocument;

    fn repository_at(root: impl Into<PathBuf>) -> FilesystemRepository<Read, SampleDocument> {
        FilesystemRepository::builder()
            .reader()
            .document_type::<SampleDocument>()
            .root(root)
            .build()
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    /// Creates empty files at the given paths under the root, with their parent directories. The
    /// walk reads no file, so the files need no content.
    fn create_files(root: &Path, paths: &[&str]) {
        for path in paths {
            let path = root.join(path);
            let directory = path
                .parent()
                .expect("Given a path under a root, the path should always have a parent");
            std::fs::create_dir_all(directory).expect(
                "Given a writable temp directory, creating subdirectories should always succeed",
            );
            std::fs::write(&path, b"")
                .expect("Given an existing directory, writing a file should always succeed");
        }
    }

    fn walked(key: &str, has_document: bool) -> WalkedKey {
        let key = DocumentKey::new(key)
            .expect("Given a valid hardcoded relative path, the key should always build");
        WalkedKey { key, has_document }
    }

    fn prefix_at(path: &str) -> KeyPrefix {
        KeyPrefix::from(
            DocumentKey::new(path)
                .expect("Given a valid hardcoded relative path, the key should always build"),
        )
    }

    #[tokio::test]
    async fn should_yield_the_key_with_its_document_when_both_files_exist() {
        let root = temporary_root();
        create_files(root.path(), &["sec/x.json", "sec/x.json.meta.json"]);
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result = vec![Ok(walked("sec/x.json", true))];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_the_key_without_its_document_when_only_the_metadata_exists() {
        let root = temporary_root();
        create_files(root.path(), &["sec/x.json.meta.json"]);
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result = vec![Ok(walked("sec/x.json", false))];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_nothing_when_a_document_has_no_metadata() {
        let root = temporary_root();
        create_files(root.path(), &["sec/x.json"]);
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result: Vec<Result<WalkedKey, ReadError>> = vec![];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_nothing_when_the_directory_holds_only_temporary_files_of_a_write() {
        let root = temporary_root();
        create_files(
            root.path(),
            &["sec/.x.json.4242.0.tmp", "sec/.x.json.meta.json.4242.1.tmp"],
        );
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result: Vec<Result<WalkedKey, ReadError>> = vec![];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_the_file_a_dash_b_before_the_directory_a_when_both_sit_in_the_root() {
        let root = temporary_root();
        create_files(
            root.path(),
            &["a/x", "a/x.meta.json", "a-b", "a-b.meta.json"],
        );
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result = vec![Ok(walked("a-b", true)), Ok(walked("a/x", true))];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_the_keys_in_byte_order_when_directories_nest() {
        let root = temporary_root();
        create_files(
            root.path(),
            &[
                "b.meta.json",
                "a/c/d.meta.json",
                "a/b.meta.json",
                "a.meta.json",
                "a/c.e.meta.json",
            ],
        );
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result = vec![
            Ok(walked("a", false)),
            Ok(walked("a/b", false)),
            Ok(walked("a/c.e", false)),
            Ok(walked("a/c/d", false)),
            Ok(walked("b", false)),
        ];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_select_a_sibling_directory_when_its_name_extends_the_prefix() {
        let root = temporary_root();
        create_files(
            root.path(),
            &[
                "sec/notes/q1.zip",
                "sec/notes/q1.zip.meta.json",
                "sec/notes-old/a.zip",
                "sec/notes-old/a.zip.meta.json",
            ],
        );
        let repository = repository_at(root.path());
        let prefix = prefix_at("sec/notes");

        let expected_result = vec![Ok(walked("sec/notes/q1.zip", true))];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_nothing_when_the_prefix_names_no_directory() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let prefix = prefix_at("sec/notes");

        let expected_result: Vec<Result<WalkedKey, ReadError>> = vec![];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_nothing_when_the_prefix_names_a_file() {
        let root = temporary_root();
        create_files(root.path(), &["sec/x.json", "sec/x.json.meta.json"]);
        let repository = repository_at(root.path());
        let prefix = prefix_at("sec/x.json");

        let expected_result: Vec<Result<WalkedKey, ReadError>> = vec![];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_one_error_and_end_when_the_root_is_missing() {
        let root = temporary_root();
        let missing_root = root.path().join("missing");
        let repository = repository_at(&missing_root);
        let prefix = prefix_at("sec/notes");

        let expected_result = vec![Err(read_error(FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::MissingRoot,
            &missing_root,
        )))];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_one_error_and_end_when_the_root_is_a_file() {
        let root = temporary_root();
        create_files(root.path(), &["file"]);
        let file_root = root.path().join("file");
        let repository = repository_at(&file_root);
        let prefix = KeyPrefix::root();

        let expected_result = vec![Err(read_error(FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::RootIsNotADirectory,
            &file_root,
        )))];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_skip_the_metadata_file_when_its_stem_is_two_dots() {
        let root = temporary_root();
        create_files(root.path(), &["sec/...meta.json"]);
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result: Vec<Result<WalkedKey, ReadError>> = vec![];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_skip_the_metadata_file_when_its_stem_is_one_dot_in_a_subdirectory() {
        let root = temporary_root();
        create_files(root.path(), &["sec/..meta.json"]);
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result: Vec<Result<WalkedKey, ReadError>> = vec![];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_skip_the_metadata_file_when_its_stem_is_one_dot_in_the_root() {
        let root = temporary_root();
        create_files(root.path(), &["..meta.json"]);
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result: Vec<Result<WalkedKey, ReadError>> = vec![];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn should_skip_the_metadata_file_when_its_name_holds_a_colon() {
        let root = temporary_root();
        create_files(root.path(), &["sec/a:b.meta.json"]);
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result: Vec<Result<WalkedKey, ReadError>> = vec![];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn should_skip_the_metadata_file_when_its_name_is_not_valid_utf8() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let root = temporary_root();
        std::fs::write(root.path().join(OsStr::from_bytes(b"\xff.meta.json")), b"").expect(
            "Given a Linux temp directory, writing a file with any byte name should always succeed",
        );
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result: Vec<Result<WalkedKey, ReadError>> = vec![];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_the_file_when_its_name_is_only_the_metadata_suffix() {
        let files = HashSet::from([METADATA_SUFFIX.to_owned()]);

        let expected_result: Vec<Name> = vec![];

        let result = sorted_names(vec![], &files);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_read_as_a_missing_directory_when_a_name_in_it_disappeared() {
        let error = FailedFilesystemOperation::from_io(
            FilesystemOperation::Inspect,
            "/data/arkad/sec/.x.json.4242.0.tmp",
            &io::Error::from(io::ErrorKind::NotFound),
        );

        let expected_result = false;

        let result = names_no_directory(&error);

        assert_eq!(result, expected_result);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn should_skip_a_symbolic_link_when_it_points_to_a_directory() {
        let root = temporary_root();
        create_files(root.path(), &["target/x", "target/x.meta.json"]);
        std::os::unix::fs::symlink(root.path().join("target"), root.path().join("link")).expect(
            "Given a writable temp directory, creating a symbolic link should always succeed",
        );
        let repository = repository_at(root.path());
        let prefix = KeyPrefix::root();

        let expected_result = vec![Ok(walked("target/x", true))];

        let result = collect(repository.walk_keys(&prefix)).await;

        assert_eq!(result, expected_result);
    }
}
