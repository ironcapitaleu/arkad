//! # Table Reader
//!
//! Provides [`TableReader`], which unzips one table of a Notes file and reads it row by row, and
//! [`TableRow`], one row of a table.

use std::fmt::{self, Display, Formatter};
use std::io::{self, Cursor, Read, Take};

use bytes::Bytes;
use csv::{ErrorKind, ReaderBuilder, StringRecord};
use flate2::Crc;
use flate2::read::DeflateDecoder;

use super::notes_table::NotesTable;
use super::table_read_error::TableReadError;

/// Reads the rows of one table of a Notes file, in the order of the member.
///
/// The reader follows the dialect of the Notes files: a tab between fields, `"` around a field
/// that holds a tab or a quote, and `""` for a quote inside such a field. Each item is one row, or
/// the first error. After an error, the reader returns `None`.
///
/// The reader holds its own reference to the zipped bytes, so readers of two tables can run on two
/// threads. It checks the CRC-32 and a member shorter than its unzipped size after the last row,
/// so a caller must read to the end to know that the table is valid. If an item is an error, a row
/// before it can hold cut data. A member longer than its unzipped size fails at the first row after
/// the reader passes the size.
#[derive(Debug)]
pub struct TableReader {
    table: NotesTable,
    columns: Vec<String>,
    rows: csv::Reader<MemberStream>,
    done: bool,
}

impl TableReader {
    /// Starts to read the member, and checks its header against the expected columns.
    pub(super) fn new(
        table: NotesTable,
        member: &ZippedMember,
        columns: &[String],
    ) -> Result<Self, TableReadError> {
        let mut rows = ReaderBuilder::new()
            .delimiter(b'\t')
            .quote(b'"')
            .has_headers(false)
            .flexible(false)
            .from_reader(member.stream());
        let mut header = StringRecord::new();
        rows.read_record(&mut header)
            .map_err(|error| read_error(table, rows.get_ref(), &error))?;
        if let Some(mismatch) = rows.get_ref().overflow() {
            return Err(mismatch.for_table(table));
        }
        let actual: Vec<String> = header.iter().map(str::to_owned).collect();
        if actual != columns {
            return Err(TableReadError::UnexpectedHeader {
                table,
                expected: columns.to_vec(),
                actual,
            });
        }
        Ok(Self {
            table,
            columns: actual,
            rows,
            done: false,
        })
    }

    /// Returns the table this reader reads.
    #[must_use]
    pub const fn table(&self) -> NotesTable {
        self.table
    }

    /// Returns the columns of the header, in order.
    #[must_use]
    pub fn columns(&self) -> &[String] {
        &self.columns
    }
}

impl Iterator for TableReader {
    type Item = Result<TableRow, TableReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let mut record = StringRecord::new();
        match self.rows.read_record(&mut record) {
            Ok(true) => match self.rows.get_ref().overflow() {
                Some(mismatch) => {
                    self.done = true;
                    Some(Err(mismatch.for_table(self.table)))
                }
                None => Some(Ok(TableRow(record))),
            },
            Ok(false) => {
                self.done = true;
                self.rows
                    .get_ref()
                    .check_end()
                    .err()
                    .map(|mismatch| Err(mismatch.for_table(self.table)))
            }
            Err(error) => {
                self.done = true;
                Some(Err(read_error(self.table, self.rows.get_ref(), &error)))
            }
        }
    }
}

/// One row of a table: the text of each field, in the order of the header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow(StringRecord);

impl TableRow {
    /// Returns the field at the given index, or `None` if the row has no such field.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&str> {
        self.0.get(index)
    }

    /// Returns the number of fields.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the row has no field.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the fields, in the order of the header.
    pub fn fields(&self) -> impl Iterator<Item = &str> {
        self.0.iter()
    }
}

/// The zipped bytes of one member, and the unzipped size and the CRC-32 of its ZIP index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ZippedMember {
    pub(super) zipped: Bytes,
    pub(super) unzipped_size: u64,
    pub(super) crc32: u32,
}

impl ZippedMember {
    /// Returns a stream of the unzipped bytes, which stops one byte after the unzipped size.
    pub(super) fn stream(&self) -> MemberStream {
        MemberStream {
            inflated: DeflateDecoder::new(Cursor::new(self.zipped.clone()))
                .take(self.unzipped_size.saturating_add(1)),
            crc: Crc::new(),
            count: 0,
            unzipped_size: self.unzipped_size,
            crc32: self.crc32,
        }
    }
}

