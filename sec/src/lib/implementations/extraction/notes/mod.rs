//! # Notes
//!
//! Provides [`BackfillNotes`], the Notes source of a backfill run, and the batch run over many
//! Notes periods with its [`NotesRunSummary`].
//!
//! [`BackfillNotes::period`] returns the builder of one [`ExtractNotesSuperState`].
//! [`BackfillNotes::periods`] returns a [`NotesBatchBuilder`], whose [`run`] extracts one Notes
//! file per period. Before it downloads a period, the run reads the stored file of the period. If
//! the file passes the size and SHA-256 check, the run skips the download.
//!
//! [`run`]: NotesBatchBuilder::run

use std::panic;
use std::sync::Arc;

use futures_util::StreamExt;
use storage::ReadError;
use storage::implementations::repository::filesystem::{
    Document, FilesystemRepository, IngestionMode, Read, Write,
};
use storage::traits::repository::ReadRepository;
use uuid::Uuid;

use super::run_event::{ErrorKind, RunEvent, Unit};
use crate::implementations::states::extract::notes::ExtractNotesSuperState;
use crate::implementations::states::extract::notes::builder::{
    ExtractNotesSuperStateBuilder, NoRepository, NoSecClient,
};
use crate::implementations::states::extract::store_notes_archive::notes_key;
use crate::implementations::states::extract::validate_notes_period::ValidateNotesPeriod;
use crate::prelude::StreamError;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::notes_archive_ref::NotesArchiveRef;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawNotesArchive;
use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
use crate::traits::state_machine::stream::IntoStateMachineStream;
use summary::{Fetch, is_absent_file};

mod summary;

pub use summary::{FailedBackfill, MissingCutOverDate, NotesRunSummary};

/// The number of Notes files that a batch run downloads at once without a call to
/// [`NotesBatchBuilder::in_flight`].
///
/// The rate limiter limits requests per second, not files in memory, so this cap bounds memory.
/// See "Memory and the Cap on Files in Flight" in `sec/design/pipeline/ingestion_paths.md`.
pub const DEFAULT_FILES_IN_FLIGHT: usize = 2;

/// The most periods at the end of a run that the run summary counts as unpublished.
///
/// If the period list ends at last month, only the last period can lack a file. The cap keeps a
/// run in which the SEC refuses every request with 403 from counting as a success.
pub const MAX_UNPUBLISHED: usize = 1;

/// The store that a batch run writes the Notes files to.
type NotesWriter = FilesystemRepository<Write, RawDocument<RawNotesArchive>>;

/// The Notes source of a backfill run, before the choice of one period or many.
///
/// Every super-state built from it carries the `execution_id` of its
/// [`Backfill`](super::Backfill) handle and [`IngestionMode::Backfill`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BackfillNotes {
    execution_id: Uuid,
}

impl BackfillNotes {
    /// Creates a new [`BackfillNotes`] for the run with `execution_id`.
    pub(super) const fn new(execution_id: Uuid) -> Self {
        Self { execution_id }
    }

    /// Returns the builder of an [`ExtractNotesSuperState`] for one raw period, for example
    /// `2024q1`.
    ///
    /// The builder still needs an SEC client and a repository of type `R`.
    #[must_use]
    pub fn period<R: RawRepository<RawNotesArchive>>(
        self,
        period: impl Into<String>,
    ) -> ExtractNotesSuperStateBuilder<String, NoSecClient, Uuid, IngestionMode, NoRepository<R>>
    {
        ExtractNotesSuperState::builder()
            .execution_id(self.execution_id)
            .ingestion_mode(IngestionMode::Backfill)
            .period(period)
    }

    /// Returns the builder of a batch run over `periods`, in the order of the list.
    ///
    /// The builder still needs an SEC client and a repository.
    #[must_use]
    pub fn periods<P: Into<String>>(
        self,
        periods: impl IntoIterator<Item = P>,
    ) -> NotesBatchBuilder<NoSecClient, NoRepository<NotesWriter>> {
        NotesBatchBuilder {
            notes: self,
            periods: periods.into_iter().map(Into::into).collect(),
            sec_client: NoSecClient,
            repository: NoRepository::new(),
            in_flight: DEFAULT_FILES_IN_FLIGHT,
        }
    }
}

