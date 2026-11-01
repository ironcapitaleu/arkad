//! # `backfill` Tests
//!
//! Runs the tests of the modules of the `backfill` binary, and its two facade calls offline
//! against a fake SEC. The binary is its own crate root, so this test compiles its modules as
//! private modules.
//!
//! ## External Dependencies
//!
//! None. A fake SEC client answers every request, and the stores write to a temp dir.

#[path = "../src/bin/shared/files_in_flight.rs"]
mod files_in_flight;
#[path = "../src/bin/shared/notes_periods.rs"]
mod notes_periods;
// The tests read only the summary of the catch-up, so its dates would fire `dead_code`.
#[allow(dead_code)]
#[path = "../src/bin/backfill/run.rs"]
mod run;

/// Runs both facade calls offline, with a fake SEC client and stores in a temp dir.
mod two_calls {
    use std::sync::{Arc, Mutex};

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use sec::implementations::extraction::{
        DailyFilingsRepositories, Extraction, FailedBackfill, MissingCutOverDate, RunEvent, Unit,
    };
    use sec::shared::raw_document::RawDocument;
    use sec::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use sec::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use serde_json::{Value, json};
    use storage::implementations::repository::filesystem::FilesystemRepository;
    use tempfile::TempDir;
    use uuid::Uuid;

    use crate::run::{BackfillInputs, CatchUp, run};

    const NOTES_URL: &str =
        "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/";
    const INDEX_URL_OF_2026_09_30: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR3/form.20260930.idx";
    const INDEX_URL_OF_2026_10_01: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261001.idx";
    /// A daily index that lists no filing.
    const EMPTY_DAILY_INDEX: &[u8] = b"Form Type   Company Name   CIK   Date Filed  File Name
---------------------------------------------------------------------
";
    const LOCK_EXPECT: &str = "Given an observer that never panics while it holds the lock, the lock should never be poisoned";

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .expect("Given a hardcoded valid date, parsing should always succeed")
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    /// Returns a client that serves the Notes file of `2026_09` and an empty daily index of
    /// 2026-09-30 and of 2026-10-01. Every other URL answers 404.
    fn sec_client() -> RoutingSecClient {
        RoutingSecClient::new()
            .route(
                &format!("{NOTES_URL}2026_09_notes.zip"),
                200,
                "application/zip",
                SAMPLE_NOTES_ZIP,
            )
            .route(
                INDEX_URL_OF_2026_09_30,
                200,
                "application/octet-stream",
                EMPTY_DAILY_INDEX,
            )
            .route(
                INDEX_URL_OF_2026_10_01,
                200,
                "application/octet-stream",
                EMPTY_DAILY_INDEX,
            )
    }

    /// Returns the inputs of a run over `periods` into `root`, with yesterday on 2026-10-01 and
    /// the `execution_id` 7.
    fn inputs(root: &TempDir, periods: &[&str]) -> BackfillInputs<RoutingSecClient> {
        BackfillInputs {
            backfill: Extraction::backfill().execution_id(Uuid::from_u128(7)),
            periods: periods.iter().map(ToString::to_string).collect(),
            files_in_flight: 2,
            yesterday: date("2026-10-01"),
            sec_client: sec_client(),
            notes_repository: FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<_>>()
                .root(root.path())
                .build(),
            daily_repositories: DailyFilingsRepositories {
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
            },
        }
    }

    /// Runs `inputs` without an observer.
    async fn run_quietly(
        inputs: BackfillInputs<RoutingSecClient>,
    ) -> Result<CatchUp, MissingCutOverDate> {
        run(inputs, |_, _| {}, |_| {}).await
    }

    /// Runs `inputs`, and returns each unit of an event that is not a Notes period, in order.
    async fn catch_up_units(inputs: BackfillInputs<RoutingSecClient>) -> Vec<String> {
        let units = Arc::new(Mutex::new(Vec::new()));
        let observed_units = Arc::clone(&units);
        let observer = move |unit: &Unit, _: RunEvent<'_>| {
            if !matches!(unit, Unit::Period(_)) {
                observed_units
                    .lock()
                    .expect(LOCK_EXPECT)
                    .push(unit.to_string());
            }
        };
        let _ = run(inputs, observer, |_| {}).await;
        units.lock().expect(LOCK_EXPECT).clone()
    }

    /// Reads the metadata file at `file` under `root` as JSON.
    fn stored_metadata(root: &TempDir, file: &str) -> Value {
        serde_json::from_slice(
            &std::fs::read(root.path().join(file)).expect(
                "Given a run that stored the document, its metadata file should always exist",
            ),
        )
        .expect("Given a metadata file that the store wrote, it should always parse as JSON")
    }

    #[tokio::test]
    async fn should_start_the_catch_up_on_the_last_day_of_the_newest_stored_month_when_the_notes_run_succeeds()
     {
        let root = temporary_root();
        let inputs = inputs(&root, &["2026_09"]);

        let expected_result = json!(["2026-09-30", "2026-10-01"]);

        let catch_up = run_quietly(inputs)
            .await
            .expect("Given a stored Notes period and no failed period, the cut-over date should always exist");
        let result: Value = catch_up
            .summary
            .entries()
            .iter()
            .map(|entry| entry["date"].clone())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_one_execution_id_for_the_notes_file_and_the_daily_index_when_both_calls_run()
     {
        let root = temporary_root();
        let inputs = inputs(&root, &["2026_09"]);
        let metadata_files = [
            "sec/notes/2026_09_notes.zip.meta.json",
            "sec/daily-index/form.20260930.idx.meta.json",
        ];

        let expected_result = json!([
            "00000000-0000-0000-0000-000000000007",
            "00000000-0000-0000-0000-000000000007",
        ]);

        run_quietly(inputs)
            .await
            .expect("Given a stored Notes period and no failed period, the cut-over date should always exist");
        let result: Value = metadata_files
            .iter()
            .map(|file| stored_metadata(&root, file)["execution_id"].clone())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_cut_over_date_when_a_notes_period_fails() {
        let root = temporary_root();
        let inputs = inputs(&root, &["2026_08", "2026_09"]);

        let expected_result = Err(MissingCutOverDate::FailedBackfill(FailedBackfill {
            failures: 1,
            periods: 2,
        }));

        let result = run_quietly(inputs).await.map(|_| ());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_run_no_catch_up_date_when_a_notes_period_fails() {
        let root = temporary_root();
        let inputs = inputs(&root, &["2026_08", "2026_09"]);

        let expected_result: Vec<String> = Vec::new();

        let result = catch_up_units(inputs).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_stored_period_when_no_notes_period_is_stored() {
        let root = temporary_root();
        let inputs = inputs(&root, &["2026_10"]);

        let expected_result = Err(MissingCutOverDate::MissingStoredPeriod);

        let result = run_quietly(inputs).await.map(|_| ());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_run_no_catch_up_date_when_no_notes_period_is_stored() {
        let root = temporary_root();
        let inputs = inputs(&root, &["2026_10"]);

        let expected_result: Vec<String> = Vec::new();

        let result = catch_up_units(inputs).await;

        assert_eq!(result, expected_result);
    }
}
