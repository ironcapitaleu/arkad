//! # Filesystem Lease Repository
//!
//! Provides [`FilesystemLeaseRepository`], which stores each lease of a job kind as a file under a
//! root directory.
//!
//! ## Usage
//!
//! ```rust
//! use chrono::{TimeDelta, Utc};
//! use storage::implementations::repository::lease::filesystem::FilesystemLeaseRepository;
//! use storage::{JobKind, LeaseGeneration, LeaseRepository};
//!
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() {
//! # let root = tempfile::TempDir::new().expect("Given a writable temp directory, it should build");
//! let repository = FilesystemLeaseRepository::new(root.path());
//! let kind = JobKind::new("transform-notes")
//!     .expect("Given a valid hardcoded job kind, the creation should always succeed");
//! let expires_at = Utc::now() + TimeDelta::hours(16);
//!
//! let expected_result = LeaseGeneration(1);
//!
//! let result = repository
//!     .acquire(&kind, "2026-11-21T180000.000Z", expires_at)
//!     .await
//!     .expect("Given a job kind without a lease, the acquire should always succeed");
//!
//! assert_eq!(result, expected_result);
//! # }
//! ```

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::{BackendError, LeaseError};
use crate::implementations::repository::filesystem::{
    FailedFilesystemOperation, FailedFilesystemOperationReason, FilesystemOperation, check_root,
};
use crate::shared::{JobKind, Lease, LeaseGeneration};
use crate::traits::repository::LeaseRepository;

/// The directory under the root that holds one directory of leases per job kind.
pub const LEASES_DIRECTORY: &str = "leases";

/// The number that makes the name of each temporary file of this process unique.
static TEMPORARY_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Stores the leases of each job kind as JSON files under one root directory.
///
/// The lease of generation `n` of a job kind lives at `leases/<kind>/<n>`. The current lease is the
/// file with the highest generation. A new lease is a new file with the next generation, so the
/// repository keeps each older lease.
///
/// A write first writes and syncs a hidden temporary file in the same directory. It then links
/// the file to its generation. The link fails if the generation exists, so of two acquires at the
/// same time only one writes the generation. The other reads the new lease again. A reader sees a
/// whole lease or none.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilesystemLeaseRepository {
    root: PathBuf,
}

impl FilesystemLeaseRepository {
    /// Creates a [`FilesystemLeaseRepository`] under the root directory.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the root directory the leases resolve under.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the directory that holds the leases of a job kind.
    fn kind_directory(&self, kind: &JobKind) -> PathBuf {
        self.root.join(LEASES_DIRECTORY).join(kind.as_str())
    }

    /// Reads the lease with the highest generation of a job kind.
    async fn read_current(
        &self,
        kind: &JobKind,
    ) -> Result<Option<(Lease, LeaseGeneration)>, FailedFilesystemOperation> {
        check_root(&self.root).await?;
        let directory = self.kind_directory(kind);
        let Some(generation) = latest_generation(&directory).await? else {
            return Ok(None);
        };
        let path = directory.join(generation.to_string());
        let bytes = fs::read(&path).await.map_err(|error| {
            FailedFilesystemOperation::from_io(FilesystemOperation::Read, &path, &error)
        })?;
        let lease = serde_json::from_slice(&bytes).map_err(|error| {
            FailedFilesystemOperation::new(
                FailedFilesystemOperationReason::InvalidLease {
                    line: error.line(),
                    column: error.column(),
                },
                &path,
            )
        })?;
        Ok(Some((lease, generation)))
    }

