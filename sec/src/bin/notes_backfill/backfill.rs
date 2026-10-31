//! # Backfill
//!
//! Provides [`run`], which extracts one Notes file per period, and the [`RunSummary`] it returns.
//!
//! The SEC posts the file of a month some days into the next month. So the last period of a run
//! can have no file yet. [`RunSummary`] counts it as unpublished, not as failed, if its file is
//! absent. The next run asks for it again.
//!
//! Before it downloads a period, the run reads the stored file of the period. If the file passes
//! the size and SHA-256 check, the run skips the download.

use std::panic;
use std::sync::Arc;

use futures_util::StreamExt;
use sec::error::{State as StateError, StateMachine};
use sec::implementations::extraction::Backfill;
use sec::implementations::states::extract::store_notes_archive::notes_key;
use sec::prelude::{StreamError, StreamItem};
use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::notes_archive_ref::NotesArchiveRef;
use sec::shared::notes_period::NotesPeriod;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::RawNotesArchive;
use serde_json::{Value, json};
use storage::ReadError;
use storage::implementations::repository::filesystem::{
    Document, FilesystemRepository, Read, Write,
};
use storage::traits::repository::ReadRepository;
use thiserror::Error;

/// The most Notes files that the run downloads at once.
///
/// The rate limiter limits requests per second, not files in memory, so this cap bounds memory.
pub const FILES_IN_FLIGHT: usize = 2;

/// The most periods at the end of a run that [`RunSummary`] counts as unpublished.
///
/// The run asks for periods up to last month, so only the last period can lack a file. The cap
/// keeps a run in which the SEC refuses every request with 403 from counting as a success.
pub const MAX_UNPUBLISHED: usize = 1;

/// How the run got the stored file of a period.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fetch {
    /// The run downloaded and stored the file.
    Downloaded,
    /// The store already held an intact file, so the run skipped the download.
    AlreadyStored,
    /// The store held a file that it could not read or that failed the size or SHA-256 check, so
    /// the run downloaded the file again.
    Replaced,
}

/// The outcome of one period: the stored archive and how the run got it, or the error of the state
/// that failed.
pub type PeriodOutcome = Result<(Fetch, NotesArchiveRef), StreamError>;

/// The outcome of every period of one run, in the order of the given periods.
pub struct RunSummary(Vec<(String, PeriodOutcome)>);

/// Returns `true` if `error` comes from an SEC answer of 403 or 404 for the Notes file.
pub const fn is_absent_file(error: &StreamError) -> bool {
    matches!(
        &error.source,
        StateMachine::State(StateError::FailedRequestExecution(failure))
            if failure.domain_error().is_absent_file()
    )
}

/// The error of a run in which at least one period failed.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("[FailedBackfill] {failures} of {periods} Notes periods failed")]
pub struct FailedBackfill {
    /// The number of periods that failed.
    pub failures: usize,
    /// The number of periods in the run.
    pub periods: usize,
}

impl RunSummary {
    /// Returns `Ok` if every period stored its file or is unpublished.
    ///
    /// # Errors
    ///
    /// Returns [`FailedBackfill`] if at least one period failed.
    pub fn result(&self) -> Result<(), FailedBackfill> {
        match self.failure_count() {
            0 => Ok(()),
            failures => Err(FailedBackfill {
                failures,
                periods: self.0.len(),
            }),
        }
    }

    /// Returns the number of periods that have a stored file, downloaded or already stored.
    pub fn success_count(&self) -> usize {
        self.0.iter().filter(|(_, outcome)| outcome.is_ok()).count()
    }

    /// Returns the number of periods whose intact file the store already held.
    pub fn already_stored_count(&self) -> usize {
        self.0
            .iter()
            .filter(|(_, outcome)| matches!(outcome, Ok((Fetch::AlreadyStored, _))))
            .count()
    }

    /// Returns the number of periods that failed and are not unpublished.
    pub fn failure_count(&self) -> usize {
        self.0.len() - self.success_count() - self.unpublished_count()
    }

    /// Returns the number of periods at the end of the run whose file is absent, at most
    /// [`MAX_UNPUBLISHED`].
    pub fn unpublished_count(&self) -> usize {
        self.0
            .iter()
            .rev()
            .take(MAX_UNPUBLISHED)
            .take_while(|(_, outcome)| outcome.as_ref().is_err_and(is_absent_file))
            .count()
    }