/// Builds and runs a batch of Notes periods, one [`ExtractNotesSuperState`] per period.
///
/// Set the client with [`sec_client`](Self::sec_client) and the store with
/// [`repository`](Self::repository), in any order. [`in_flight`](Self::in_flight) is optional.
/// Then call [`run`](Self::run).
///
/// Only a batch has `run`, and only one period has `build`. So `build` does not follow
/// `periods`:
///
/// ```compile_fail,E0599
/// use sec::implementations::extraction::Extraction;
///
/// let _state = Extraction::backfill().notes().periods(["2024q1"]).build();
/// ```
///
/// And `run` does not follow `period`:
///
/// ```compile_fail,E0599
/// use sec::implementations::extraction::Extraction;
/// use sec::shared::raw_document::RawDocument;
/// use sec::shared::raw_source::RawNotesArchive;
/// use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
///
/// let _run = Extraction::backfill()
///     .notes()
///     .period::<FilesystemRepository<Write, RawDocument<RawNotesArchive>>>("2024q1")
///     .run(|_, _| {});
/// ```
#[derive(Debug, Clone)]
pub struct NotesBatchBuilder<C, R> {
    notes: BackfillNotes,
    periods: Vec<String>,
    sec_client: C,
    repository: R,
    in_flight: usize,
}

impl<R> NotesBatchBuilder<NoSecClient, R> {
    /// Sets the SEC client that downloads the Notes files.
    #[must_use]
    pub fn sec_client<C>(self, sec_client: C) -> NotesBatchBuilder<C, R>
    where
        C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
    {
        NotesBatchBuilder {
            notes: self.notes,
            periods: self.periods,
            sec_client,
            repository: self.repository,
            in_flight: self.in_flight,
        }
    }
}

impl<C> NotesBatchBuilder<C, NoRepository<NotesWriter>> {
    /// Sets the store of the Notes files. The run also reads the stored files from its root.
    #[must_use]
    pub fn repository(self, repository: NotesWriter) -> NotesBatchBuilder<C, NotesWriter> {
        NotesBatchBuilder {
            notes: self.notes,
            periods: self.periods,
            sec_client: self.sec_client,
            repository,
            in_flight: self.in_flight,
        }
    }
}

impl<C, R> NotesBatchBuilder<C, R> {
    /// Sets the most periods that run at once. The default is [`DEFAULT_FILES_IN_FLIGHT`], and `0`
    /// counts as `1`.
    #[must_use]
    pub fn in_flight(self, in_flight: usize) -> Self {
        Self {
            in_flight: in_flight.max(1),
            ..self
        }
    }
}