/// The unzipped bytes of one member, with their count and their CRC-32.
pub(super) struct MemberStream {
    inflated: Take<DeflateDecoder<Cursor<Bytes>>>,
    crc: Crc,
    count: u64,
    unzipped_size: u64,
    crc32: u32,
}

impl MemberStream {
    /// Returns the size mismatch if the stream has unzipped more bytes than the ZIP index records.
    const fn overflow(&self) -> Option<MemberMismatch> {
        if self.count > self.unzipped_size {
            Some(MemberMismatch::Size {
                expected: self.unzipped_size,
                actual: self.count,
            })
        } else {
            None
        }
    }

    /// Checks the count and the CRC-32 of the bytes read against the ZIP index.
    pub(super) fn check_end(&self) -> Result<(), MemberMismatch> {
        if self.count != self.unzipped_size {
            return Err(MemberMismatch::Size {
                expected: self.unzipped_size,
                actual: self.count,
            });
        }
        if self.crc.sum() != self.crc32 {
            return Err(MemberMismatch::Checksum {
                expected: self.crc32,
                actual: self.crc.sum(),
            });
        }
        Ok(())
    }
}

impl Read for MemberStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.inflated.read(buf)?;
        self.crc.update(&buf[..read]);
        self.count += read as u64;
        Ok(read)
    }
}

impl fmt::Debug for MemberStream {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("MemberStream")
            .field("count", &self.count)
            .field("unzipped_size", &self.unzipped_size)
            .field("crc32", &self.crc32)
            .finish_non_exhaustive()
    }
}

/// A difference between the unzipped bytes of a member and its ZIP index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MemberMismatch {
    Size { expected: u64, actual: u64 },
    Checksum { expected: u32, actual: u32 },
}

impl MemberMismatch {
    const fn for_table(self, table: NotesTable) -> TableReadError {
        match self {
            Self::Size { expected, actual } => TableReadError::InvalidMemberSize {
                table,
                expected,
                actual,
            },
            Self::Checksum { expected, actual } => TableReadError::InvalidMemberChecksum {
                table,
                expected,
                actual,
            },
        }
    }
}

/// Formats as `Size: <actual> bytes, Expected: <expected> bytes`, or the same for the CRC-32.
impl Display for MemberMismatch {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Size { expected, actual } => {
                write!(f, "Size: {actual} bytes, Expected: {expected} bytes")
            }
            Self::Checksum { expected, actual } => {
                write!(f, "CRC-32: {actual:08x}, Expected: {expected:08x}")
            }
        }
    }
}

/// Maps a read error of the rows. If the member unzipped to more bytes than the ZIP index records,
/// the stream stopped inside a row, so the size is the cause of the error.
fn read_error(table: NotesTable, stream: &MemberStream, error: &csv::Error) -> TableReadError {
    stream.overflow().map_or_else(
        || table_error(table, error),
        |mismatch| mismatch.for_table(table),
    )
}

