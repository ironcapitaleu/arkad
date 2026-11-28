//! # Notes Archive Constants
//!
//! Provides the ZIP signature, the names of the metadata member, the size limits, and the header
//! aliases that [`NotesArchive::open`](super::NotesArchive::open) checks.

use super::notes_table::NotesTable;

/// The four bytes that start every ZIP file: the signature of a local file header.
pub const ZIP_SIGNATURE: &[u8; 4] = b"PK\x03\x04";

/// The name of the metadata member of a Notes file.
pub const METADATA_MEMBER: &str = "notes-metadata.json";

/// The end of the name of a metadata member that starts with its period, as in
/// `2009q1_notes-metadata.json`.
pub const PERIOD_METADATA_MEMBER_SUFFIX: &str = "_notes-metadata.json";

/// The largest sum of the unzipped sizes of the members of one Notes file, in bytes: 16 GB.
///
/// The largest measured file, `2024q1`, unzips to 4.052 GB.
pub const MAX_UNZIPPED_ARCHIVE_SIZE: u64 = 16_000_000_000;

/// The largest ratio of the unzipped size of one member to its zipped size.
///
/// The highest measured ratio is 12.8, for `cal.tsv` of `2024q1`.
pub const MAX_COMPRESSION_RATIO: u64 = 50;

/// The header columns whose name differs from the column name in the metadata member, as
/// `(table, metadata name, header name)`.
///
/// The metadata member names the first column of `dim.tsv` `dimh`, and the header of `dim.tsv`
/// names it `dimhash`.
pub const HEADER_ALIASES: [(NotesTable, &str, &str); 1] = [(NotesTable::Dim, "dimh", "dimhash")];