    /// Returns one JSON entry per period.
    ///
    /// A stored period lists its status (`downloaded`, `already_stored`, or `replaced`), key, size,
    /// and SHA-256.
    /// An unpublished period says so. A failed period lists the failed state and the error.
    pub fn entries(&self) -> Vec<Value> {
        let first_unpublished = self.0.len() - self.unpublished_count();
        self.0
            .iter()
            .enumerate()
            .map(|(index, (period, outcome))| match outcome {
                Err(_) if index >= first_unpublished => json!({
                    "period": period,
                    "status": "unpublished",
                }),
                Ok((fetch, archive)) => json!({
                    "period": period,
                    "status": match fetch {
                        Fetch::Downloaded => "downloaded",
                        Fetch::AlreadyStored => "already_stored",
                        Fetch::Replaced => "replaced",
                    },
                    "key": archive.key().as_path().display().to_string(),
                    "size_in_bytes": archive.size_in_bytes(),
                    "sha256": archive.sha256().as_str(),
                }),
                Err(error) => json!({
                    "period": period,
                    "failed_state": error.state_name,
                    "error": error.to_string(),
                }),
            })
            .collect()
    }
}

/// Returns the stored archive of `period`, or `None` if `reader` holds no file of it.
///
/// An invalid period also returns `None`, so the run reports its validation error.
///
/// # Errors
///
/// Returns a [`ReadError`] if `reader` cannot read the stored file, or if the file fails the size
/// or SHA-256 check.
pub async fn stored_archive(
    reader: &FilesystemRepository<Read, RawDocument<RawNotesArchive>>,
    period: &str,
) -> Result<Option<NotesArchiveRef>, ReadError> {
    let Ok(period) = NotesPeriod::new(period) else {
        return Ok(None);
    };
    let key = notes_key(period);
    Ok(reader.get(key.clone()).await?.map(|document| {
        let metadata = document.metadata();
        NotesArchiveRef::new(
            period,
            key,
            metadata.size_in_bytes(),
            metadata.sha256().clone(),
        )
    }))
}

/// What the run does with a period, given the read of its stored file.
#[derive(Debug, PartialEq, Eq)]
pub enum Plan {
    /// The stored file is intact, so the run skips the download.
    Skip(NotesArchiveRef),
    /// The run downloads the file and tags the period with this fetch.
    Download(Fetch),
}

/// Returns the plan for a period from the read of its stored file.
///
/// A read error means the stored file is unreadable or corrupt, so the run replaces it.
pub fn plan(read: Result<Option<NotesArchiveRef>, ReadError>) -> Plan {
    match read {
        Ok(Some(archive)) => Plan::Skip(archive),
        Ok(None) => Plan::Download(Fetch::Downloaded),
        Err(_) => Plan::Download(Fetch::Replaced),
    }
}

