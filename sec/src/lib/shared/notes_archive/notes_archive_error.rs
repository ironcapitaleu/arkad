//! # Notes Archive Error
//!
//! Provides [`NotesArchiveError`], the error of a read or an open of a Notes file.

use storage::ReadError;
use thiserror::Error;

/// Enum representing the reason why a Notes file could not be read or opened.
///
/// [`NotesArchive::read`](super::NotesArchive::read) and
/// [`NotesArchive::open`](super::NotesArchive::open) return it. Every variant fails the whole
/// Notes file, not one filing.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NotesArchiveError {
    /// The repository failed to read the Notes file.
    #[error("[FailedArchiveRead] Failed to read the Notes file, Caused by: {0}")]
    FailedArchiveRead(#[source] ReadError),

    /// The repository holds no Notes file under the key.
    #[error("[MissingArchive] No Notes file is stored under the key, Key: '{key}'")]
    MissingArchive {
        /// The key of the Notes file.
        key: String,
    },

    /// The bytes do not start with the ZIP signature `PK\x03\x04`.
    #[error("[MissingZipSignature] The Notes file does not start with the ZIP signature")]
    MissingZipSignature,

    /// The ZIP index does not parse, or a member lies outside the bytes of the file.
    #[error("[InvalidZipIndex] Failed to parse the ZIP index, Reason: '{reason}'")]
    InvalidZipIndex {
        /// The text of the parse error.
        reason: String,
    },

    /// The name of a member holds a path separator, `/` or `\`.
    #[error("[InvalidMemberName] The member name holds a path separator, Member: '{member}'")]
    InvalidMemberName {
        /// The name of the member.
        member: String,
    },

    /// A member uses another compression method than deflate.
    #[error(
        "[UnexpectedCompression] The member is not compressed with deflate, Member: '{member}'"
    )]
    UnexpectedCompression {
        /// The name of the member.
        member: String,
    },

    /// The unzipped size of a member is more than the maximum ratio times its zipped size.
    #[error(
        "[OversizedMember] The member unzips to more than {max_ratio} times its zipped size, Member: '{member}', Zipped: {zipped_size} bytes, Unzipped: {unzipped_size} bytes"
    )]
    OversizedMember {
        /// The name of the member.
        member: String,
        /// The zipped size of the member, from the ZIP index.
        zipped_size: u64,
        /// The unzipped size of the member, from the ZIP index.
        unzipped_size: u64,
        /// The largest ratio of the unzipped size to the zipped size.
        max_ratio: u64,
    },

    /// The sum of the unzipped sizes of all members is more than the limit.
    #[error(
        "[OversizedArchive] The Notes file unzips to more than the limit, Unzipped: {unzipped_size} bytes, Limit: {limit} bytes"
    )]
    OversizedArchive {
        /// The sum of the unzipped sizes of all members, from the ZIP index.
        unzipped_size: u64,
        /// The largest sum of the unzipped sizes.
        limit: u64,
    },

    /// One of the eight tables has no member.
    #[error("[MissingMember] The Notes file has no member for the table, Member: '{member}'")]
    MissingMember {
        /// The name of the missing member.
        member: String,
    },

    /// The Notes file holds no metadata member, or more than one.
    #[error(
        "[UnexpectedMetadataMemberCount] The Notes file must hold exactly one metadata member, Count: {count}"
    )]
    UnexpectedMetadataMemberCount {
        /// The number of metadata members.
        count: usize,
    },

    /// The metadata member does not unzip to its CRC-32 and size, does not parse, or lists no
    /// columns for a table.
    #[error(
        "[InvalidMetadata] Failed to read the columns of the metadata member, Reason: '{reason}'"
    )]
    InvalidMetadata {
        /// The text of the error.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_notes_archive_error() {
        implements_auto_traits::<NotesArchiveError>();
    }

    #[test]
    fn should_print_the_cause_when_the_read_fails() {
        let read_error = ReadError::missing_record("sec/notes/2024q1_notes.zip");
        let error = NotesArchiveError::FailedArchiveRead(read_error);

        let expected_result = "[FailedArchiveRead] Failed to read the Notes file, Caused by: [MissingRecord] Requested record 'sec/notes/2024q1_notes.zip' not found";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_both_sizes_when_a_member_is_oversized() {
        let error = NotesArchiveError::OversizedMember {
            member: "cal.tsv".to_owned(),
            zipped_size: 10,
            unzipped_size: 501,
            max_ratio: 50,
        };

        let expected_result = "[OversizedMember] The member unzips to more than 50 times its zipped size, Member: 'cal.tsv', Zipped: 10 bytes, Unzipped: 501 bytes";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
