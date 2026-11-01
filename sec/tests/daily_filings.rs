//! # `daily_filings` Tests
//!
//! Runs the tests of the modules of the `daily_filings` binary. The binary is its own crate root,
//! so this test compiles its modules as private modules, as `notes_backfill.rs` does.

#[path = "../src/bin/daily_filings/constants.rs"]
mod constants;
#[path = "../src/bin/daily_filings/dates.rs"]
mod dates;
#[path = "../src/bin/daily_filings/run.rs"]
mod run;

/// Runs the loop of `run.rs` offline, with a fake SEC client and stores in a temp dir.
mod run_loop {
    use std::time::Duration;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use sec::shared::raw_document::RawDocument;
    use sec::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use serde_json::{Value, json};
    use storage::implementations::repository::filesystem::FilesystemRepository;
    use tempfile::TempDir;
    use uuid::Uuid;

    use crate::run::{FailedDailyRun, Repositories, RunSummary, Settings, run};

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

    /// The index of 2026-10-26 with a second 10-Q of the same company.
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

    fn repositories(root: &TempDir) -> Repositories {
        Repositories {
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

    /// Runs `dates` with yesterday on 2026-10-28 and no wait between attempts.
    async fn run_offline(dates: &[NaiveDate], sec_client: &RoutingSecClient) -> RunSummary {
        let root = temporary_root();
        let settings = Settings {
            execution_id: Uuid::nil(),
            yesterday: date("2026-10-28"),
            retry_delays: vec![Duration::ZERO, Duration::ZERO],
        };
        run(
            dates,
            sec_client,
            &repositories(&root),
            &settings,
            |_, _| {},
            |_, _| {},
        )
        .await
    }

    /// Returns the detection outcome of each date, and for each date the counts of stored
    /// filings, unextractable filings, failed filings, and stored `submissions` files.
    fn outcomes(summary: &RunSummary) -> Vec<Value> {
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
}