/// Extracts the Notes file of each period in `periods` into `repository`.
///
/// Every super-state comes from `backfill`, so all of them carry one `execution_id`. At most
/// [`FILES_IN_FLIGHT`] periods run at once, each in its own task. A failed period does not stop
/// the others. A period whose intact file `repository` already holds skips the download.
///
/// Calls `on_item` with the period and each successful stream item, and `on_error` with the period
/// and the error of a failed period.
///
/// # Panics
///
/// Panics if the task of a period panics, with the payload of that panic.
pub async fn run(
    backfill: Backfill,
    periods: &[&str],
    sec_client: &SecClient,
    repository: &FilesystemRepository<Write, RawDocument<RawNotesArchive>>,
    on_item: impl Fn(&str, &StreamItem) + Send + Sync + 'static,
    on_error: impl Fn(&str, &StreamError) + Send + Sync + 'static,
) -> RunSummary {
    let on_item = Arc::new(on_item);
    let on_error = Arc::new(on_error);
    let reader = FilesystemRepository::builder()
        .reader()
        .document_type::<RawDocument<RawNotesArchive>>()
        .root(repository.root())
        .build();
    let extractions = periods.iter().enumerate().map(|(index, &period)| {
        let super_state = backfill
            .notes()
            .period(period)
            .sec_client(sec_client.clone())
            .repository(repository.clone())
            .build();
        let period = period.to_string();
        let on_item = Arc::clone(&on_item);
        let on_error = Arc::clone(&on_error);
        let reader = reader.clone();
        // STA-344 added a task per period after a stream cut-off. Its cause is open, so keeping
        // this split is an open decision (STA-360).
        // The check runs inside the future, so `FILES_IN_FLIGHT` also caps the files it reads.
        async move {
            let fetch = match plan(stored_archive(&reader, &period).await) {
                Plan::Skip(archive) => return (index, period, Ok((Fetch::AlreadyStored, archive))),
                Plan::Download(fetch) => fetch,
            };
            let task = tokio::spawn(async move {
                let outcome = super_state
                    .run_to_archive(|item| on_item(&period, item))
                    .await
                    .map(|archive| (fetch, archive))
                    .inspect_err(|error| on_error(&period, error));
                (index, period, outcome)
            });
            task.await
                .unwrap_or_else(|error| panic::resume_unwind(error.into_panic()))
        }
    });
    let mut outcomes: Vec<_> = futures_util::stream::iter(extractions)
        .buffer_unordered(FILES_IN_FLIGHT)
        .collect()
        .await;
    outcomes.sort_by_key(|(index, _, _)| *index);
    RunSummary(
        outcomes
            .into_iter()
            .map(|(_, period, outcome)| (period, outcome))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::{DateTime, Utc};
    use pretty_assertions::assert_eq;
    use sec::error::state_machine::state::FailedRequestExecution;
    use sec::implementations::extraction::Extraction;
    use sec::prelude::StreamEvent;
    use sec::shared::build_info::producer;
    use sec::shared::content_type::ContentType;
    use sec::shared::headers::Headers;
    use sec::shared::http_client::implementations::reqwest_client::ReqwestClient;
    use sec::shared::http_client::implementations::sec_client::error::FailedSecRequest;
    use sec::shared::notes_period::NotesPeriod;
    use sec::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use sec::shared::response::implementations::sec_response::{SecResponse, ZipBody};
    use sec::shared::run_fields::RunFields;
    use sec::shared::status_code::StatusCode;
    use sec::shared::url::Url;
    use sec::shared::user_agent::UserAgent;
    use storage::implementations::repository::filesystem::{DocumentKey, IngestionMode};
    use storage::shared::Sha256Digest;
    use storage::traits::repository::WriteRepository;
    use tempfile::TempDir;

    use super::*;

    const STORED_PERIOD: &str = "2009q1";
    const STORED_KEY: &str = "sec/notes/2009q1_notes.zip";

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn writer(root: &TempDir) -> FilesystemRepository<Write, RawDocument<RawNotesArchive>> {
        FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<RawNotesArchive>>()
            .root(root.path())
            .build()
    }

    fn reader(root: &TempDir) -> FilesystemRepository<Read, RawDocument<RawNotesArchive>> {
        FilesystemRepository::builder()
            .reader()
            .document_type::<RawDocument<RawNotesArchive>>()
            .root(root.path())
            .build()
    }

    fn user_agent() -> UserAgent {
        UserAgent::new("ArkadTest admin@example.com")
            .expect("A hardcoded SEC-format string should always validate")
    }

    /// Stores a file with the bytes `PK` under the key of [`STORED_PERIOD`].
    async fn store_sample_file(root: &TempDir) {
        let url: Url = "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2009q1_notes.zip"
            .parse()
            .expect("A hardcoded valid URL string should always parse");
        let headers = Headers::new(HashMap::from([(
            "content-type".to_string(),
            "application/zip".to_string(),
        )]));
        let file = SecResponse::<ZipBody>::from_parts(
            url.clone(),
            headers,
            ContentType::Zip,
            StatusCode::Ok,
            b"PK".to_vec(),
        )
        .expect("Given status 200 and a ZIP, building the response should always succeed");
        let key = DocumentKey::new(STORED_KEY)
            .expect("Given a valid relative path, the key should always build");
        let run_fields = RunFields {
            produced_at: DateTime::<Utc>::UNIX_EPOCH,
            execution_id: uuid::Uuid::nil(),
            producer: producer("Store Notes Archive"),
            ingestion_mode: IngestionMode::Backfill,
        };
        let document =
            RawDocument::<RawNotesArchive>::from_file(key, &file, &url, &user_agent(), run_fields);
        writer(root)
            .persist(document)
            .await
            .expect("Given a writable temporary root, the write should always succeed");
    }

    /// Replaces the stored bytes with two other bytes, so the size check passes and the SHA-256
    /// check fails.
    fn corrupt_sample_file(root: &TempDir) {
        std::fs::write(root.path().join(STORED_KEY), b"PX")
            .expect("Given a stored file in a writable temporary root, overwriting it should always succeed");
    }

    /// Returns an SEC client that sends every request to a closed local port, so no request
    /// reaches the SEC.
    fn offline_sec_client() -> SecClient {
        let proxy = reqwest::Proxy::all("http://127.0.0.1:9")
            .expect("A hardcoded proxy URL should always parse");
        let builder = reqwest::Client::builder().proxy(proxy);
        let http_client = ReqwestClient::from_builder(builder, user_agent())
            .expect("Given a validated user agent, building the HTTP client fails only if the TLS backend cannot initialize");
        SecClient::new(http_client)
    }

    async fn run_offline(root: &TempDir) -> RunSummary {
        run(
            Extraction::backfill(),
            &[STORED_PERIOD],
            &offline_sec_client(),
            &writer(root),
            |_, _| {},
            |_, _| {},
        )
        .await
    }

    /// Returns the archive of `period` with the bytes `PK`, under the key that `notes_key` gives.
    fn archive_of(period: &str) -> NotesArchiveRef {
        let period = NotesPeriod::new(period)
            .expect("Given a hardcoded published period, parsing should always succeed");
        NotesArchiveRef::new(period, notes_key(period), 2, Sha256Digest::of(b"PK"))
    }

    fn sample_archive() -> NotesArchiveRef {
        archive_of(STORED_PERIOD)
    }

    #[tokio::test]
    async fn should_return_the_stored_archive_when_the_stored_file_is_intact() {
        let root = temporary_root();
        store_sample_file(&root).await;

        let expected_result = Ok(Some(sample_archive()));

        let result = stored_archive(&reader(&root), STORED_PERIOD).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_the_store_holds_no_file_of_the_period() {
        let root = temporary_root();

        let expected_result = Ok(None);

        let result = stored_archive(&reader(&root), STORED_PERIOD).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_a_read_error_when_the_stored_file_fails_the_sha256_check() {
        let root = temporary_root();
        store_sample_file(&root).await;
        corrupt_sample_file(&root);

        let expected_result = true;

        let result = matches!(
            stored_archive(&reader(&root), STORED_PERIOD).await,
            Err(ReadError::Backend(_))
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_plan_a_replacement_when_the_stored_file_fails_the_sha256_check() {
        let root = temporary_root();
        store_sample_file(&root).await;
        corrupt_sample_file(&root);
        let read = stored_archive(&reader(&root), STORED_PERIOD).await;

        let expected_result = Plan::Download(Fetch::Replaced);

        let result = plan(read);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_plan_a_download_when_the_store_holds_no_file_of_the_period() {
        let root = temporary_root();
        let read = stored_archive(&reader(&root), STORED_PERIOD).await;

        let expected_result = Plan::Download(Fetch::Downloaded);

        let result = plan(read);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_plan_a_skip_when_the_stored_file_is_intact() {
        let root = temporary_root();
        store_sample_file(&root).await;
        let read = stored_archive(&reader(&root), STORED_PERIOD).await;

        let expected_result = Plan::Skip(sample_archive());

        let result = plan(read);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_skip_the_download_when_the_stored_file_is_intact() {
        let root = temporary_root();
        store_sample_file(&root).await;

        let expected_result = vec![json!({
            "period": STORED_PERIOD,
            "status": "already_stored",
            "key": STORED_KEY,
            "size_in_bytes": 2,
            "sha256": "fcab7fcc2b4cffd9bb45003bfc2e468a04ef6f77ca8200a7341f027631584d25",
        })];

        let result = run_offline(&root).await.entries();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_request_the_file_again_when_the_stored_file_fails_the_sha256_check() {
        let root = temporary_root();
        store_sample_file(&root).await;
        corrupt_sample_file(&root);

        let expected_result = json!("Execute SEC Request");

        let result = run_offline(&root).await.entries()[0]["failed_state"].clone();

        assert_eq!(result, expected_result);
    }

    fn failed(period: &str, source: StateMachine) -> (String, PeriodOutcome) {
        let error = StreamError {
            event: StreamEvent::StateFailed,
            execution_id: uuid::Uuid::nil(),
            state_name: "Execute SEC Request".to_string(),
            data: Value::Null,
            source,
        };
        (period.to_string(), Err(error))
    }

    fn stored(period: &str) -> (String, PeriodOutcome) {
        stored_by(period, Fetch::Downloaded)
    }

    fn stored_by(period: &str, fetch: Fetch) -> (String, PeriodOutcome) {
        (period.to_string(), Ok((fetch, archive_of(period))))
    }

    fn answered(period: &str, status_code: StatusCode) -> (String, PeriodOutcome) {
        let response_error =
            InvalidSecResponse::new(ErrorReason::InvalidStatusCode { status_code });
        let failure = FailedRequestExecution::new(
            "Execute SEC Request",
            FailedSecRequest::from(response_error),
        );
        failed(
            period,
            StateMachine::State(StateError::FailedRequestExecution(failure)),
        )
    }

    #[test]
    fn should_return_ok_when_only_the_last_period_is_absent() {
        let summary = RunSummary(vec![
            stored("2026_08"),
            answered("2026_09", StatusCode::NotFound),
        ]);

        let expected_result = Ok(());

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_only_already_stored_periods_when_the_run_mixes_fetches() {
        let summary = RunSummary(vec![
            stored_by("2009q1", Fetch::AlreadyStored),
            stored_by("2009q2", Fetch::Downloaded),
            stored_by("2009q3", Fetch::Replaced),
        ]);

        let expected_result = 1;

        let result = summary.already_stored_count();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_the_status_of_each_stored_period_when_listing_entries() {
        let summary = RunSummary(vec![
            stored_by("2009q1", Fetch::AlreadyStored),
            stored_by("2009q2", Fetch::Downloaded),
            stored_by("2009q3", Fetch::Replaced),
        ]);

        let expected_result = vec![
            json!("already_stored"),
            json!("downloaded"),
            json!("replaced"),
        ];

        let result: Vec<Value> = summary
            .entries()
            .into_iter()
            .map(|entry| entry["status"].clone())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_absent_periods_as_failed_when_more_than_the_last_period_is_absent() {
        let summary = RunSummary(vec![
            answered("2026_08", StatusCode::Forbidden),
            answered("2026_09", StatusCode::Forbidden),
        ]);

        let expected_result = Err(FailedBackfill {
            failures: 1,
            periods: 2,
        });

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_an_absent_period_as_failed_when_a_later_period_is_stored() {
        let summary = RunSummary(vec![
            answered("2026_08", StatusCode::NotFound),
            stored("2026_09"),
        ]);

        let expected_result = Err(FailedBackfill {
            failures: 1,
            periods: 2,
        });

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_an_absent_period_as_failed_when_a_later_period_failed_otherwise() {
        let summary = RunSummary(vec![
            answered("2026_09", StatusCode::NotFound),
            failed("2026_10", StateMachine::InvalidConfiguration),
        ]);

        let expected_result = Err(FailedBackfill {
            failures: 2,
            periods: 2,
        });

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_a_last_period_as_failed_when_the_sec_answers_500() {
        let summary = RunSummary(vec![answered("2026_09", StatusCode::InternalServerError)]);

        let expected_result = Err(FailedBackfill {
            failures: 1,
            periods: 1,
        });

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_mark_the_last_absent_period_as_unpublished_when_listing_entries() {
        let summary = RunSummary(vec![
            failed("2026_08", StateMachine::InvalidConfiguration),
            answered("2026_09", StatusCode::NotFound),
        ]);

        let expected_result = json!({ "period": "2026_09", "status": "unpublished" });

        let result = summary.entries()[1].clone();

        assert_eq!(result, expected_result);
    }
}
