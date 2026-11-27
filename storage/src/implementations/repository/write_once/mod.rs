//! # Write-Once Repository
//!
//! Provides the [`Blob`] that a write-once repository reads and writes, the [`ListedBlob`] it
//! lists, and its backends. A write-once repository writes each key once and never replaces it.
//!
//! ## Modules
//!
//! - [`filesystem`]: The [`WriteOnceFilesystemRepository`] storing each blob once as a file under
//!   a root directory.

use bytes::Bytes;

use crate::implementations::repository::filesystem::DocumentKey;

pub mod filesystem;

pub use filesystem::WriteOnceFilesystemRepository;

/// One blob: its key and its bytes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blob {
    /// The key of the blob in the repository.
    pub key: DocumentKey,
    /// The bytes of the blob.
    pub bytes: Bytes,
}

/// One blob that a listing found: its key and its size.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListedBlob {
    /// The key of the blob in the repository.
    pub key: DocumentKey,
    /// The number of bytes the blob holds.
    pub size_in_bytes: u64,
}

impl Blob {
    /// Creates a [`Blob`] from its key and its bytes.
    #[must_use]
    pub fn new(key: DocumentKey, bytes: impl Into<Bytes>) -> Self {
        Self {
            key,
            bytes: bytes.into(),
        }
    }
}
