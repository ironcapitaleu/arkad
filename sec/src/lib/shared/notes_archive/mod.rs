//! # Notes Archive
//!
//! Provides [`NotesArchive`], one Notes file opened after its container checks, and
//! [`TableReader`], which reads one of its tables after the table format checks.
//!
//! A Notes file is a ZIP of eight tab-separated tables, a readme, and a metadata member that lists
//! the columns of each table. A damaged file fails as a whole: every error of this module fails
//! the Notes file, not one filing.
//!
//! ## Modules
//!
//! - [`constants`]: The ZIP signature, the names of the metadata member, the size limits, and the
//!   header aliases.
//! - [`notes_table`]: The [`NotesTable`] enum of the eight tables.
//! - [`table_reader`]: The [`TableReader`] of one table and its [`TableRow`].
//! - [`notes_archive_error`]: The [`NotesArchiveError`] of a read or an open.
//! - [`table_read_error`]: The [`TableReadError`] of a table read.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use bytes::Bytes;
use serde::Deserialize;
use storage::ReadRepository;
use storage::implementations::repository::filesystem::DocumentKey;
use zip::{CompressionMethod, ZipArchive};

use crate::shared::notes_archive_ref::NotesArchiveRef;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::RawNotesArchive;

pub mod constants;
pub mod notes_archive_error;
pub mod notes_table;
pub mod table_read_error;
pub mod table_reader;

pub use constants::{
    HEADER_ALIASES, MAX_COMPRESSION_RATIO, MAX_UNZIPPED_ARCHIVE_SIZE, METADATA_MEMBER,
    PERIOD_METADATA_MEMBER_SUFFIX, ZIP_SIGNATURE,
};
pub use notes_archive_error::NotesArchiveError;
pub use notes_table::NotesTable;
pub use table_read_error::TableReadError;
pub use table_reader::{TableReader, TableRow};

use table_reader::ZippedMember;

/// One Notes file whose ZIP container passed its checks, with the expected header of each table.
///
/// [`NotesArchive::open`] checks the container: the ZIP signature, the ZIP index, the eight tables
/// and exactly one metadata member, deflate for every member, no path separator in a member name,
/// and the size limits. [`NotesArchive::table`] then reads one table with the table format checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotesArchive {
    tables: BTreeMap<NotesTable, ArchiveTable>,
}

/// The zipped member of one table and the columns its header must hold.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ArchiveTable {
    member: ZippedMember,
    columns: Vec<String>,
}

/// One member of the ZIP index, after the checks of its name, its compression, and its size.
struct IndexEntry {
    name: String,
    member: ZippedMember,
}

impl NotesArchive {
    /// Reads the Notes file that `archive` names from the repository, and opens it.
    ///
    /// The repository checks the size and the SHA-256 of the bytes against their metadata.
    ///
    /// # Errors
    ///
    /// Returns [`NotesArchiveError::FailedArchiveRead`] if the read fails,
    /// [`NotesArchiveError::MissingArchive`] if no file is stored under the key, and every error of
    /// [`NotesArchive::open`].
    pub async fn read<R>(
        repository: &R,
        archive: &NotesArchiveRef,
    ) -> Result<Self, NotesArchiveError>
    where
        R: ReadRepository<Record = RawDocument<RawNotesArchive>, Key = DocumentKey>,
    {
        let document = repository
            .get(archive.key().clone())
            .await
            .map_err(NotesArchiveError::FailedArchiveRead)?
            .ok_or_else(|| NotesArchiveError::MissingArchive {
                key: archive.key().as_path().display().to_string(),
            })?;
        Self::open(&document.into_bytes())
    }

    /// Opens the bytes of a Notes file, checks its container, and reads the columns of each table
    /// from its metadata member.
    ///
    /// # Errors
    ///
    /// Returns a [`NotesArchiveError`] if a container check fails:
    /// [`MissingZipSignature`](NotesArchiveError::MissingZipSignature),
    /// [`InvalidZipIndex`](NotesArchiveError::InvalidZipIndex),
    /// [`InvalidMemberName`](NotesArchiveError::InvalidMemberName),
    /// [`UnexpectedCompression`](NotesArchiveError::UnexpectedCompression),
    /// [`OversizedMember`](NotesArchiveError::OversizedMember),
    /// [`OversizedArchive`](NotesArchiveError::OversizedArchive),
    /// [`MissingMember`](NotesArchiveError::MissingMember),
    /// [`UnexpectedMetadataMemberCount`](NotesArchiveError::UnexpectedMetadataMemberCount), or
    /// [`InvalidMetadata`](NotesArchiveError::InvalidMetadata).
    pub fn open(bytes: &Bytes) -> Result<Self, NotesArchiveError> {
        open_with_limit(bytes, MAX_UNZIPPED_ARCHIVE_SIZE)
    }

