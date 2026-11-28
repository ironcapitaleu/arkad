//! # Table Read Error
//!
//! Provides [`TableReadError`], the error of a read of one table of a Notes file.

use thiserror::Error;

use super::notes_table::NotesTable;

/// Enum representing the reason why a table of a Notes file could not be read.
///
/// [`NotesArchive::table`](super::NotesArchive::table) and each item of a
/// [`TableReader`](super::TableReader) return it. Every variant fails the whole Notes file, not one
/// filing.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TableReadError {
    /// The header of the table differs from the columns of the metadata member.
    #[error(
        "[UnexpectedHeader] The header differs from the columns of the metadata member, Table: '{table}', Expected: '{}', Actual: '{}'",
        .expected.join(", "),
        .actual.join(", ")
    )]
    UnexpectedHeader {
        /// The table.
        table: NotesTable,
        /// The columns of the metadata member, in order.
        expected: Vec<String>,
        /// The columns of the header, in order. Empty if the table has no header.
        actual: Vec<String>,
    },

    /// A row of the table is not valid UTF-8.
    #[error("[InvalidUtf8] The row is not valid UTF-8, Table: '{table}', Line: {line}")]
    InvalidUtf8 {
        /// The table.
        table: NotesTable,
        /// The line of the row in the member, starting at 1.
        line: u64,
    },

    /// A row of the table has another number of fields than the header.
    #[error(
        "[UnexpectedFieldCount] The row has another number of fields than the header, Table: '{table}', Line: {line}, Expected: {expected}, Actual: {actual}"
    )]
    UnexpectedFieldCount {
        /// The table.
        table: NotesTable,
        /// The line of the row in the member, starting at 1.
        line: u64,
        /// The number of fields of the header.
        expected: u64,
        /// The number of fields of the row.
        actual: u64,
    },

    /// The unzipped bytes of the member have another CRC-32 than the ZIP index records.
    #[error(
        "[InvalidMemberChecksum] The CRC-32 of the table differs from the ZIP index, Table: '{table}', Expected: {expected:08x}, Actual: {actual:08x}"
    )]
    InvalidMemberChecksum {
        /// The table.
        table: NotesTable,
        /// The CRC-32 of the ZIP index.
        expected: u32,
        /// The CRC-32 of the unzipped bytes.
        actual: u32,
    },

    /// The member unzips to another size than the ZIP index records.
    #[error(
        "[InvalidMemberSize] The table unzips to another size than the ZIP index records, Table: '{table}', Expected: {expected} bytes, Actual: {actual} bytes"
    )]
    InvalidMemberSize {
        /// The table.
        table: NotesTable,
        /// The unzipped size of the ZIP index.
        expected: u64,
        /// The number of unzipped bytes, at most one more than `expected`.
        actual: u64,
    },

    /// The deflate stream of the member is damaged.
    #[error("[FailedTableRead] Failed to unzip the table, Table: '{table}', Reason: '{reason}'")]
    FailedTableRead {
        /// The table.
        table: NotesTable,
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
    const fn should_implement_auto_traits_for_table_read_error() {
        implements_auto_traits::<TableReadError>();
    }

    #[test]
    fn should_print_both_column_lists_when_the_header_differs() {
        let error = TableReadError::UnexpectedHeader {
            table: NotesTable::Dim,
            expected: vec!["dimhash".to_owned(), "segments".to_owned()],
            actual: vec!["dimhash".to_owned()],
        };

        let expected_result = "[UnexpectedHeader] The header differs from the columns of the metadata member, Table: 'dim.tsv', Expected: 'dimhash, segments', Actual: 'dimhash'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_checksums_as_hex_when_the_checksum_differs() {
        let error = TableReadError::InvalidMemberChecksum {
            table: NotesTable::Num,
            expected: 0x00ab_cdef,
            actual: 0x1234_5678,
        };

        let expected_result = "[InvalidMemberChecksum] The CRC-32 of the table differs from the ZIP index, Table: 'num.tsv', Expected: 00abcdef, Actual: 12345678";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
