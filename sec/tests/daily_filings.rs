//! # `daily_filings` Tests
//!
//! Runs the tests of the modules of the `daily_filings` binary, and the run of the `Extraction`
//! facade that the binary calls: offline against a fake SEC, and for one date live. The binary is
//! its own crate root, so this test compiles its modules as private modules.
//!
//! ## External Dependencies
//!
//! - `www.sec.gov` and `data.sec.gov`: the daily index of [`live::DATE`], and the folders and the
//!   `submissions` files of its filings

#[path = "../src/bin/daily_filings/constants.rs"]
mod constants;
#[path = "../src/bin/daily_filings/dates.rs"]
mod dates;

/// Runs the facade offline, with a fake SEC client and stores in a temp dir.
mod run_loop {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use sec::implementations::extraction::{
        DailyFilingsRepositories, DailyRunSummary, Extraction, FailedDailyRun, RunEvent, Unit,
    };
    use sec::shared::raw_document::RawDocument;
    use sec::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use serde_json::{Value, json};
    use storage::implementations::repository::filesystem::FilesystemRepository;
    use tempfile::TempDir;
    use uuid::Uuid;

    const INDEX_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261026.idx";
    const QTR4_LISTING_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/index.json";
    const QTR4_LISTING: &[u8] = br#"{"directory":{"item":[{"name":"form.20261026.idx"}]}}"#;
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
    fn sec_client_with_folder(folder_index: &[(u16, &str, &[u8])]) -> RoutingSecClient {
        RoutingSecClient::new()
            .route(INDEX_URL, 200, "application/octet-stream", DAILY_INDEX)
            .route(QTR4_LISTING_URL, 200, "application/json", QTR4_LISTING)
            .route_sequence(FOLDER_INDEX_URL, folder_index)
            .route(INSTANCE_URL, 200, "text/xml", b"<xbrl/>")
            .route(LABEL_FILE_URL, 200, "text/xml", b"<labels/>")
            .route(SUBMISSIONS_URL, 200, "application/json", SUBMISSIONS)
    }

    /// The index of 2026-10-26 with a 10-Q and a 10-Q/A of the same company.
    const DAILY_INDEX_WITH_TWO_FILINGS: &[u8] = b"Form Type   Company Name   CIK   Date Filed  File Name
---------------------------------------------------------------------
10-Q             ALCO STORES INC                                               3545        20261026    edgar/data/3545/0000003545-26-000047.txt
10-Q/A           ALCO STORES INC                                               3545        20261026    edgar/data/3545/0000003545-26-000048.txt
";