    /// Starts a [`TableReader`] of one table, and checks its header against the columns of the
    /// metadata member.
    ///
    /// # Errors
    ///
    /// Returns [`TableReadError::UnexpectedHeader`] if the header differs from the columns, and
    /// the other variants of [`TableReadError`] if the header row cannot be read.
    pub fn table(&self, table: NotesTable) -> Result<TableReader, TableReadError> {
        let archive_table = &self.tables[&table];
        TableReader::new(table, &archive_table.member, &archive_table.columns)
    }
}

fn open_with_limit(bytes: &Bytes, limit: u64) -> Result<NotesArchive, NotesArchiveError> {
    if !bytes.starts_with(ZIP_SIGNATURE) {
        return Err(NotesArchiveError::MissingZipSignature);
    }
    let entries = index_entries(bytes)?;
    let unzipped_size = entries.iter().fold(0_u64, |sum, entry| {
        sum.saturating_add(entry.member.unzipped_size)
    });
    if unzipped_size > limit {
        return Err(NotesArchiveError::OversizedArchive {
            unzipped_size,
            limit,
        });
    }
    let metadata = metadata_member(&entries)?;
    let mut columns = metadata_columns(metadata)?;
    let mut tables = BTreeMap::new();
    for table in NotesTable::ALL {
        let member = entries
            .iter()
            .find(|entry| entry.name == table.member_name())
            .ok_or_else(|| NotesArchiveError::MissingMember {
                member: table.member_name().to_owned(),
            })?
            .member
            .clone();
        let columns = columns.remove(table.member_name()).ok_or_else(|| {
            NotesArchiveError::InvalidMetadata {
                reason: format!("No columns for the table '{table}'"),
            }
        })?;
        tables.insert(
            table,
            ArchiveTable {
                member,
                columns: header_of(table, columns),
            },
        );
    }
    Ok(NotesArchive { tables })
}

/// Parses the ZIP index, and checks the name, the compression, and the size of each member.
fn index_entries(bytes: &Bytes) -> Result<Vec<IndexEntry>, NotesArchiveError> {
    let invalid_index = |error: zip::result::ZipError| NotesArchiveError::InvalidZipIndex {
        reason: error.to_string(),
    };
    let mut archive = ZipArchive::new(Cursor::new(bytes.clone())).map_err(invalid_index)?;
    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let file = archive.by_index_raw(index).map_err(invalid_index)?;
        let name = file.name().map_err(invalid_index)?.into_owned();
        if name.contains(['/', '\\']) {
            return Err(NotesArchiveError::InvalidMemberName { member: name });
        }
        if file.compression() != CompressionMethod::Deflated {
            return Err(NotesArchiveError::UnexpectedCompression { member: name });
        }
        let zipped_size = file.compressed_size();
        let unzipped_size = file.size();
        if unzipped_size > zipped_size.saturating_mul(MAX_COMPRESSION_RATIO) {
            return Err(NotesArchiveError::OversizedMember {
                member: name,
                zipped_size,
                unzipped_size,
                max_ratio: MAX_COMPRESSION_RATIO,
            });
        }
        let zipped = file
            .data_start()
            .and_then(|start| {
                let start = usize::try_from(start).ok()?;
                let end = start.checked_add(usize::try_from(zipped_size).ok()?)?;
                (end <= bytes.len()).then(|| bytes.slice(start..end))
            })
            .ok_or_else(|| NotesArchiveError::InvalidZipIndex {
                reason: format!("The data of the member '{name}' lies outside the file"),
            })?;
        entries.push(IndexEntry {
            member: ZippedMember {
                zipped,
                unzipped_size,
                crc32: file.crc32(),
            },
            name,
        });
    }
    Ok(entries)
}

/// Returns the only metadata member, named `notes-metadata.json` or `<period>_notes-metadata.json`.
fn metadata_member(entries: &[IndexEntry]) -> Result<&ZippedMember, NotesArchiveError> {
    let metadata: Vec<&IndexEntry> = entries
        .iter()
        .filter(|entry| {
            entry.name == METADATA_MEMBER || entry.name.ends_with(PERIOD_METADATA_MEMBER_SUFFIX)
        })
        .collect();
    match metadata.as_slice() {
        [entry] => Ok(&entry.member),
        _ => Err(NotesArchiveError::UnexpectedMetadataMemberCount {
            count: metadata.len(),
        }),
    }
}

