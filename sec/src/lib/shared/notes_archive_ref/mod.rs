//! # Notes Archive Reference
//!
//! Provides the [`NotesArchiveRef`] type, which names one stored Notes file without holding its
//! bytes.

use std::fmt::{self, Display, Formatter};

use storage::Sha256Digest;
use storage::implementations::repository::filesystem::DocumentKey;

use crate::shared::notes_period::NotesPeriod;

/// The period, the key, the size, and the SHA-256 digest of one stored Notes file.
///
/// A reader finds the bytes under the key and checks them against the size and the digest.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotesArchiveRef {
    period: NotesPeriod,
    key: DocumentKey,
    size_in_bytes: u64,
    sha256: Sha256Digest,
}

impl NotesArchiveRef {
    /// Creates a new [`NotesArchiveRef`] from the period, the key, the size in bytes, and the
    /// SHA-256 digest of a stored Notes file.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::notes_archive_ref::NotesArchiveRef;
    /// use sec::shared::notes_period::NotesPeriod;
    /// use storage::Sha256Digest;
    /// use storage::implementations::repository::filesystem::DocumentKey;
    ///
    /// let period = NotesPeriod::new("2024q1")
    ///     .expect("Given a hardcoded published quarter, parsing should always succeed");
    /// let key = DocumentKey::new("sec/notes/2024q1_notes.zip")
    ///     .expect("Given a hardcoded valid relative path, the key should always build");
    /// let archive = NotesArchiveRef::new(period, key, 2, Sha256Digest::of(b"PK"));
    ///
    /// let expected_result = 2;
    ///
    /// let result = archive.size_in_bytes();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(
        period: NotesPeriod,
        key: DocumentKey,
        size_in_bytes: u64,
        sha256: Sha256Digest,
    ) -> Self {
        Self {
            period,
            key,
            size_in_bytes,
            sha256,
        }
    }

    /// Returns the period of the Notes file.
    #[must_use]
    pub const fn period(&self) -> NotesPeriod {
        self.period
    }

    /// Returns the key of the stored Notes file.
    #[must_use]
    pub const fn key(&self) -> &DocumentKey {
        &self.key
    }

    /// Returns the size of the stored Notes file in bytes.
    #[must_use]
    pub const fn size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }

    /// Returns the SHA-256 digest of the stored Notes file.
    #[must_use]
    pub const fn sha256(&self) -> &Sha256Digest {
        &self.sha256
    }
}

/// Formats as `Period: <period>, Key: <key>, Size: <size> bytes, SHA-256: <digest>`.
impl Display for NotesArchiveRef {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Period: {}, Key: {}, Size: {} bytes, SHA-256: {}",
            self.period,
            self.key.as_path().display(),
            self.size_in_bytes,
            self.sha256
        )
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    fn sample_archive() -> NotesArchiveRef {
        let period = NotesPeriod::new("2010q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed");
        let key = DocumentKey::new("sec/notes/2010q1_notes_1.zip")
            .expect("Given a hardcoded valid relative path, the key should always build");
        NotesArchiveRef::new(period, key, 2, Sha256Digest::of(b"PK"))
    }

    #[test]
    fn should_display_the_period_the_key_the_size_and_the_digest_when_formatted() {
        let archive = sample_archive();

        let expected_result = format!(
            "Period: 2010q1, Key: sec/notes/2010q1_notes_1.zip, Size: 2 bytes, SHA-256: {}",
            Sha256Digest::of(b"PK")
        );

        let result = archive.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_notes_archive_ref() {
        implements_value_traits::<NotesArchiveRef>();
    }
}