    fn sec_client() -> RoutingSecClient {
        sec_client_with_folder(&[(200, "application/json", FOLDER_INDEX)])
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn repositories(root: &TempDir) -> DailyFilingsRepositories {
        DailyFilingsRepositories {
            daily_index: FilesystemRepository::builder()
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

    /// Runs `dates` with the stores under `root`, yesterday on 2026-10-28, and no wait between
    /// attempts, and passes each event to `observer`.
    async fn run_observed(
        root: &TempDir,
        dates: &[NaiveDate],
        sec_client: &RoutingSecClient,
        observer: impl Fn(&Unit, RunEvent<'_>) + Send + Sync + 'static,
    ) -> DailyRunSummary {
        Extraction::incremental()
            .execution_id(Uuid::from_u128(7))
            .daily_filings()
            .dates(dates.iter().copied())
            .yesterday(date("2026-10-28"))
            .sec_client(sec_client.clone())
            .repositories(repositories(root))
            .retry_delays([Duration::ZERO, Duration::ZERO])
            .run(observer)
            .await
    }

    /// Runs `dates` as `run_observed` does, without an observer, in a new temp dir.
    async fn run_offline(dates: &[NaiveDate], sec_client: &RoutingSecClient) -> DailyRunSummary {
        run_offline_in(&temporary_root(), dates, sec_client).await
    }

    /// Runs `dates` as `run_observed` does, without an observer.
    async fn run_offline_in(
        root: &TempDir,
        dates: &[NaiveDate],
        sec_client: &RoutingSecClient,
    ) -> DailyRunSummary {
        run_observed(root, dates, sec_client, |_, _| {}).await
    }

    const LOCK_EXPECT: &str = "Given an observer that never panics while it holds the lock, the lock should never be poisoned";

    /// Reads the metadata file at `file` under `root` as JSON.
    fn stored_metadata(root: &TempDir, file: &str) -> Value {
        serde_json::from_slice(
            &std::fs::read(root.path().join(file)).expect(
                "Given a run that stored the document, its metadata file should always exist",
            ),
        )
        .expect("Given a metadata file that the store wrote, it should always parse as JSON")
    }

    /// Runs `dates` as `run_offline` does, and returns the unit and the kind of each error.
    async fn reported_errors(dates: &[NaiveDate], sec_client: &RoutingSecClient) -> Vec<Value> {
        run_reporting_errors(dates, sec_client).await.0
    }

    /// Runs `dates` as `run_offline` does, and returns the unit and the kind of each error, and
    /// the summary.
    async fn run_reporting_errors(
        dates: &[NaiveDate],
        sec_client: &RoutingSecClient,
    ) -> (Vec<Value>, DailyRunSummary) {
        let errors = Arc::new(Mutex::new(Vec::new()));
        let observed_errors = Arc::clone(&errors);
        let summary = run_observed(&temporary_root(), dates, sec_client, move |unit, event| {
            if let RunEvent::Error(_, kind) = event {
                observed_errors
                    .lock()
                    .expect(LOCK_EXPECT)
                    .push(json!({ "context": unit, "kind": kind.to_string() }));
            }
        })
        .await;
        let errors = errors.lock().expect(LOCK_EXPECT).clone();
        (errors, summary)
    }

    /// Returns the detection outcome of each date, and for each date the counts of stored
    /// filings, unextractable filings, failed filings, and stored `submissions` files.
    fn outcomes(summary: &DailyRunSummary) -> Vec<Value> {
        summary
            .entries()
            .iter()
            .map(|entry| {
                json!({
                    "detection": entry["detection"],
                    "stored": entry["filings"]["stored"],
                    "unextractable": entry["filings"]["unextractable"].as_array().map(Vec::len),
                    "failed": entry["filings"]["failed"].as_array().map(Vec::len),
                    "submissions_stored": entry["submissions"]["stored"],
                })
            })
            .collect()
    }

    #[tokio::test]
    async fn should_store_the_filing_and_its_company_when_the_date_has_an_index() {
        let dates = [date("2026-10-26")];
        let sec_client = sec_client();

        let expected_result = vec![json!({
            "detection": "processed",
            "stored": 1,
            "unextractable": 0,
            "failed": 0,
            "submissions_stored": 1,
        })];

        let result = outcomes(&run_offline(&dates, &sec_client).await);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_count_a_closed_day_as_empty_when_a_later_index_exists() {
        let dates = [date("2026-10-25"), date("2026-10-26")];
        let sec_client = sec_client();

        let expected_result = vec!["empty", "processed"];

        let result: Vec<String> = run_offline(&dates, &sec_client)
            .await
            .entries()
            .iter()
            .map(|entry| entry["detection"].as_str().unwrap_or_default().to_string())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_filing_when_the_sec_answers_503_once() {
        let dates = [date("2026-10-26")];
        let sec_client = sec_client_with_folder(&[
            (503, "text/html", b""),
            (200, "application/json", FOLDER_INDEX),
        ]);

        let expected_result = Ok(());

        let result = run_offline(&dates, &sec_client).await.result();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_the_filing_and_still_store_the_company_when_every_attempt_gets_503() {
        let dates = [date("2026-10-26")];
        let sec_client = sec_client_with_folder(&[(503, "text/html", b"")]);

        let expected_result = vec![json!({
            "detection": "processed",
            "stored": 0,
            "unextractable": 0,
            "failed": 1,
            "submissions_stored": 1,
        })];

        let result = outcomes(&run_offline(&dates, &sec_client).await);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_ok_when_the_only_filing_has_no_instance() {
        let dates = [date("2026-10-26")];
        let sec_client = sec_client_with_folder(&[(
            200,
            "application/json",
            br#"{"directory":{"item":[{"name":"alco-20260908.htm"}]}}"#,
        )]);

        let expected_result = Ok(());

        let result = run_offline(&dates, &sec_client).await.result();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_the_run_when_the_folder_holds_two_instances() {
        let dates = [date("2026-10-26")];
        let sec_client = sec_client_with_folder(&[(
            200,
            "application/json",
            br#"{"directory":{"item":[{"name":"a_htm.xml"},{"name":"b_htm.xml"}]}}"#,
        )]);

        let expected_result = Err(FailedDailyRun {
            failures: 1,
            dates: 1,
        });

        let result = run_offline(&dates, &sec_client).await.result();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_count_recent_missing_indexes_as_pending_when_the_first_date_has_an_index() {
        let dates = [date("2026-10-26"), date("2026-10-27"), date("2026-10-28")];
        let sec_client = sec_client();

        let expected_result = vec!["processed", "pending", "pending"];

        let result: Vec<String> = run_offline(&dates, &sec_client)
            .await
            .entries()
            .iter()
            .map(|entry| entry["detection"].as_str().unwrap_or_default().to_string())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_the_run_when_the_first_date_has_no_index() {
        let dates = [date("2026-10-27"), date("2026-10-28")];
        let sec_client = sec_client();

        let expected_result = Err(FailedDailyRun {
            failures: 1,
            dates: 2,
        });

        let result = run_offline(&dates, &sec_client).await.result();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_run_one_submissions_machine_when_two_filings_share_a_company() {
        let dates = [date("2026-10-26")];
        let sec_client = RoutingSecClient::new()
            .route(
                INDEX_URL,
                200,
                "application/octet-stream",
                DAILY_INDEX_WITH_TWO_FILINGS,
            )
            .route(QTR4_LISTING_URL, 200, "application/json", QTR4_LISTING)
            .route(SUBMISSIONS_URL, 200, "application/json", SUBMISSIONS);

        let expected_result = json!({ "companies": 1, "stored": 1, "failed": [] });

        let result = run_offline(&dates, &sec_client).await.entries()[0]["submissions"].clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_report_a_retried_and_a_failed_error_with_the_filing_when_a_503_is_followed_by_a_404()
     {
        let dates = [date("2026-10-26")];
        let sec_client =
            sec_client_with_folder(&[(503, "text/html", b""), (404, "text/html", b"")]);

        let expected_result = vec![
            json!({
                "context": { "date": "2026-10-26", "accession_number": "0000003545-26-000047" },
                "kind": "retried",
            }),
            json!({
                "context": { "date": "2026-10-26", "accession_number": "0000003545-26-000047" },
                "kind": "failed",
            }),
        ];

        let result = reported_errors(&dates, &sec_client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_report_pending_with_the_date_when_a_recent_index_is_missing() {
        let dates = [date("2026-10-26"), date("2026-10-27")];
        let sec_client = sec_client();

        let expected_result = vec![json!({
            "context": { "date": "2026-10-27" },
            "kind": "pending",
        })];

        let result = reported_errors(&dates, &sec_client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_report_unextractable_with_the_filing_when_the_folder_has_no_instance() {
        let dates = [date("2026-10-26")];
        let sec_client = sec_client_with_folder(&[(
            200,
            "application/json",
            br#"{"directory":{"item":[{"name":"alco-20260908.htm"}]}}"#,
        )]);

        let expected_result = vec![json!({
            "context": { "date": "2026-10-26", "accession_number": "0000003545-26-000047" },
            "kind": "unextractable",
        })];

        let result = reported_errors(&dates, &sec_client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_name_the_daily_index_as_the_input_of_a_stored_filing() {
        let root = temporary_root();
        let dates = [date("2026-10-26")];
        let sec_client = sec_client();
        let metadata_file = "sec/filings/CIK0000003545/0000003545-26-000047/instance.xml.meta.json";

        let expected_result = json!([{
            "key": "sec/daily-index/form.20261026.idx",
            "sha256": "83ca0d88935c90c2780df7f3e0ff306903d79779102d891ccb17146987888964",
        }]);

        run_offline_in(&root, &dates, &sec_client).await;
        let result = stored_metadata(&root, metadata_file)["inputs"].clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_the_incremental_ingestion_mode_for_every_document_type_of_a_run() {
        let root = temporary_root();
        let dates = [date("2026-10-26")];
        let sec_client = sec_client();
        let metadata_files = [
            "sec/daily-index/form.20261026.idx.meta.json",
            "sec/filings/CIK0000003545/0000003545-26-000047/instance.xml.meta.json",
            "sec/filings/CIK0000003545/0000003545-26-000047/lab.xml.meta.json",
            "sec/submissions/CIK0000003545.json.meta.json",
        ];

        let expected_result = json!(["incremental", "incremental", "incremental", "incremental"]);

        run_offline_in(&root, &dates, &sec_client).await;
        let result: Value = metadata_files
            .iter()
            .map(|file| stored_metadata(&root, file)["ingestion_mode"].clone())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_run_each_date_once_oldest_first_when_the_dates_are_unsorted_and_repeated() {
        let dates = [date("2026-10-27"), date("2026-10-26"), date("2026-10-27")];
        let sec_client = sec_client();

        let expected_result = json!(["2026-10-26", "2026-10-27"]);

        let result: Value = run_offline(&dates, &sec_client)
            .await
            .entries()
            .iter()
            .map(|entry| entry["date"].clone())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_run_one_filing_at_a_time_when_in_flight_is_one() {
        let root = temporary_root();
        let dates = [date("2026-10-26")];
        let sec_client = RoutingSecClient::new()
            .route(
                INDEX_URL,
                200,
                "application/octet-stream",
                DAILY_INDEX_WITH_TWO_FILINGS,
            )
            .route(QTR4_LISTING_URL, 200, "application/json", QTR4_LISTING)
            .route(FOLDER_INDEX_URL, 200, "application/json", FOLDER_INDEX)
            .route(INSTANCE_URL, 200, "text/xml", b"<xbrl/>")
            .route(LABEL_FILE_URL, 200, "text/xml", b"<labels/>")
            .route(SUBMISSIONS_URL, 200, "application/json", SUBMISSIONS);
        let filing_events = Arc::new(Mutex::new(Vec::new()));
        let observed_filings = Arc::clone(&filing_events);
        let observer = move |unit: &Unit, _: RunEvent<'_>| {
            if let Unit::Filing {
                accession_number, ..
            } = unit
            {
                observed_filings
                    .lock()
                    .expect(LOCK_EXPECT)
                    .push(accession_number.to_string());
            }
        };

        let expected_result = vec!["0000003545-26-000047", "0000003545-26-000048"];

        Extraction::incremental()
            .daily_filings()
            .dates(dates)
            .yesterday(date("2026-10-28"))
            .sec_client(sec_client)
            .repositories(repositories(&root))
            .retry_delays([Duration::ZERO, Duration::ZERO])
            .in_flight(1)
            .run(observer)
            .await;
        let mut result = filing_events.lock().expect(LOCK_EXPECT).clone();
        result.dedup();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_report_every_event_of_a_date_before_the_next_date_starts() {
        let dates = [date("2026-10-26"), date("2026-10-27")];
        let sec_client = sec_client();
        let event_dates = Arc::new(Mutex::new(Vec::new()));
        let observed_dates = Arc::clone(&event_dates);
        let observer = move |unit: &Unit, _: RunEvent<'_>| {
            let unit_date = match unit {
                Unit::Date(date) | Unit::Filing { date, .. } | Unit::Company { date, .. } => *date,
                _ => NaiveDate::MIN,
            };
            observed_dates.lock().expect(LOCK_EXPECT).push(unit_date);
        };

        let expected_result = vec![date("2026-10-26"), date("2026-10-27")];

        run_observed(&temporary_root(), &dates, &sec_client, observer).await;
        let mut result = event_dates.lock().expect(LOCK_EXPECT).clone();
        result.dedup();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_one_execution_id_for_every_document_type_of_a_run() {
        let root = temporary_root();
        let dates = [date("2026-10-26")];
        let sec_client = sec_client();
        let metadata_files = [
            "sec/daily-index/form.20261026.idx.meta.json",
            "sec/filings/CIK0000003545/0000003545-26-000047/instance.xml.meta.json",
            "sec/filings/CIK0000003545/0000003545-26-000047/lab.xml.meta.json",
            "sec/submissions/CIK0000003545.json.meta.json",
        ];

        let expected_result = json!([
            "00000000-0000-0000-0000-000000000007",
            "00000000-0000-0000-0000-000000000007",
            "00000000-0000-0000-0000-000000000007",
            "00000000-0000-0000-0000-000000000007",
        ]);

        run_offline_in(&root, &dates, &sec_client).await;
        let result: Value = metadata_files
            .iter()
            .map(|file| stored_metadata(&root, file)["execution_id"].clone())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_report_a_retried_error_and_store_the_filing_when_a_503_is_followed_by_a_200() {
        let dates = [date("2026-10-26")];
        let sec_client = sec_client_with_folder(&[
            (503, "text/html", b""),
            (200, "application/json", FOLDER_INDEX),
        ]);

        let expected_result = json!({
            "errors": [{
                "context": { "date": "2026-10-26", "accession_number": "0000003545-26-000047" },
                "kind": "retried",
            }],
            "stored": 1,
        });

        let (errors, summary) = run_reporting_errors(&dates, &sec_client).await;
        let result =
            json!({ "errors": errors, "stored": summary.entries()[0]["filings"]["stored"] });

        assert_eq!(result, expected_result);
    }
}

/// Runs the facade for one past date against the live SEC, as the binary does.
mod live {
    use chrono::{NaiveDate, Utc};
    use pretty_assertions::assert_eq;
    use sec::implementations::extraction::{DailyFilingsRepositories, Extraction};
    use sec::shared::http_client::implementations::reqwest_client::ReqwestClient;
    use sec::shared::http_client::implementations::sec_client::SecClient;
    use sec::shared::raw_document::RawDocument;
    use sec::shared::raw_source::RawSource;
    use sec::shared::user_agent::UserAgent;
    use serde_json::json;
    use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
    use tempfile::TempDir;

    use crate::dates::new_york_yesterday;

    /// A past weekday with few filings, so the test stays short: the Friday after Thanksgiving
    /// 2025. A run of it took 10 s on 2026-11-01.
    pub const DATE: &str = "2025-11-28";

    fn sec_client() -> SecClient {
        let user_agent = UserAgent::new("ArkadTest admin@example.com")
            .expect("A hardcoded SEC-format string should always validate");
        let http_client = ReqwestClient::new(user_agent)
            .expect("Given a validated user agent, building the HTTP client fails only if the TLS backend cannot initialize");
        SecClient::new(http_client)
    }

    fn writer<S: RawSource>(root: &TempDir) -> FilesystemRepository<Write, RawDocument<S>> {
        FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<S>>()
            .root(root.path())
            .build()
    }

    fn repositories(root: &TempDir) -> DailyFilingsRepositories {
        DailyFilingsRepositories {
            daily_index: writer(root),
            instance: writer(root),
            labels: writer(root),
            submissions: writer(root),
        }
    }

    #[tokio::test]
    async fn should_store_filings_and_submissions_files_when_a_past_weekday_runs_live() {
        let root = TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        );
        let date = NaiveDate::parse_from_str(DATE, "%Y-%m-%d")
            .expect("Given a hardcoded valid date, parsing should always succeed");

        let expected_result = json!({
            "detection": "processed",
            "stored_a_filing": true,
            "stored_a_submissions_file": true,
        });

        let summary = Extraction::incremental()
            .daily_filings()
            .dates([date])
            .yesterday(new_york_yesterday(Utc::now()))
            .sec_client(sec_client())
            .repositories(repositories(&root))
            .run(|_, _| {})
            .await;
        let entry = &summary.entries()[0];
        let result = json!({
            "detection": entry["detection"],
            "stored_a_filing": entry["filings"]["stored"].as_u64() > Some(0),
            "stored_a_submissions_file": entry["submissions"]["stored"].as_u64() > Some(0),
        });

        assert_eq!(result, expected_result);
    }
}
