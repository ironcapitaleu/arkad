//! # Notes Archive CIKs
//!
//! Provides [`distinct_ciks_of_notes_archive`], which reads the CIKs of the companies in one SEC
//! Financial Statement and Notes file.
//!
//! A Notes file is a ZIP. Its entry `sub.tsv` holds one row per filing, with the CIK of the filer
//! in the column `cik`. The reader decompresses only this entry.

use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};
use std::io::{BufRead, BufReader, Cursor};

use thiserror::Error;
use zip::ZipArchive;
use zip::result::ZipError;

/// The name of the entry of a Notes file that lists the filings.
pub const SUBMISSIONS_TABLE: &str = "sub.tsv";

/// The name of the column of [`SUBMISSIONS_TABLE`] that holds the CIK.
pub const CIK_COLUMN: &str = "cik";

/// Returns the distinct raw CIKs in the `cik` column of the `sub.tsv` entry of `archive`, sorted.
///
/// The first line of `sub.tsv` names the tab-separated columns. The reader finds the column by
/// the name `cik`, so the column order does not matter. A CIK keeps the text of the file, without
/// padding and without validation, for example `320193`. The reader skips empty lines and rows
/// without a value in the column `cik`.
///
/// # Errors
///
/// Returns an [`InvalidNotesArchive`] if `archive` is not a ZIP, has no `sub.tsv` entry, has a
/// `sub.tsv` that cannot be decompressed, or has a `sub.tsv` without a `cik` column.
///
/// # Examples
///
/// ```
/// use sec::shared::notes_archive_ciks::{
///     InvalidNotesArchive, InvalidNotesArchiveReason, distinct_ciks_of_notes_archive,
/// };
///
/// let archive = b"not a ZIP";
///
/// let expected_result = true;
///
/// let result = matches!(
///     distinct_ciks_of_notes_archive(archive),
///     Err(InvalidNotesArchive {
///         reason: InvalidNotesArchiveReason::NotAZip(_)
///     })
/// );
///
/// assert_eq!(result, expected_result);
/// ```
pub fn distinct_ciks_of_notes_archive(
    archive: &[u8],
) -> Result<BTreeSet<String>, InvalidNotesArchive> {
    let mut archive = ZipArchive::new(Cursor::new(archive)).map_err(|error| {
        InvalidNotesArchive::new(InvalidNotesArchiveReason::NotAZip(error.to_string()))
    })?;
    let table = archive.by_name(SUBMISSIONS_TABLE).map_err(|error| {
        InvalidNotesArchive::new(match error {
            ZipError::FileNotFound => InvalidNotesArchiveReason::MissingSubmissionsTable,
            error => InvalidNotesArchiveReason::UnreadableSubmissionsTable(error.to_string()),
        })
    })?;
    let unreadable = |error: std::io::Error| {
        InvalidNotesArchive::new(InvalidNotesArchiveReason::UnreadableSubmissionsTable(
            error.to_string(),
        ))
    };
    let mut lines = BufReader::new(table).split(b'\n');
    let header = lines
        .next()
        .transpose()
        .map_err(unreadable)?
        .unwrap_or_default();
    let cik_index = fields(&header)
        .position(|name| name == CIK_COLUMN.as_bytes())
        .ok_or_else(|| InvalidNotesArchive::new(InvalidNotesArchiveReason::MissingCikColumn))?;
    let mut ciks = BTreeSet::new();
    for line in lines {
        let line = line.map_err(unreadable)?;
        if trim_line_end(&line).is_empty() {
            continue;
        }
        let cik = fields(&line).nth(cik_index).unwrap_or_default();
        if !cik.is_empty() {
            ciks.insert(String::from_utf8_lossy(cik).into_owned());
        }
    }
    Ok(ciks)
}

/// Returns the tab-separated fields of `line`, without the line end.
fn fields(line: &[u8]) -> impl Iterator<Item = &[u8]> {
    trim_line_end(line).split(|byte| *byte == b'\t')
}

/// Returns `line` without a trailing carriage return.
fn trim_line_end(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\r").unwrap_or(line)
}

/// Error indicating a Notes file whose CIKs cannot be read.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("[InvalidNotesArchive] Invalid Notes file, Reason: '{reason}'")]
pub struct InvalidNotesArchive {
    /// The reason why the CIKs cannot be read.
    pub reason: InvalidNotesArchiveReason,
}

impl InvalidNotesArchive {
    /// Creates a new [`InvalidNotesArchive`] error from its reason.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::notes_archive_ciks::{InvalidNotesArchive, InvalidNotesArchiveReason};
    ///
    /// let reason = InvalidNotesArchiveReason::MissingCikColumn;
    ///
    /// let expected_result = "[InvalidNotesArchive] Invalid Notes file, Reason: 'sub.tsv has no column cik'";
    ///
    /// let result = InvalidNotesArchive::new(reason).to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(reason: InvalidNotesArchiveReason) -> Self {
        Self { reason }
    }
}

/// Enum representing the reason why the CIKs of a Notes file cannot be read.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InvalidNotesArchiveReason {
    /// The bytes are not a ZIP. Holds the message of the ZIP reader.
    NotAZip(String),
    /// The ZIP has no `sub.tsv` entry.
    MissingSubmissionsTable,
    /// The `sub.tsv` entry cannot be opened or decompressed, for example because it is encrypted
    /// or uses an unsupported compression method. Holds the message of the ZIP reader.
    UnreadableSubmissionsTable(String),
    /// The first line of `sub.tsv` names no column `cik`.
    MissingCikColumn,
}

