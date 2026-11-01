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
    use sec::tests::fixtures::data::notes_archive::notes_zip;
    use sec::tests::fixtures::data::submissions::SAMPLE_SUBMISSIONS_JSON;
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
    const SUBMISSIONS_URL_OF_320193: &str = "https://data.sec.gov/submissions/CIK0000320193.json";
    /// The `sub.tsv` of the Notes file of `2026_09`, with one filing of CIK `320193`.
    /// A `sub.tsv` of `2026_09` with CIK `320193` and CIK `789019`. The client of [`sec_client`]
    /// has no `submissions` file of `789019`, so its machine fails with 404.
    const SUB_TSV_WITH_A_CIK_WITHOUT_SUBMISSIONS: &str =
        "adsh\tcik\n0000320193-26-000071\t320193\n0000789019-26-000072\t789019\n";
    const SUB_TSV_OF_2026_09: &str = "adsh\tcik\tname\n0000320193-26-000071\t320193\tAPPLE INC\n";
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

    /// Returns a client that serves the Notes file of `2026_09`, the `submissions` file of CIK
    /// `320193`, and an empty daily index of 2026-09-30 and of 2026-10-01. Every other URL answers
    /// 404.
    fn sec_client() -> RoutingSecClient {
        sec_client_with_sub_tsv(SUB_TSV_OF_2026_09)
    }

    /// Returns the client of [`sec_client`], with a Notes file of `2026_09` that holds `sub_tsv`.
    fn sec_client_with_sub_tsv(sub_tsv: &str) -> RoutingSecClient {
        RoutingSecClient::new()
            .route(
                &format!("{NOTES_URL}2026_09_notes.zip"),
                200,
                "application/zip",
                &notes_zip(sub_tsv),
            )
            .route(
                SUBMISSIONS_URL_OF_320193,
                200,
                "application/json",
                SAMPLE_SUBMISSIONS_JSON,
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

    /// Runs `inputs`, and returns the unit of each event that `is_kept` keeps, in order. A unit
    /// with many events in a row appears once.
    async fn units_of(
        inputs: BackfillInputs<RoutingSecClient>,
        is_kept: impl Fn(&Unit) -> bool + Send + Sync + 'static,
    ) -> Vec<String> {
        let units = Arc::new(Mutex::new(Vec::<String>::new()));
        let observed_units = Arc::clone(&units);
        let observer = move |unit: &Unit, _: RunEvent<'_>| {
            if !is_kept(unit) {
                return;
            }
            let mut units = observed_units.lock().expect(LOCK_EXPECT);
            let unit = unit.to_string();
            if units.last() != Some(&unit) {
                units.push(unit);
            }
        };
        let _ = run(inputs, observer, |_| {}).await;
        units.lock().expect(LOCK_EXPECT).clone()
    }

    /// Runs `inputs`, and returns each unit of the catch-up, in order.
    async fn catch_up_units(inputs: BackfillInputs<RoutingSecClient>) -> Vec<String> {
        units_of(inputs, |unit| {
            matches!(
                unit,
                Unit::Date(_) | Unit::Filing { .. } | Unit::Company { .. }
            )
        })
        .await
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
    async fn should_store_the_submissions_of_the_notes_ciks_before_the_catch_up_when_both_calls_run()
     {
        let root = temporary_root();
        let inputs = inputs(&root, &["2026_09"]);

        let expected_result = vec![
            "320193".to_string(),
            "2026-09-30".to_string(),
            "2026-10-01".to_string(),
        ];

        let result = units_of(inputs, |unit| !matches!(unit, Unit::Period(_))).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_run_every_catch_up_date_when_a_submissions_cik_fails() {
        let root = temporary_root();
        let mut inputs = inputs(&root, &["2026_09"]);
        inputs.sec_client = sec_client_with_sub_tsv(SUB_TSV_WITH_A_CIK_WITHOUT_SUBMISSIONS);

        let expected_result = vec!["2026-09-30".to_string(), "2026-10-01".to_string()];

        let result = catch_up_units(inputs).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_count_one_failed_submissions_cik_when_the_sec_has_no_submissions_file_of_a_cik()
    {
        let root = temporary_root();
        let mut inputs = inputs(&root, &["2026_09"]);
        inputs.sec_client = sec_client_with_sub_tsv(SUB_TSV_WITH_A_CIK_WITHOUT_SUBMISSIONS);
        let mut failed_ciks = None;

        let expected_result = Some(1);

        run(inputs, |_, _| {}, |past| {
            failed_ciks = Some(past.submissions().failure_count());
        })
        .await
        .expect("Given a stored Notes period and no failed period, the cut-over date should always exist");
        let result = failed_ciks;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_the_backfill_mode_for_the_submissions_file_when_both_calls_run() {
        let root = temporary_root();
        let inputs = inputs(&root, &["2026_09"]);

        let expected_result = json!("backfill");

        run_quietly(inputs)
            .await
            .expect("Given a stored Notes period and no failed period, the cut-over date should always exist");
        let result =
            stored_metadata(&root, "sec/submissions/CIK0000320193.json.meta.json")["ingestion_mode"]
                .clone();

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
