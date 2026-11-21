//! # Silver Object Store
//!
//! Provides the [`SilverObject`] that a silver object store reads and writes, the [`ListedObject`]
//! it lists, and its backends.
//!
//! ## Modules
//!
//! - [`filesystem`]: The [`FilesystemSilverObjectStore`] storing each silver object once as a file
//!   under a root directory.

use bytes::Bytes;

use crate::implementations::repository::filesystem::DocumentKey;

pub mod filesystem;

pub use filesystem::FilesystemSilverObjectStore;

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