impl Display for InvalidNotesArchiveReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAZip(message) => write!(f, "not a ZIP, {message}"),
            Self::MissingSubmissionsTable => write!(f, "no entry {SUBMISSIONS_TABLE}"),
            Self::UnreadableSubmissionsTable(message) => {
                write!(f, "{SUBMISSIONS_TABLE} cannot be decompressed, {message}")
            }
            Self::MissingCikColumn => write!(f, "{SUBMISSIONS_TABLE} has no column {CIK_COLUMN}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::data::notes_archive::{SAMPLE_NOTES_ZIP, notes_zip};

    /// Returns `archive` with the encryption flag set on every entry, in the local header and in
    /// the central directory. The ZIP reader then refuses to open an entry without a password.
    fn with_encryption_flag(archive: &[u8]) -> Vec<u8> {
        const LOCAL_HEADER: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
        const CENTRAL_HEADER: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
        let mut patched = archive.to_vec();
        for start in 0..archive.len().saturating_sub(4) {
            let flag_offset = match &archive[start..start + 4] {
                signature if signature == LOCAL_HEADER => 6,
                signature if signature == CENTRAL_HEADER => 8,
                _ => continue,
            };
            patched[start + flag_offset] |= 1;
        }
        patched
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_invalid_notes_archive() {
        implements_auto_traits::<InvalidNotesArchive>();
    }

    #[test]
    fn should_return_each_cik_once_sorted_when_two_filings_share_a_cik() {
        let archive = notes_zip(
            "adsh\tcik\tname\n\
             0000320193-26-000001\t320193\tAPPLE INC\n\
             0001067983-26-000002\t1067983\tBERKSHIRE HATHAWAY INC\n\
             0000320193-26-000003\t320193\tAPPLE INC\n",
        );

        let expected_result = Ok(BTreeSet::from([
            "1067983".to_string(),
            "320193".to_string(),
        ]));

        let result = distinct_ciks_of_notes_archive(&archive);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_the_column_named_cik_when_it_is_not_the_first_column() {
        let archive = notes_zip("name\tadsh\tcik\r\nAPPLE INC\t0000320193-26-000001\t320193\r\n");

        let expected_result = Ok(BTreeSet::from(["320193".to_string()]));

        let result = distinct_ciks_of_notes_archive(&archive);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_an_invalid_cik_as_written_when_a_row_holds_one() {
        let archive = notes_zip("adsh\tcik\n0000320193-26-000001\tABC\n");

        let expected_result = Ok(BTreeSet::from(["ABC".to_string()]));

        let result = distinct_ciks_of_notes_archive(&archive);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_an_empty_line_when_sub_tsv_holds_one() {
        let archive = notes_zip("adsh\tcik\n0000320193-26-000001\t320193\n\r\n\n");

        let expected_result = Ok(BTreeSet::from(["320193".to_string()]));

        let result = distinct_ciks_of_notes_archive(&archive);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_a_row_without_a_cik_when_the_row_is_short() {
        let archive = notes_zip(
            "adsh\tname\tcik\n0000320193-26-000001\tAPPLE INC\t320193\n0000320193-26-000002\n",
        );

        let expected_result = Ok(BTreeSet::from(["320193".to_string()]));

        let result = distinct_ciks_of_notes_archive(&archive);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unreadable_submissions_table_when_the_zip_cannot_open_sub_tsv() {
        let archive = with_encryption_flag(&notes_zip("adsh\tcik\n"));

        let expected_result = true;

        let result = matches!(
            distinct_ciks_of_notes_archive(&archive),
            Err(InvalidNotesArchive {
                reason: InvalidNotesArchiveReason::UnreadableSubmissionsTable(_)
            })
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_cik_column_when_the_header_names_no_cik() {
        let archive = notes_zip("adsh\tname\n0000320193-26-000001\tAPPLE INC\n");

        let expected_result = Err(InvalidNotesArchive::new(
            InvalidNotesArchiveReason::MissingCikColumn,
        ));

        let result = distinct_ciks_of_notes_archive(&archive);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_submissions_table_when_the_zip_has_no_sub_tsv() {
        let archive = SAMPLE_NOTES_ZIP;

        let expected_result = Err(InvalidNotesArchive::new(
            InvalidNotesArchiveReason::MissingSubmissionsTable,
        ));

        let result = distinct_ciks_of_notes_archive(archive);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_not_a_zip_when_the_bytes_are_not_a_zip() {
        let archive = b"PK";

        let expected_result = true;

        let result = matches!(
            distinct_ciks_of_notes_archive(archive),
            Err(InvalidNotesArchive {
                reason: InvalidNotesArchiveReason::NotAZip(_)
            })
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_the_reason_when_displaying_invalid_notes_archive() {
        let error = InvalidNotesArchive::new(InvalidNotesArchiveReason::MissingSubmissionsTable);

        let expected_result =
            "[InvalidNotesArchive] Invalid Notes file, Reason: 'no entry sub.tsv'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
