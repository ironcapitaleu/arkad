//! # Extraction Idempotency Tests
//!
//! Runs a facade call twice on the same inputs in one store, and compares the whole store after
//! each run. An equal manifest proves that the second run wrote no file.
//!
//! The second run uses another `execution_id`. A rewritten metadata file therefore differs in its
//! run fields, even when both runs share the same `produced_at`.
//!
//! ## External Dependencies
//!
//! None. A fake SEC client answers every request, and the stores write to a temp dir.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use storage::implementations::repository::filesystem::METADATA_SUFFIX;
use tempfile::TempDir;
use uuid::Uuid;

/// The `execution_id` of the first run.
const FIRST_EXECUTION_ID: Uuid = Uuid::from_u128(1);
/// The `execution_id` of the second run.
const SECOND_EXECUTION_ID: Uuid = Uuid::from_u128(2);

/// One document of a store: its bytes and its metadata file.
///
/// A field is `None` when its file is missing, so the manifest also lists a document without
/// metadata, and metadata without a document.
#[derive(Debug, PartialEq, Eq)]
struct StoredDocument {
    /// The bytes of the document file.
    bytes: Option<Vec<u8>>,
    /// The text of the metadata file, run fields included.
    metadata: Option<String>,
}

/// Every document of a store, by key.
type Manifest = BTreeMap<String, StoredDocument>;

/// Reads every file under `root` into a [`Manifest`].
///
/// The key of a document is its path relative to `root`, with `/` as separator. A metadata file
/// adds its text to the document whose key it extends by [`METADATA_SUFFIX`].
fn manifest(root: &Path) -> Manifest {
    let mut manifest = Manifest::new();
    for path in files_under(root) {
        let relative_path = path
            .strip_prefix(root)
            .expect("Given a path found under the root, stripping the root should always succeed")
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let contents = std::fs::read(&path)
            .expect("Given a file that the walk just found, reading it should always succeed");
        if let Some(key) = relative_path.strip_suffix(METADATA_SUFFIX) {
            let metadata = String::from_utf8(contents).expect(
                "Given a metadata file that the store wrote as JSON, it should always be UTF-8",
            );
            document_entry(&mut manifest, key).metadata = Some(metadata);
        } else {
            document_entry(&mut manifest, &relative_path).bytes = Some(contents);
        }
    }
    manifest
}

/// Returns the entry of `key` in `manifest`, and adds an empty one first if it is missing.
fn document_entry<'a>(manifest: &'a mut Manifest, key: &str) -> &'a mut StoredDocument {
    manifest.entry(key.to_string()).or_insert(StoredDocument {
        bytes: None,
        metadata: None,
    })
}

/// Returns the path of every file under `directory`, at any depth.
fn files_under(directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let entries = std::fs::read_dir(directory)
        .expect("Given a directory of the temp store, listing it should always succeed");
    for entry in entries {
        let path = entry
            .expect("Given a directory that lists, reading its entries should always succeed")
            .path();
        if path.is_dir() {
            files.extend(files_under(&path));
        } else {
            files.push(path);
        }
    }
    files
}

/// Returns the keys of `manifest`, in order.
fn keys(manifest: Manifest) -> Vec<String> {
    manifest.into_keys().collect()
}

/// Returns a new empty temp directory for one store.
fn temporary_root() -> TempDir {
    TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    )
}

/// Runs the daily filings of one date with a fake SEC.
mod daily_filings {
    use std::time::Duration;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use sec::implementations::extraction::{DailyFilingsRepositories, DailyRunSummary, Extraction};
    use sec::shared::raw_document::RawDocument;
    use sec::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use serde_json::json;
    use storage::implementations::repository::filesystem::FilesystemRepository;
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::{FIRST_EXECUTION_ID, SECOND_EXECUTION_ID, keys, manifest, temporary_root};