impl<C> NotesBatchBuilder<C, NotesWriter>
where
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest> + 'static,
    ExtractNotesSuperState<ValidateNotesPeriod, NotesWriter, C>: IntoStateMachineStream,
{
    /// Extracts the Notes file of each period into the repository and returns the summary.
    ///
    /// At most the in-flight cap of periods run at once, each in its own task. A failed period
    /// does not stop the others. A period whose intact file the repository already holds skips
    /// the download.
    ///
    /// Calls `observer` with the period and each successful stream item, and with the period and
    /// the error of a failed period. An SEC answer of 403 or 404 gives [`ErrorKind::Absent`], and
    /// every other error gives [`ErrorKind::Failed`]. A skipped period sends no event to the
    /// observer.
    ///
    /// # Panics
    ///
    /// Panics if the task of a period panics, with the payload of that panic.
    pub async fn run(
        self,
        observer: impl Fn(&Unit, RunEvent<'_>) + Send + Sync + 'static,
    ) -> NotesRunSummary {
        let observer = Arc::new(observer);
        let reader = FilesystemRepository::builder()
            .reader()
            .document_type::<RawDocument<RawNotesArchive>>()
            .root(self.repository.root())
            .build();
        let extractions = self.periods.into_iter().enumerate().map(|(index, period)| {
            let super_state = self
                .notes
                .period(period.as_str())
                .sec_client(self.sec_client.clone())
                .repository(self.repository.clone())
                .build();
            let observer = Arc::clone(&observer);
            let reader = reader.clone();
            // STA-344 added a task per period after a stream cut-off. Its cause is open, so keeping
            // this split is an open decision (STA-360).
            // The check runs inside the future, so the cap also limits the files it reads.
            async move {
                let fetch = match plan(stored_archive(&reader, &period).await) {
                    Plan::Skip(archive) => {
                        return (index, period, Ok((Fetch::AlreadyStored, archive)));
                    }
                    Plan::Download(fetch) => fetch,
                };
                let task = tokio::spawn(async move {
                    let unit = Unit::Period(period.clone());
                    let outcome = super_state
                        .run_to_archive(|item| observer(&unit, RunEvent::Item(item)))
                        .await
                        .map(|archive| (fetch, archive))
                        .inspect_err(|error| {
                            observer(&unit, RunEvent::Error(error, error_kind(error)));
                        });
                    (index, period, outcome)
                });
                task.await
                    .unwrap_or_else(|error| panic::resume_unwind(error.into_panic()))
            }
        });
        let mut outcomes: Vec<_> = futures_util::stream::iter(extractions)
            .buffer_unordered(self.in_flight)
            .collect()
            .await;
        outcomes.sort_by_key(|(index, _, _)| *index);
        NotesRunSummary(
            outcomes
                .into_iter()
                .map(|(_, period, outcome)| (period, outcome))
                .collect(),
        )
    }
}

/// Returns [`ErrorKind::Absent`] for an SEC answer of 403 or 404, else [`ErrorKind::Failed`].
fn error_kind(error: &StreamError) -> ErrorKind {
    if is_absent_file(error) {
        ErrorKind::Absent
    } else {
        ErrorKind::Failed
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
async fn stored_archive(
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
enum Plan {
    /// The stored file is intact, so the run skips the download.
    Skip(NotesArchiveRef),
    /// The run downloads the file and tags the period with this fetch.
    Download(Fetch),
}

/// Returns the plan for a period from the read of its stored file.
///
/// A read error means the stored file is unreadable or corrupt, so the run replaces it.
fn plan(read: Result<Option<NotesArchiveRef>, ReadError>) -> Plan {
    match read {
        Ok(Some(archive)) => Plan::Skip(archive),
        Ok(None) => Plan::Download(Fetch::Downloaded),
        Err(_) => Plan::Download(Fetch::Replaced),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use chrono::{DateTime, NaiveDate, Utc};
    use pretty_assertions::assert_eq;
    use serde_json::{Value, json};
    use storage::implementations::repository::filesystem::DocumentKey;
    use storage::shared::Sha256Digest;
    use storage::traits::repository::WriteRepository;
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::extraction::Extraction;
    use crate::shared::build_info::producer;
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::run_fields::RunFields;
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;

    const STORED_PERIOD: &str = "2009q1";
    const STORED_KEY: &str = "sec/notes/2009q1_notes.zip";
    const NOTES_URL: &str =
        "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/";
    const LOCK_EXPECT: &str = "Given an observer that never panics while it holds the lock, the lock should never be poisoned";

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn writer(root: &TempDir) -> NotesWriter {
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
            execution_id: Uuid::nil(),
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

    /// Returns a fake SEC client that answers each of `periods` with the sample Notes file, and
    /// every other period with 404.
    fn sec_client_with_files_of(periods: &[&str]) -> RoutingSecClient {
        periods
            .iter()
            .fold(RoutingSecClient::new(), |client, period| {
                let url = format!("{NOTES_URL}{period}_notes.zip");
                client.route(&url, 200, "application/zip", SAMPLE_NOTES_ZIP)
            })
    }

    /// Runs a batch over `periods` into `root` and passes each event to `observer`.
    async fn run_batch(
        root: &TempDir,
        sec_client: RoutingSecClient,
        periods: &[&str],
        observer: impl Fn(&Unit, RunEvent<'_>) + Send + Sync + 'static,
    ) -> NotesRunSummary {
        Extraction::backfill()
            .notes()
            .periods(periods.iter().copied())
            .sec_client(sec_client)
            .repository(writer(root))
            .run(observer)
            .await
    }

    fn sample_archive() -> NotesArchiveRef {
        let period = NotesPeriod::new(STORED_PERIOD)
            .expect("Given a hardcoded published period, parsing should always succeed");
        NotesArchiveRef::new(period, notes_key(period), 2, Sha256Digest::of(b"PK"))
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

    #[test]
    fn should_plan_a_download_when_the_store_holds_no_file_of_the_period() {
        let read = Ok(None);

        let expected_result = Plan::Download(Fetch::Downloaded);

        let result = plan(read);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_plan_a_skip_when_the_stored_file_is_intact() {
        let read = Ok(Some(sample_archive()));

        let expected_result = Plan::Skip(sample_archive());

        let result = plan(read);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_skip_the_download_when_the_stored_file_is_intact() {
        let root = temporary_root();
        store_sample_file(&root).await;
        let sec_client_without_files = sec_client_with_files_of(&[]);

        let expected_result = vec![json!({
            "period": STORED_PERIOD,
            "status": "already_stored",
            "key": STORED_KEY,
            "size_in_bytes": 2,
            "sha256": "fcab7fcc2b4cffd9bb45003bfc2e468a04ef6f77ca8200a7341f027631584d25",
        })];

        let result = run_batch(&root, sec_client_without_files, &[STORED_PERIOD], |_, _| {})
            .await
            .entries();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_replace_the_stored_file_when_the_stored_file_fails_the_sha256_check() {
        let root = temporary_root();
        store_sample_file(&root).await;
        corrupt_sample_file(&root);
        let sec_client = sec_client_with_files_of(&[STORED_PERIOD]);

        let expected_result = json!("replaced");

        let result = run_batch(&root, sec_client, &[STORED_PERIOD], |_, _| {})
            .await
            .entries()[0]["status"]
            .clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_ok_when_the_file_of_the_newest_period_is_absent() {
        let root = temporary_root();
        let sec_client = sec_client_with_files_of(&["2025q3"]);
        let periods = ["2025q3", "2025_10"];

        let expected_result = Ok(());

        let result = run_batch(&root, sec_client, &periods, |_, _| {})
            .await
            .result();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_backfill_when_the_file_of_an_older_period_is_absent() {
        let root = temporary_root();
        let sec_client = sec_client_with_files_of(&["2025_10"]);
        let periods = ["2025q3", "2025_10"];

        let expected_result = Err(FailedBackfill {
            failures: 1,
            periods: 2,
        });

        let result = run_batch(&root, sec_client, &periods, |_, _| {})
            .await
            .result();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_last_day_of_the_period_before_when_running_a_batch_whose_newest_period_is_unpublished()
     {
        let root = temporary_root();
        let sec_client = sec_client_with_files_of(&["2025q3"]);
        let periods = ["2025q3", "2025_10"];

        let expected_result = Ok(NaiveDate::from_ymd_opt(2025, 9, 30)
            .expect("Given a hardcoded calendar date, building it should always succeed"));

        let result = run_batch(&root, sec_client, &periods, |_, _| {})
            .await
            .cut_over_date();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_pass_each_unit_to_the_observer_when_the_run_extracts_the_units() {
        let root = temporary_root();
        let sec_client = sec_client_with_files_of(&["2025q3", "2025_10"]);
        let periods = ["2025q3", "2025_10"];
        let units = Arc::new(Mutex::new(Vec::new()));
        let observed_units = Arc::clone(&units);
        let observer = move |unit: &Unit, _: RunEvent<'_>| {
            observed_units
                .lock()
                .expect(LOCK_EXPECT)
                .push(unit.to_string());
        };

        let expected_result = vec!["2025_10".to_string(), "2025q3".to_string()];

        run_batch(&root, sec_client, &periods, observer).await;
        let mut result = units.lock().expect(LOCK_EXPECT).clone();
        result.sort();
        result.dedup();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_pass_the_absent_kind_to_the_observer_when_the_sec_answers_404() {
        let root = temporary_root();
        let sec_client = sec_client_with_files_of(&[]);
        let periods = ["2025_10"];
        let kinds = Arc::new(Mutex::new(Vec::new()));
        let observed_kinds = Arc::clone(&kinds);
        let observer = move |_: &Unit, event: RunEvent<'_>| {
            if let RunEvent::Error(_, kind) = event {
                observed_kinds.lock().expect(LOCK_EXPECT).push(kind);
            }
        };

        let expected_result = vec![ErrorKind::Absent];

        run_batch(&root, sec_client, &periods, observer).await;
        let result = kinds.lock().expect(LOCK_EXPECT).clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_pass_the_failed_kind_to_the_observer_when_the_period_is_invalid() {
        let root = temporary_root();
        let sec_client = sec_client_with_files_of(&[]);
        let periods = ["not_a_period"];
        let kinds = Arc::new(Mutex::new(Vec::new()));
        let observed_kinds = Arc::clone(&kinds);
        let observer = move |_: &Unit, event: RunEvent<'_>| {
            if let RunEvent::Error(_, kind) = event {
                observed_kinds.lock().expect(LOCK_EXPECT).push(kind);
            }
        };

        let expected_result = vec![ErrorKind::Failed];

        run_batch(&root, sec_client, &periods, observer).await;
        let result = kinds.lock().expect(LOCK_EXPECT).clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_call_the_observer_before_the_next_period_starts_when_one_period_runs_at_once() {
        let root = temporary_root();
        let sec_client = sec_client_with_files_of(&["2025q3", "2025_10"]);
        let periods = ["2025q3", "2025_10"];
        let later_file = root.path().join("sec/notes/2025_10_notes.zip");
        let later_file_seen = Arc::new(Mutex::new(Vec::new()));
        let seen_by_observer = Arc::clone(&later_file_seen);
        let observer = move |unit: &Unit, _: RunEvent<'_>| {
            if unit.to_string() == "2025q3" {
                seen_by_observer
                    .lock()
                    .expect(LOCK_EXPECT)
                    .push(later_file.exists());
            }
        };

        let expected_result = false;

        Extraction::backfill()
            .notes()
            .periods(periods)
            .sec_client(sec_client)
            .repository(writer(&root))
            .in_flight(1)
            .run(observer)
            .await;
        let result = later_file_seen
            .lock()
            .expect(LOCK_EXPECT)
            .iter()
            .any(|seen| *seen);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_every_period_when_in_flight_is_zero() {
        let root = temporary_root();
        let sec_client = sec_client_with_files_of(&["2025q3", "2025_10"]);
        let periods = ["2025q3", "2025_10"];

        let expected_result = 2;

        let result = Extraction::backfill()
            .notes()
            .periods(periods)
            .sec_client(sec_client)
            .repository(writer(&root))
            .in_flight(0)
            .run(|_, _| {})
            .await
            .success_count();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_execution_id_and_the_backfill_mode_when_running_a_batch() {
        let root = temporary_root();
        let sec_client = sec_client_with_files_of(&["2025q3"]);

        let expected_result = (
            json!("00000000-0000-0000-0000-000000000007"),
            json!("backfill"),
        );

        Extraction::backfill()
            .execution_id(Uuid::from_u128(7))
            .notes()
            .periods(["2025q3"])
            .sec_client(sec_client)
            .repository(writer(&root))
            .run(|_, _| {})
            .await;
        let metadata = std::fs::read(root.path().join("sec/notes/2025q3_notes.zip.meta.json"))
            .expect("Given a run that stored the file, the metadata file should always exist");
        let metadata: Value = serde_json::from_slice(&metadata)
            .expect("Given metadata written by the repository, it should always be valid JSON");
        let result = (
            metadata["execution_id"].clone(),
            metadata["ingestion_mode"].clone(),
        );

        assert_eq!(result, expected_result);
    }
}