/// The parts of a CSVW metadata member that name the columns of each table.
#[derive(Deserialize)]
struct Metadata {
    tables: Vec<MetadataTable>,
}

#[derive(Deserialize)]
struct MetadataTable {
    url: String,
    #[serde(rename = "tableSchema")]
    table_schema: TableSchema,
}

#[derive(Deserialize)]
struct TableSchema {
    columns: Vec<MetadataColumn>,
}

#[derive(Deserialize)]
struct MetadataColumn {
    name: String,
}

/// Unzips the metadata member, and returns the column names of each table by its member name.
fn metadata_columns(
    member: &ZippedMember,
) -> Result<BTreeMap<String, Vec<String>>, NotesArchiveError> {
    let invalid = |reason: String| NotesArchiveError::InvalidMetadata { reason };
    let mut stream = member.stream();
    let mut text = Vec::new();
    stream
        .read_to_end(&mut text)
        .map_err(|error| invalid(error.to_string()))?;
    stream
        .check_end()
        .map_err(|mismatch| invalid(mismatch.to_string()))?;
    let metadata: Metadata =
        serde_json::from_slice(&text).map_err(|error| invalid(error.to_string()))?;
    Ok(metadata
        .tables
        .into_iter()
        .map(|table| {
            let columns = table
                .table_schema
                .columns
                .into_iter()
                .map(|column| column.name)
                .collect();
            (table.url, columns)
        })
        .collect())
}