    // The routes repeat those of `daily_filings.rs` on purpose. This file reads on its own, and
    // a change to the fixture there cannot change what this file proves.
    const INDEX_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261026.idx";
    const DAILY_INDEX: &[u8] = b"Form Type   Company Name   CIK   Date Filed  File Name
---------------------------------------------------------------------
10-Q             ALCO STORES INC                                               3545        20261026    edgar/data/3545/0000003545-26-000047.txt
";
    const FOLDER_INDEX_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json";
    const FOLDER_INDEX: &[u8] = br#"{"directory":{"item":[{"name":"alco-20260908_htm.xml"},{"name":"alco-20260908_lab.xml"}]}}"#;
    const INSTANCE_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_htm.xml";
    const LABEL_FILE_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_lab.xml";
    const SUBMISSIONS_URL: &str = "https://data.sec.gov/submissions/CIK0000003545.json";
    const SUBMISSIONS: &[u8] = br#"{"cik":"3545","name":"ALCO STORES INC","filings":{"recent":{"accessionNumber":["0000003545-26-000047"],"filingDate":["2026-10-26"],"form":["10-Q"]}}}"#;

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .expect("Given a hardcoded valid date, parsing should always succeed")
    }

    /// Returns a client that serves the index of 2026-10-26 with one 10-Q, its folder, its two
    /// files, and the `submissions` file of its company. Every other URL answers 404.
    fn sec_client() -> RoutingSecClient {
        RoutingSecClient::new()
            .route(INDEX_URL, 200, "application/octet-stream", DAILY_INDEX)
            .route(FOLDER_INDEX_URL, 200, "application/json", FOLDER_INDEX)
            .route(INSTANCE_URL, 200, "text/xml", b"<xbrl/>")
            .route(LABEL_FILE_URL, 200, "text/xml", b"<labels/>")
            .route(SUBMISSIONS_URL, 200, "application/json", SUBMISSIONS)
    }

    fn repositories(root: &TempDir) -> DailyFilingsRepositories {
        DailyFilingsRepositories {
            daily_index: FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<_>>()
                .root(root.path())
                .build(),
            filing_index: FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<_>>()
                .root(root.path())
                .build(),
            instance: FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<_>>()
                .root(root.path())
                .build(),
            labels: FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<_>>()
                .root(root.path())
                .build(),
            submissions: FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<_>>()
                .root(root.path())
                .build(),
        }
    }

    /// Runs the date 2026-10-26 into `root` with `execution_id`. The run treats 2026-10-28 as
    /// yesterday.
    async fn run(root: &TempDir, execution_id: Uuid) -> DailyRunSummary {
        Extraction::incremental()
            .execution_id(execution_id)
            .daily_filings()
            .dates([date("2026-10-26")])
            .yesterday(date("2026-10-28"))
            .sec_client(sec_client())
            .repositories(repositories(root))
            .retry_delays([Duration::ZERO, Duration::ZERO])
            .run(|_, _| {})
            .await
    }

    #[tokio::test]
    async fn should_store_every_document_of_the_date_when_the_first_run_ends() {
        let root = temporary_root();
        let _ = run(&root, FIRST_EXECUTION_ID).await;

        let expected_result = vec![
            "sec/daily-index/form.20261026.idx",
            "sec/filings/CIK0000003545/0000003545-26-000047/index.json",
            "sec/filings/CIK0000003545/0000003545-26-000047/instance.xml",
            "sec/filings/CIK0000003545/0000003545-26-000047/lab.xml",
            "sec/submissions/CIK0000003545.json",
        ];

        let result = keys(manifest(root.path()));

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_skip_the_filing_and_fail_nothing_when_the_same_date_runs_again() {
        let root = temporary_root();
        let _ = run(&root, FIRST_EXECUTION_ID).await;

        let expected_result = json!({
            "date": "2026-10-26",
            "detection": "processed",
            "filings": {
                "selected": 1,
                "stored": 0,
                "already_stored": 1,
                "skipped_unextractable": 0,
                "unextractable": [],
                "failed": [],
            },
            "submissions": { "companies": 1, "stored": 1, "failed": [] },
        });

        let result = run(&root, SECOND_EXECUTION_ID).await.entries()[0].clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_the_store_unchanged_when_the_same_date_runs_again() {
        let root = temporary_root();
        let _ = run(&root, FIRST_EXECUTION_ID).await;
        let first_manifest = manifest(root.path());

        let expected_result = first_manifest;

        let _ = run(&root, SECOND_EXECUTION_ID).await;
        let result = manifest(root.path());

        assert_eq!(result, expected_result);
    }
}

/// Runs the Notes backfill of one period with a fake SEC.
mod notes {
    use pretty_assertions::assert_eq;
    use sec::implementations::extraction::{Extraction, NotesRunSummary};
    use sec::shared::raw_document::RawDocument;
    use sec::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use sec::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use storage::implementations::repository::filesystem::FilesystemRepository;
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::{FIRST_EXECUTION_ID, SECOND_EXECUTION_ID, keys, manifest, temporary_root};

    const PERIOD: &str = "2026_09";
    const NOTES_URL: &str =
        "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2026_09_notes.zip";

    /// Returns a client that serves the Notes file of [`PERIOD`]. Every other URL answers 404.
    fn sec_client() -> RoutingSecClient {
        RoutingSecClient::new().route(NOTES_URL, 200, "application/zip", SAMPLE_NOTES_ZIP)
    }

    /// Runs the Notes backfill of [`PERIOD`] into `root` with `execution_id`.
    async fn run(root: &TempDir, execution_id: Uuid) -> NotesRunSummary {
        Extraction::backfill()
            .execution_id(execution_id)
            .notes()
            .periods([PERIOD])
            .sec_client(sec_client())
            .repository(
                FilesystemRepository::builder()
                    .writer()
                    .document_type::<RawDocument<_>>()
                    .root(root.path())
                    .build(),
            )
            .run(|_, _| {})
            .await
    }

    #[tokio::test]
    async fn should_store_the_notes_file_of_the_period_when_the_first_run_ends() {
        let root = temporary_root();
        let _ = run(&root, FIRST_EXECUTION_ID).await;

        let expected_result = vec!["sec/notes/2026_09_notes.zip"];

        let result = keys(manifest(root.path()));

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_count_the_period_as_already_stored_when_the_same_period_runs_again() {
        let root = temporary_root();
        let _ = run(&root, FIRST_EXECUTION_ID).await;

        let expected_result = 1;

        let result = run(&root, SECOND_EXECUTION_ID).await.already_stored_count();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_the_store_unchanged_when_the_same_period_runs_again() {
        let root = temporary_root();
        let _ = run(&root, FIRST_EXECUTION_ID).await;
        let first_manifest = manifest(root.path());

        let expected_result = first_manifest;

        let _ = run(&root, SECOND_EXECUTION_ID).await;
        let result = manifest(root.path());

        assert_eq!(result, expected_result);
    }
}