fn table_error(table: NotesTable, error: &csv::Error) -> TableReadError {
    match error.kind() {
        ErrorKind::Utf8 { pos, .. } => TableReadError::InvalidUtf8 {
            table,
            line: pos.as_ref().map_or(0, csv::Position::line),
        },
        ErrorKind::UnequalLengths {
            pos,
            expected_len,
            len,
        } => TableReadError::UnexpectedFieldCount {
            table,
            line: pos.as_ref().map_or(0, csv::Position::line),
            expected: *expected_len,
            actual: *len,
        },
        _ => TableReadError::FailedTableRead {
            table,
            reason: error.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::notes_archive::NotesArchive;
    use crate::tests::fixtures::sample_notes_archive::{
        NotesZipBuilder, SAMPLE_HEADERS, SampleMember, sample_metadata,
    };

    /// The directory of the tables of the fixture filing `0001628280-26-068246` in `xbrl`.
    macro_rules! fixture_table {
        ($name:literal) => {
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../xbrl/src/tests/fixtures/data/notes/0001628280-26-068246/",
                $name
            ))
        };
    }

    fn open(builder: &NotesZipBuilder) -> NotesArchive {
        NotesArchive::open(&Bytes::from(builder.build()))
            .expect("Given a sample whose container is valid, the open should always succeed")
    }

    fn sample_with_dim(content: &[u8]) -> NotesZipBuilder {
        let mut builder = NotesZipBuilder::sample();
        builder.member_mut("dim.tsv").content = content.to_vec();
        builder
    }

    const fn implements_auto_traits<T: Sized + Send + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_table_reader() {
        implements_auto_traits::<TableReader>();
    }

    const fn implements_thread_safe_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_table_row() {
        implements_thread_safe_traits::<TableRow>();
    }

    #[test]
    fn should_read_a_quoted_field_with_a_tab_as_one_field() {
        let archive = open(&NotesZipBuilder::sample());
        let reader = archive
            .table(NotesTable::Dim)
            .expect("Given the sample dim.tsv, its header should always match");

        let expected_result: Result<Vec<Vec<&str>>, TableReadError> = Ok(vec![
            vec!["0x00000000", "", "0"],
            vec!["0x1a2b3c4d", "Axis=A\tB;", "0"],
        ]);

        let result: Result<Vec<TableRow>, TableReadError> = reader.collect();
        let result = result.as_ref().map(|rows| {
            rows.iter()
                .map(|row| row.fields().collect::<Vec<&str>>())
                .collect::<Vec<_>>()
        });
        let result = result.map_err(Clone::clone);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_header_names_as_columns_for_dim_table() {
        let archive = open(&NotesZipBuilder::sample());
        let reader = archive
            .table(NotesTable::Dim)
            .expect("Given the sample dim.tsv, its header should always match");

        let expected_result = ["dimhash", "segments", "segt"];

        let result = reader.columns();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_unexpected_header_when_the_header_differs_from_the_metadata() {
        let archive = open(&sample_with_dim(b"dimh\tsegments\tsegt\n"));

        let expected_result = Some(TableReadError::UnexpectedHeader {
            table: NotesTable::Dim,
            expected: vec![
                "dimhash".to_owned(),
                "segments".to_owned(),
                "segt".to_owned(),
            ],
            actual: vec!["dimh".to_owned(), "segments".to_owned(), "segt".to_owned()],
        });

        let result = archive.table(NotesTable::Dim).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_unexpected_header_when_the_member_is_empty() {
        let archive = open(&sample_with_dim(b""));

        let expected_result = Some(TableReadError::UnexpectedHeader {
            table: NotesTable::Dim,
            expected: vec![
                "dimhash".to_owned(),
                "segments".to_owned(),
                "segt".to_owned(),
            ],
            actual: vec![],
        });

        let result = archive.table(NotesTable::Dim).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_utf8_when_a_row_is_not_utf8() {
        let archive = open(&sample_with_dim(
            b"dimhash\tsegments\tsegt\n0x00\t\xff\t0\n",
        ));
        let mut reader = archive
            .table(NotesTable::Dim)
            .expect("Given a valid header, the start of the read should always succeed");

        let expected_result = Some(Err(TableReadError::InvalidUtf8 {
            table: NotesTable::Dim,
            line: 2,
        }));

        let result = reader.next();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_unexpected_field_count_when_a_row_has_fewer_fields_than_the_header() {
        let archive = open(&sample_with_dim(
            b"dimhash\tsegments\tsegt\n0x00\t\t0\n0x01\t0\n",
        ));
        let mut reader = archive
            .table(NotesTable::Dim)
            .expect("Given a valid header, the start of the read should always succeed");

        let expected_result = Some(TableReadError::UnexpectedFieldCount {
            table: NotesTable::Dim,
            line: 3,
            expected: 3,
            actual: 2,
        });

        let result = reader.find_map(Result::err);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_unexpected_field_count_when_a_tab_splits_an_unquoted_field() {
        let archive = open(&sample_with_dim(
            b"dimhash\tsegments\tsegt\n0x01\tAxis=A\tB;\t0\n",
        ));
        let mut reader = archive
            .table(NotesTable::Dim)
            .expect("Given a valid header, the start of the read should always succeed");

        let expected_result = Some(Err(TableReadError::UnexpectedFieldCount {
            table: NotesTable::Dim,
            line: 2,
            expected: 3,
            actual: 4,
        }));

        let result = reader.next();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_member_checksum_after_the_last_row_when_the_crc_differs() {
        let mut builder = NotesZipBuilder::sample();
        builder.member_mut("dim.tsv").crc32 = Some(0x0000_0001);
        let archive = open(&builder);
        let reader = archive
            .table(NotesTable::Dim)
            .expect("Given the sample dim.tsv, its header should always match");

        let expected_result = Some(Err(TableReadError::InvalidMemberChecksum {
            table: NotesTable::Dim,
            expected: 0x0000_0001,
            actual: 0xc081_79a8,
        }));

        let result = reader.last();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_member_size_when_the_member_unzips_to_fewer_bytes() {
        let mut builder = NotesZipBuilder::sample();
        builder.member_mut("dim.tsv").unzipped_size = Some(70);
        let archive = open(&builder);
        let reader = archive
            .table(NotesTable::Dim)
            .expect("Given the sample dim.tsv, its header should always match");

        let expected_result = Some(Err(TableReadError::InvalidMemberSize {
            table: NotesTable::Dim,
            expected: 70,
            actual: 61,
        }));

        let result = reader.last();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_cut_row_when_the_cut_falls_inside_the_last_field_of_a_row() {
        let rows = (0..1_000_u32).fold(String::new(), |mut rows, hash| {
            writeln!(rows, "0x{hash:08x}\t\t0")
                .expect("Given a String, a write should always succeed");
            rows
        });
        let content = format!("dimhash\tsegments\tsegt\n{rows}");
        let mut builder = sample_with_dim(content.as_bytes());
        // 14,022 bytes. The size cuts `0\n` from the last row, so its last field reads empty.
        builder.member_mut("dim.tsv").unzipped_size = Some(14_019);
        let archive = open(&builder);
        let reader = archive.table(NotesTable::Dim).expect(
            "Given a header in the first 8 KiB, the start of the read should always succeed",
        );

        let expected_result = vec![Err(TableReadError::InvalidMemberSize {
            table: NotesTable::Dim,
            expected: 14_019,
            actual: 14_020,
        })];

        let result: Vec<Result<TableRow, TableReadError>> = reader
            .filter(|item| !matches!(item, Ok(row) if row.get(2) == Some("0")))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_member_size_when_the_cut_falls_inside_the_header() {
        let mut builder = NotesZipBuilder::sample();
        builder.member_mut("dim.tsv").unzipped_size = Some(5);
        let archive = open(&builder);

        let expected_result = Some(TableReadError::InvalidMemberSize {
            table: NotesTable::Dim,
            expected: 5,
            actual: 6,
        });

        let result = archive.table(NotesTable::Dim).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_failed_table_read_when_the_deflate_stream_is_damaged() {
        let mut builder = NotesZipBuilder::sample();
        builder.member_mut("dim.tsv").data = Some(vec![0xff; 16]);
        let archive = open(&builder);

        let result = archive.table(NotesTable::Dim).err();

        // The reason is the text of `flate2`, which the test leaves out.
        assert!(matches!(
            result,
            Some(TableReadError::FailedTableRead {
                table: NotesTable::Dim,
                ..
            })
        ));
    }

    #[test]
    fn should_read_every_row_of_the_fixture_filing() {
        let tables = [
            ("sub.tsv", fixture_table!("sub.tsv").as_slice()),
            ("num.tsv", fixture_table!("num.tsv").as_slice()),
            ("txt.tsv", fixture_table!("txt.tsv").as_slice()),
            ("dim.tsv", fixture_table!("dim.tsv").as_slice()),
            ("tag.tsv", fixture_table!("tag.tsv").as_slice()),
        ];
        let mut members: Vec<SampleMember> = tables
            .iter()
            .map(|(name, content)| SampleMember::new(name, content))
            .collect();
        for (name, header) in SAMPLE_HEADERS {
            if !tables.iter().any(|(table, _)| *table == name) {
                members.push(SampleMember::new(name, format!("{header}\n").as_bytes()));
            }
        }
        members.push(SampleMember::new(
            "notes-metadata.json",
            sample_metadata(&SAMPLE_HEADERS).as_bytes(),
        ));
        let archive = open(&NotesZipBuilder::with_members(members));
        let read_tables = [
            NotesTable::Sub,
            NotesTable::Num,
            NotesTable::Txt,
            NotesTable::Dim,
            NotesTable::Tag,
        ];

        let expected_result = Ok(vec![
            (NotesTable::Sub, 1),
            (NotesTable::Num, 603),
            (NotesTable::Txt, 75),
            (NotesTable::Dim, 38),
            (NotesTable::Tag, 178),
        ]);

        let result: Result<Vec<(NotesTable, usize)>, TableReadError> = read_tables
            .into_iter()
            .map(|table| {
                let rows = archive.table(table)?.collect::<Result<Vec<_>, _>>()?;
                Ok((table, rows.len()))
            })
            .collect();

        assert_eq!(result, expected_result);
    }
}
