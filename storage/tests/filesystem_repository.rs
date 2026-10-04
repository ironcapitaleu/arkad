//! # Filesystem Repository Integration Tests
//!
//! Persists a record through [`FilesystemRepository`] and reads the same record from a real
//! directory on disk.
//!
//! ## External Dependencies
//!
//! - A writable system temp directory. Each test creates its own root there and deletes it at the
//!   end.

use chrono::{TimeZone, Utc};
use pretty_assertions::assert_eq;
use tempfile::TempDir;

use storage::implementations::repository::filesystem::{
    DocumentKey, DocumentMetadata, FilesystemRepository, RawDocument,
};
use storage::{ReadRepository, WriteRepository};

fn sample_raw_document() -> RawDocument {
    let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
        .expect("Given a valid relative path, the key should always build");
    let metadata = DocumentMetadata {
        metadata_version: 1,
        url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
        fetched_at: Utc
            .with_ymd_and_hms(2026, 9, 22, 20, 5, 30)
            .single()
            .expect("Given a hardcoded valid timestamp, the conversion should always succeed"),
        http_status: 200,
        user_agent: "arkad contact@example.com".to_owned(),
        bytes: 41,
        sha256: "9d7e13ec27ff57b43422959a9ae627fd5bd65e67b54af2857e9c454a9262376b".to_owned(),
    };

    RawDocument::new(
        key,
        br#"{"cik":320193, "n":1.50, "dup":1,"dup":2}"#.to_vec(),
        metadata,
    )
}

#[tokio::test]
async fn should_return_the_same_record_when_reading_back_a_persisted_record_by_key() {
    let root = TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    );
    let repository = FilesystemRepository::new(root.path());
    let record = sample_raw_document();
    let key = record.key().clone();

    let expected_result = Some(record.clone());

    repository
        .persist(record)
        .await
        .expect("Given an existing writable root, persisting a record should always succeed");
    let result = repository
        .get(key)
        .await
        .expect("Given a record persisted under this key, reading it back should always succeed");

    assert_eq!(result, expected_result);
}