/// Replaces each column name of the metadata member that the header writes under another name.
fn header_of(table: NotesTable, columns: Vec<String>) -> Vec<String> {
    columns
        .into_iter()
        .map(|column| {
            HEADER_ALIASES
                .iter()
                .find(|(alias_table, name, _)| *alias_table == table && *name == column)
                .map_or(column, |(_, _, header)| (*header).to_owned())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use jiff::Timestamp;
    use pretty_assertions::assert_eq;
    use storage::implementations::repository::filesystem::{
        Document, DocumentMetadata, IngestionMode, MediaType, Producer,
    };
    use storage::{ReadError, Sha256Digest};
    use uuid::Uuid;

    use super::*;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::raw_document::RawDocumentMetadataContext;
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_notes_archive::{
        NotesZipBuilder, SAMPLE_HEADERS, SampleCompression, SampleMember, sample_metadata,
    };

    /// A repository that returns one fixed read result for every key.
    struct FakeNotesStore(Result<Option<Vec<u8>>, ReadError>);

    #[async_trait]
    impl ReadRepository for FakeNotesStore {
        type Record = RawDocument<RawNotesArchive>;
        type Key = DocumentKey;

        async fn get(&self, key: DocumentKey) -> Result<Option<Self::Record>, ReadError> {
            self.0.clone().map(|bytes| {
                bytes.map(|bytes| RawDocument::from_parts(key, bytes, sample_document_metadata()))
            })
        }
    }

    fn sample_document_metadata() -> DocumentMetadata<RawDocumentMetadataContext> {
        let url: Url = "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2024q1_notes.zip"
            .parse()
            .expect("A hardcoded valid URL string should always parse");
        DocumentMetadata::builder()
            .document_type::<RawDocument<RawNotesArchive>>()
            .media_type(
                MediaType::new("application/zip")
                    .expect("Given a hardcoded valid media type, creation should always succeed"),
            )
            .content(b"PK")
            .produced_at(Timestamp::UNIX_EPOCH)
            .run(
                Uuid::nil(),
                Producer::new("Store Notes Archive", "0.1.0", "63c1f59"),
            )
            .ingestion_mode(IngestionMode::Backfill)
            .context(RawDocumentMetadataContext {
                requested_url: url.clone(),
                final_url: url,
                http_status: StatusCode::Ok,
                user_agent: UserAgent::new("arkad contact@example.com")
                    .expect("A hardcoded SEC-format string should always validate"),
                filing: None,
            })
            .build()
    }

    fn sample_archive_ref() -> NotesArchiveRef {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed");
        let key = DocumentKey::new("sec/notes/2024q1_notes.zip")
            .expect("Given a hardcoded valid relative path, the key should always build");
        NotesArchiveRef::new(period, key, 2, Sha256Digest::of(b"PK"))
    }

    fn open_error(builder: &NotesZipBuilder) -> Option<NotesArchiveError> {
        NotesArchive::open(&Bytes::from(builder.build())).err()
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_notes_archive() {
        implements_auto_traits::<NotesArchive>();
    }

    #[test]
    fn should_open_the_archive_when_every_container_check_passes() {
        let bytes = Bytes::from(NotesZipBuilder::sample().build());

        let expected_result = None;

        let result = NotesArchive::open(&bytes).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_missing_zip_signature_when_the_bytes_are_not_a_zip() {
        let bytes = Bytes::from_static(b"<html>Not a ZIP</html>");

        let expected_result = Some(NotesArchiveError::MissingZipSignature);

        let result = NotesArchive::open(&bytes).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_zip_index_when_the_zip_index_does_not_parse() {
        let bytes = Bytes::from_static(b"PK\x03\x04 cut after the signature");

        let result = NotesArchive::open(&bytes).err();

        // The reason is the text of the `zip` crate, which the test leaves out.
        assert!(matches!(
            result,
            Some(NotesArchiveError::InvalidZipIndex { .. })
        ));
    }

    #[test]
    fn should_fail_with_missing_member_when_a_table_is_missing() {
        let mut builder = NotesZipBuilder::sample();
        builder
            .members_mut()
            .retain(|member| member.name != "pre.tsv");

        let expected_result = Some(NotesArchiveError::MissingMember {
            member: "pre.tsv".to_owned(),
        });

        let result = open_error(&builder);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_unexpected_metadata_member_count_when_no_metadata_member_exists() {
        let mut builder = NotesZipBuilder::sample();
        builder
            .members_mut()
            .retain(|member| member.name != "notes-metadata.json");

        let expected_result = Some(NotesArchiveError::UnexpectedMetadataMemberCount { count: 0 });

        let result = open_error(&builder);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_unexpected_metadata_member_count_when_two_metadata_members_exist() {
        let mut builder = NotesZipBuilder::sample();
        let metadata = sample_metadata(&SAMPLE_HEADERS);
        builder.members_mut().push(SampleMember::new(
            "2009q1_notes-metadata.json",
            metadata.as_bytes(),
        ));

        let expected_result = Some(NotesArchiveError::UnexpectedMetadataMemberCount { count: 2 });

        let result = open_error(&builder);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_open_the_archive_when_the_metadata_member_starts_with_its_period() {
        let mut builder = NotesZipBuilder::sample();
        builder.member_mut("notes-metadata.json").name = "2009q1_notes-metadata.json".to_owned();
        let bytes = Bytes::from(builder.build());

        let expected_result = None;

        let result = NotesArchive::open(&bytes).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_unexpected_compression_when_a_member_is_stored() {
        let mut builder = NotesZipBuilder::sample();
        builder.member_mut("readme.htm").compression = SampleCompression::Stored;

        let expected_result = Some(NotesArchiveError::UnexpectedCompression {
            member: "readme.htm".to_owned(),
        });

        let result = open_error(&builder);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_member_name_when_a_name_holds_a_slash() {
        let mut builder = NotesZipBuilder::sample();
        builder.member_mut("readme.htm").name = "docs/readme.htm".to_owned();

        let expected_result = Some(NotesArchiveError::InvalidMemberName {
            member: "docs/readme.htm".to_owned(),
        });

        let result = open_error(&builder);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_member_name_when_a_name_holds_a_backslash() {
        let mut builder = NotesZipBuilder::sample();
        builder.member_mut("readme.htm").name = "docs\\readme.htm".to_owned();

        let expected_result = Some(NotesArchiveError::InvalidMemberName {
            member: "docs\\readme.htm".to_owned(),
        });

        let result = open_error(&builder);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_oversized_member_when_a_member_unzips_to_more_than_the_ratio() {
        let mut builder = NotesZipBuilder::sample();
        builder
            .members_mut()
            .push(SampleMember::new("padding.bin", &[0; 10_000]));

        let result = open_error(&builder);

        // The zipped size depends on the deflate encoder, so the test leaves the sizes out.
        assert!(matches!(
            result,
            Some(NotesArchiveError::OversizedMember { member, unzipped_size: 10_000, max_ratio: 50, .. })
                if member == "padding.bin"
        ));
    }

    #[test]
    fn should_fail_with_oversized_archive_when_the_members_unzip_to_more_than_the_limit() {
        let mut builder = NotesZipBuilder::sample();
        builder
            .members_mut()
            .retain(|member| member.name != "readme.htm");
        let bytes = Bytes::from(builder.build());
        let limit = 100;

        let result = open_with_limit(&bytes, limit).err();

        // The metadata member makes up most of the sum, so the test leaves the sum out.
        assert!(matches!(
            result,
            Some(NotesArchiveError::OversizedArchive { limit: 100, .. })
        ));
    }

    #[test]
    fn should_fail_with_invalid_metadata_when_the_metadata_member_is_not_json() {
        let mut builder = NotesZipBuilder::sample();
        builder.member_mut("notes-metadata.json").content = b"not json".to_vec();

        let result = open_error(&builder);

        // The reason is the text of `serde_json`, which the test leaves out.
        assert!(matches!(
            result,
            Some(NotesArchiveError::InvalidMetadata { .. })
        ));
    }

    #[test]
    fn should_fail_with_invalid_metadata_when_the_metadata_member_lists_no_columns_of_a_table() {
        let mut builder = NotesZipBuilder::sample();
        let headers: Vec<(&str, &str)> = SAMPLE_HEADERS
            .into_iter()
            .filter(|(table, _)| *table != "cal.tsv")
            .collect();
        builder.member_mut("notes-metadata.json").content = sample_metadata(&headers).into_bytes();

        let expected_result = Some(NotesArchiveError::InvalidMetadata {
            reason: "No columns for the table 'cal.tsv'".to_owned(),
        });

        let result = open_error(&builder);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_metadata_when_the_metadata_member_has_another_checksum() {
        let mut builder = NotesZipBuilder::sample();
        builder.member_mut("notes-metadata.json").crc32 = Some(0);

        let result = open_error(&builder);

        // The CRC-32 of the metadata member depends on all sample headers, so the test leaves it
        // out.
        assert!(matches!(
            result,
            Some(NotesArchiveError::InvalidMetadata { reason })
                if reason.starts_with("CRC-32: ") && reason.ends_with(", Expected: 00000000")
        ));
    }

    #[test]
    fn should_fail_with_invalid_zip_index_when_the_member_data_lies_outside_the_file() {
        let mut builder = NotesZipBuilder::sample();
        builder.member_mut("notes-metadata.json").zipped_size = Some(1_000_000);

        let expected_result = Some(NotesArchiveError::InvalidZipIndex {
            reason: "The data of the member 'notes-metadata.json' lies outside the file".to_owned(),
        });

        let result = open_error(&builder);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_metadata_when_the_metadata_member_has_another_size() {
        let mut builder = NotesZipBuilder::sample();
        let metadata = builder.member_mut("notes-metadata.json");
        metadata.unzipped_size = Some(10);

        let result = open_error(&builder);

        // The size of the metadata member depends on all sample headers, so the test leaves the
        // count of bytes out.
        assert!(matches!(
            result,
            Some(NotesArchiveError::InvalidMetadata { reason })
                if reason == "Size: 11 bytes, Expected: 10 bytes"
        ));
    }

    #[test]
    fn should_replace_the_metadata_name_with_the_header_name_for_the_dim_hash_column() {
        let columns = vec!["dimh".to_owned(), "segments".to_owned(), "segt".to_owned()];

        let expected_result = vec![
            "dimhash".to_owned(),
            "segments".to_owned(),
            "segt".to_owned(),
        ];

        let result = header_of(NotesTable::Dim, columns);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_metadata_name_when_another_table_has_a_column_of_the_alias_name() {
        let columns = vec!["adsh".to_owned(), "dimh".to_owned()];

        let expected_result = vec!["adsh".to_owned(), "dimh".to_owned()];

        let result = header_of(NotesTable::Num, columns);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_open_the_stored_archive_when_the_repository_holds_it() {
        let repository = FakeNotesStore(Ok(Some(NotesZipBuilder::sample().build())));
        let archive = sample_archive_ref();

        let expected_result = None;

        let result = NotesArchive::read(&repository, &archive).await.err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_with_missing_archive_when_the_repository_holds_no_file() {
        let repository = FakeNotesStore(Ok(None));
        let archive = sample_archive_ref();

        let expected_result = Some(NotesArchiveError::MissingArchive {
            key: "sec/notes/2024q1_notes.zip".to_owned(),
        });

        let result = NotesArchive::read(&repository, &archive).await.err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_with_failed_archive_read_when_the_repository_read_fails() {
        let read_error = ReadError::missing_record("sec/notes/2024q1_notes.zip");
        let repository = FakeNotesStore(Err(read_error.clone()));
        let archive = sample_archive_ref();

        let expected_result = Some(NotesArchiveError::FailedArchiveRead(read_error));

        let result = NotesArchive::read(&repository, &archive).await.err();

        assert_eq!(result, expected_result);
    }
}