    /// Writes the lease as the generation of the job kind.
    ///
    /// Returns `Ok(false)` if the generation exists.
    async fn write_generation(
        &self,
        kind: &JobKind,
        generation: LeaseGeneration,
        lease: &Lease,
    ) -> Result<bool, FailedFilesystemOperation> {
        let directory = self.kind_directory(kind);
        fs::create_dir_all(&directory).await.map_err(|error| {
            FailedFilesystemOperation::from_io(
                FilesystemOperation::CreateDirectory,
                &directory,
                &error,
            )
        })?;
        let temporary_path = directory.join(format!(
            ".{generation}.{}.{}.tmp",
            std::process::id(),
            TEMPORARY_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let linked = async {
            write_synced(&temporary_path, lease).await?;
            let path = directory.join(generation.to_string());
            match fs::hard_link(&temporary_path, &path).await {
                Ok(()) => Ok(true),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(false),
                Err(error) => Err(FailedFilesystemOperation::from_io(
                    FilesystemOperation::Link,
                    &path,
                    &error,
                )),
            }
        }
        .await;
        // The generation file holds the lease, or the write failed. Either way the temporary file
        // has no further use. If the removal fails, a later listing skips the hidden name.
        let _ = fs::remove_file(&temporary_path).await;
        linked
    }
}

#[async_trait]
impl LeaseRepository for FilesystemLeaseRepository {
    /// Takes the lease of the job kind, as the contract of [`LeaseRepository::acquire`] says.
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`LeaseError::LeaseHeld`] if the lease of the job kind has not expired.
    /// - [`LeaseError::Backend`] that holds a [`FailedFilesystemOperation`] if the root does not
    ///   exist or is not a directory, the current lease file holds no valid lease, or the
    ///   filesystem cannot read, create, write, sync, or link a file.
    /// - [`LeaseError::Backend`] that holds a [`BackendError::FailedOperation`] with a reason if
    ///   the current generation is `u64::MAX`.
    async fn acquire(
        &self,
        kind: &JobKind,
        plan_id: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<LeaseGeneration, LeaseError> {
        let lease = Lease {
            plan_id: plan_id.to_owned(),
            expires_at,
        };
        loop {
            let generation = match self.read_current(kind).await.map_err(backend_error)? {
                Some((held, _)) if held.expires_at > Utc::now() => {
                    return Err(LeaseError::LeaseHeld {
                        kind: kind.clone(),
                        lease: held,
                    });
                }
                Some((_, LeaseGeneration(previous))) => {
                    LeaseGeneration(previous.checked_add(1).ok_or_else(|| {
                        LeaseError::Backend(BackendError::failed_operation(format!(
                            "Lease generation {previous} of job kind '{kind}' has no next generation"
                        )))
                    })?)
                }
                None => LeaseGeneration(1),
            };
            // A write that finds its generation taken lost a race. The next pass reads the
            // lease that won.
            if self
                .write_generation(kind, generation, &lease)
                .await
                .map_err(backend_error)?
            {
                return Ok(generation);
            }
        }
    }

    /// Returns the lease with the highest generation of the job kind.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError::Backend`] that holds a [`FailedFilesystemOperation`] if the root does
    /// not exist or is not a directory, the lease file holds no valid lease, or the filesystem
    /// cannot list the directory or read the file.
    async fn current(
        &self,
        kind: &JobKind,
    ) -> Result<Option<(Lease, LeaseGeneration)>, LeaseError> {
        self.read_current(kind).await.map_err(backend_error)
    }
}

/// Returns the highest generation among the file names of the directory, or `None` if the
/// directory does not exist or holds no generation.
///
/// A generation is a name of ASCII digits only, so the walk skips a hidden temporary file.
async fn latest_generation(
    directory: &Path,
) -> Result<Option<LeaseGeneration>, FailedFilesystemOperation> {
    let list_error = |error: io::Error| {
        FailedFilesystemOperation::from_io(FilesystemOperation::ListDirectory, directory, &error)
    };
    let mut entries = match fs::read_dir(directory).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(list_error(error)),
    };
    let mut latest = None;
    while let Some(entry) = entries.next_entry().await.map_err(list_error)? {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        if let Ok(generation) = name.parse::<u64>() {
            latest = latest.max(Some(LeaseGeneration(generation)));
        }
    }
    Ok(latest)
}

/// Creates the file, writes the lease as JSON, and syncs it.
async fn write_synced(path: &Path, lease: &Lease) -> Result<(), FailedFilesystemOperation> {
    let io_error = |operation| {
        move |error: io::Error| FailedFilesystemOperation::from_io(operation, path, &error)
    };
    let json = serde_json::to_vec(lease)
        .expect("Given a lease of a string and a time, the serialization should always succeed");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(io_error(FilesystemOperation::CreateFile))?;
    file.write_all(&json)
        .await
        .map_err(io_error(FilesystemOperation::Write))?;
    file.sync_all()
        .await
        .map_err(io_error(FilesystemOperation::Sync))
}

fn backend_error(error: FailedFilesystemOperation) -> LeaseError {
    LeaseError::Backend(BackendError::from(error))
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn transform_notes() -> JobKind {
        JobKind::new("transform-notes")
            .expect("Given a valid hardcoded job kind, the creation should always succeed")
    }

    fn in_hours(hours: i64) -> DateTime<Utc> {
        Utc::now() + TimeDelta::hours(hours)
    }

    async fn acquired(
        repository: &FilesystemLeaseRepository,
        plan_id: &str,
        expires_at: DateTime<Utc>,
    ) {
        repository
            .acquire(&transform_notes(), plan_id, expires_at)
            .await
            .expect("Given a job kind without a live lease, the acquire should always succeed");
    }

    const fn implements_lease_repository<T: LeaseRepository>() {}
    #[test]
    const fn should_implement_lease_repository_for_filesystem_lease_repository() {
        implements_lease_repository::<FilesystemLeaseRepository>();
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filesystem_lease_repository() {
        implements_auto_traits::<FilesystemLeaseRepository>();
    }

    #[tokio::test]
    async fn should_return_lease_held_when_the_lease_has_not_expired() {
        let root = temporary_root();
        let repository = FilesystemLeaseRepository::new(root.path());
        let expires_at = in_hours(16);
        acquired(&repository, "plan-a", expires_at).await;
        let kind = transform_notes();
        let plan_id = "plan-b";
        let new_expires_at = in_hours(16);

        let expected_result = Err(LeaseError::LeaseHeld {
            kind: transform_notes(),
            lease: Lease {
                plan_id: "plan-a".to_owned(),
                expires_at,
            },
        });

        let result = repository.acquire(&kind, plan_id, new_expires_at).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_next_generation_when_the_lease_expired() {
        let root = temporary_root();
        let repository = FilesystemLeaseRepository::new(root.path());
        acquired(&repository, "plan-a", in_hours(-1)).await;
        let kind = transform_notes();
        let plan_id = "plan-b";
        let expires_at = in_hours(16);

        let expected_result = Ok(LeaseGeneration(2));

        let result = repository.acquire(&kind, plan_id, expires_at).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_new_lease_when_an_acquire_replaced_an_expired_lease() {
        let root = temporary_root();
        let repository = FilesystemLeaseRepository::new(root.path());
        let expires_at = in_hours(16);
        acquired(&repository, "plan-a", in_hours(-1)).await;
        acquired(&repository, "plan-b", expires_at).await;
        let kind = transform_notes();

        let expected_result = Ok(Some((
            Lease {
                plan_id: "plan-b".to_owned(),
                expires_at,
            },
            LeaseGeneration(2),
        )));

        let result = repository.current(&kind).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_the_job_kind_has_no_lease() {
        let root = temporary_root();
        let repository = FilesystemLeaseRepository::new(root.path());
        let kind = transform_notes();

        let expected_result = Ok(None);

        let result = repository.current(&kind).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_let_one_acquire_win_when_two_run_at_the_same_time() {
        let root = temporary_root();
        let repository = FilesystemLeaseRepository::new(root.path());
        let kind = transform_notes();
        let first_plan_id = "plan-a";
        let second_plan_id = "plan-b";
        let expires_at = in_hours(16);

        let expected_result = 1;

        let (first, second) = tokio::join!(
            repository.acquire(&kind, first_plan_id, expires_at),
            repository.acquire(&kind, second_plan_id, expires_at),
        );
        let result = usize::from(first.is_ok()) + usize::from(second.is_ok());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_lease_held_to_the_loser_when_two_acquires_run_at_the_same_time() {
        let root = temporary_root();
        let repository = FilesystemLeaseRepository::new(root.path());
        let kind = transform_notes();
        let first_plan_id = "plan-a";
        let second_plan_id = "plan-b";
        let expires_at = in_hours(16);

        let expected_result = 1;

        let (first, second) = tokio::join!(
            repository.acquire(&kind, first_plan_id, expires_at),
            repository.acquire(&kind, second_plan_id, expires_at),
        );
        let result = usize::from(matches!(first, Err(LeaseError::LeaseHeld { .. })))
            + usize::from(matches!(second, Err(LeaseError::LeaseHeld { .. })));

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_skip_a_temporary_file_and_a_signed_name_when_finding_the_latest_generation() {
        let root = temporary_root();
        let directory = root.path().join("leases/transform-notes");
        std::fs::create_dir_all(&directory)
            .expect("Given a writable temp directory, creating subdirectories should succeed");
        for name in ["2", ".7.1.0.tmp", "+9"] {
            std::fs::write(directory.join(name), b"")
                .expect("Given an existing directory, writing a file should always succeed");
        }

        let expected_result = Ok(Some(LeaseGeneration(2)));

        let result = latest_generation(&directory).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_a_backend_error_when_the_latest_generation_is_the_maximum() {
        let root = temporary_root();
        let directory = root.path().join("leases/transform-notes");
        std::fs::create_dir_all(&directory)
            .expect("Given a writable temp directory, creating subdirectories should succeed");
        std::fs::write(
            directory.join(u64::MAX.to_string()),
            br#"{"plan_id":"plan-a","expires_at":"1970-01-01T00:00:00Z"}"#,
        )
        .expect("Given an existing directory, writing a file should always succeed");
        let repository = FilesystemLeaseRepository::new(root.path());
        let kind = transform_notes();
        let plan_id = "plan-b";
        let expires_at = in_hours(16);

        let expected_result = Err(LeaseError::Backend(BackendError::failed_operation(
            "Lease generation 18446744073709551615 of job kind 'transform-notes' has no next generation",
        )));

        let result = repository.acquire(&kind, plan_id, expires_at).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_root_when_the_root_does_not_exist() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let repository = FilesystemLeaseRepository::new(&root);
        let kind = transform_notes();
        let plan_id = "plan-a";
        let expires_at = in_hours(16);

        let expected_result = Err(LeaseError::Backend(BackendError::from(
            FailedFilesystemOperation::new(FailedFilesystemOperationReason::MissingRoot, &root),
        )));

        let result = repository.acquire(&kind, plan_id, expires_at).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_lease_when_the_lease_file_holds_no_json() {
        let root = temporary_root();
        let directory = root.path().join("leases/transform-notes");
        std::fs::create_dir_all(&directory)
            .expect("Given a writable temp directory, creating subdirectories should succeed");
        std::fs::write(directory.join("1"), b"{")
            .expect("Given an existing directory, writing a file should always succeed");
        let repository = FilesystemLeaseRepository::new(root.path());
        let kind = transform_notes();

        let expected_result = Err(LeaseError::Backend(BackendError::from(
            FailedFilesystemOperation::new(
                FailedFilesystemOperationReason::InvalidLease { line: 1, column: 1 },
                directory.join("1"),
            ),
        )));

        let result = repository.current(&kind).await;

        assert_eq!(result, expected_result);
    }
}
