//! # All
//!
//! Provides [`BackfillAll`], the pipeline of a backfill run over every source, and its
//! [`AllRunSummary`].
//!
//! [`BackfillAll::periods`] returns an [`AllBatchBuilder`], whose [`run`] runs two steps. The Notes
//! step runs [`BackfillNotes`](super::BackfillNotes) over the periods. The `submissions` step
//! reads the CIKs of each stored Notes file and runs [`BackfillSubmissions`](super::BackfillSubmissions)
//! over the distinct CIKs.
//!
//! [`run`]: AllBatchBuilder::run

use std::collections::BTreeSet;
use std::sync::Arc;

use storage::ReadError;
use storage::implementations::repository::filesystem::{
    Document, FilesystemRepository, Read, Write,
};
use storage::traits::repository::ReadRepository;
use thiserror::Error;

use super::Backfill;
use super::daily_filings::NoRepositories;
use super::run_event::{RunEvent, Unit};
use crate::implementations::states::extract::notes::ExtractNotesSuperState;
use crate::implementations::states::extract::notes::builder::NoSecClient;
use crate::implementations::states::extract::submissions::ExtractSubmissionsSuperState;
use crate::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use crate::implementations::states::extract::validate_notes_period::ValidateNotesPeriod;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::notes_archive_ciks::{InvalidNotesArchive, distinct_ciks_of_notes_archive};
use crate::shared::notes_archive_ref::NotesArchiveRef;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::{RawNotesArchive, RawSubmissions};
use crate::shared::response::implementations::sec_response::{JsonBody, SecResponse, ZipBody};
use crate::traits::state_machine::stream::IntoStateMachineStream;
use summary::CikRead;

mod summary;

pub use summary::{AllRunSummary, FailedAllBackfill};

/// The store of the Notes files.
type NotesWriter = FilesystemRepository<Write, RawDocument<RawNotesArchive>>;

/// The reader of the stored Notes files.
type NotesReader = FilesystemRepository<Read, RawDocument<RawNotesArchive>>;

/// The store of the `submissions` files.
type SubmissionsWriter = FilesystemRepository<Write, RawDocument<RawSubmissions>>;

/// The SEC client that both steps of the pipeline accept.
// Deviation: the bound names `SecClientTrait` once per response type, because Rust 1.90 rejects
// two bindings of `Response` in one bound (E0719).
pub trait BackfillClient:
    SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>
    + SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>>
    + 'static
{
}

impl<C> BackfillClient for C where
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>
        + SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>>
        + 'static
{
}

/// The stores that the pipeline writes to.
#[derive(Debug, Clone)]
pub struct BackfillRepositories {
    /// The store of the Notes files. The `submissions` step also reads the stored files from its
    /// root.
    pub notes: NotesWriter,
    /// The store of the `submissions` files.
    pub submissions: SubmissionsWriter,
}

/// The pipeline of a backfill run over every source, before the choice of the periods.
///
/// Every super-state of the pipeline carries the `execution_id` of its [`Backfill`] handle and
/// [`IngestionMode::Backfill`](storage::implementations::repository::filesystem::IngestionMode::Backfill).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BackfillAll {
    backfill: Backfill,
}

impl BackfillAll {
    /// Creates a new [`BackfillAll`] for the run of `backfill`.
    pub(super) const fn new(backfill: Backfill) -> Self {
        Self { backfill }
    }

    /// Returns the builder of a pipeline run over `periods`, in the order of the list.
    ///
    /// The builder still needs an SEC client and the stores.
    #[must_use]
    pub fn periods<P: Into<String>>(
        self,
        periods: impl IntoIterator<Item = P>,
    ) -> AllBatchBuilder<NoSecClient, NoRepositories> {
        AllBatchBuilder {
            backfill: self.backfill,
            periods: periods.into_iter().map(Into::into).collect(),
            sec_client: NoSecClient,
            repositories: NoRepositories,
            in_flight: super::DEFAULT_FILES_IN_FLIGHT,
        }
    }
}

/// Builds and runs the pipeline of a backfill run over many Notes periods.
///
/// Set the client with [`sec_client`](Self::sec_client) and the stores with
/// [`repositories`](Self::repositories), in any order. [`in_flight`](Self::in_flight) is optional.
/// Then call [`run`](Self::run).
#[derive(Debug, Clone)]
pub struct AllBatchBuilder<C, R> {
    backfill: Backfill,
    periods: Vec<String>,
    sec_client: C,
    repositories: R,
    in_flight: usize,
}

impl<R> AllBatchBuilder<NoSecClient, R> {
    /// Sets the SEC client of both steps.
    #[must_use]
    pub fn sec_client<C: BackfillClient>(self, sec_client: C) -> AllBatchBuilder<C, R> {
        AllBatchBuilder {
            backfill: self.backfill,
            periods: self.periods,
            sec_client,
            repositories: self.repositories,
            in_flight: self.in_flight,
        }
    }
}

impl<C> AllBatchBuilder<C, NoRepositories> {
    /// Sets the stores of the Notes files and of the `submissions` files.
    #[must_use]
    pub fn repositories(
        self,
        repositories: BackfillRepositories,
    ) -> AllBatchBuilder<C, BackfillRepositories> {
        AllBatchBuilder {
            backfill: self.backfill,
            periods: self.periods,
            sec_client: self.sec_client,
            repositories,
            in_flight: self.in_flight,
        }
    }
}

impl<C, R> AllBatchBuilder<C, R> {
    /// Sets the most Notes periods that run at once. The default is
    /// [`DEFAULT_FILES_IN_FLIGHT`](super::DEFAULT_FILES_IN_FLIGHT), and `0` counts as `1`.
    ///
    /// The `submissions` step keeps its default,
    /// [`DEFAULT_CIKS_IN_FLIGHT`](super::DEFAULT_CIKS_IN_FLIGHT).
    #[must_use]
    pub fn in_flight(self, in_flight: usize) -> Self {
        Self {
            in_flight: in_flight.max(1),
            ..self
        }
    }
}

impl<C> AllBatchBuilder<C, BackfillRepositories>
where
    C: BackfillClient,
    ExtractNotesSuperState<ValidateNotesPeriod, NotesWriter, C>: IntoStateMachineStream,
    ExtractSubmissionsSuperState<ValidateCikFormat, SubmissionsWriter, C>: IntoStateMachineStream,
{
    /// Runs the Notes step, then the `submissions` step, and returns the summary of both.
    ///
    /// The Notes step follows the rules of [`NotesBatchBuilder::run`](super::NotesBatchBuilder::run).
    /// The `submissions` step reads the stored Notes files one at a time, so at most one file is
    /// in memory. It collects the distinct CIKs of the files that it can read and extracts the
    /// `submissions` file of each CIK, as [`SubmissionsBatchBuilder::run`](super::SubmissionsBatchBuilder::run)
    /// does.
    ///
    /// The `submissions` step runs even if a Notes period failed. It then reads only the files of
    /// the periods that the store holds. A Notes file that it cannot read, or a failed CIK, fails
    /// [`AllRunSummary::result`] but does not stop the other files or CIKs, and does not change
    /// [`AllRunSummary::cut_over_date`].
    ///
    /// Calls `observer` with every event of both steps.
    ///
    /// # Panics
    ///
    /// Panics if the task of a Notes period panics, with the payload of that panic.
    pub async fn run(
        self,
        observer: impl Fn(&Unit, RunEvent<'_>) + Send + Sync + 'static,
    ) -> AllRunSummary {
        let observer = Arc::new(observer);
        let notes_observer = Arc::clone(&observer);
        let reader = FilesystemRepository::builder()
            .reader()
            .document_type::<RawDocument<RawNotesArchive>>()
            .root(self.repositories.notes.root())
            .build();
        let notes = self
            .backfill
            .notes()
            .periods(self.periods)
            .sec_client(self.sec_client.clone())
            .repository(self.repositories.notes)
            .in_flight(self.in_flight)
            .run(move |unit, event| notes_observer(unit, event))
            .await;
        let (ciks, cik_reads) = read_ciks(&reader, notes.stored_archives()).await;
        let submissions = self
            .backfill
            .submissions()
            .ciks(ciks)
            .sec_client(self.sec_client)
            .repository(self.repositories.submissions)
            .run(move |unit, event| observer(unit, event))
            .await;
        AllRunSummary {
            notes,
            cik_reads,
            submissions,
        }
    }
}

/// Reads the CIKs of each stored Notes file of `archives`, one file at a time, and returns the
/// distinct CIKs of all files and the read of each file.
///
/// A file that cannot be read gives a failed [`CikRead`] and does not stop the other files.
async fn read_ciks<'a>(
    reader: &NotesReader,
    archives: impl IntoIterator<Item = &'a NotesArchiveRef>,
) -> (BTreeSet<String>, Vec<CikRead>) {
    let mut ciks = BTreeSet::new();
    let mut cik_reads = Vec::new();
    for archive in archives {
        let read = ciks_of_stored_archive(reader, archive).await;
        cik_reads.push(CikRead {
            period: archive.period().to_string(),
            result: read.map(|archive_ciks| {
                let count = archive_ciks.len();
                ciks.extend(archive_ciks);
                count
            }),
        });
    }
    (ciks, cik_reads)
}

/// Returns the distinct CIKs of the stored Notes file of `archive`.
///
/// # Errors
///
/// Returns a [`FailedCikRead`] if `reader` cannot read the file, holds no file under its key, or
/// the file is an [`InvalidNotesArchive`].
async fn ciks_of_stored_archive(
    reader: &NotesReader,
    archive: &NotesArchiveRef,
) -> Result<BTreeSet<String>, FailedCikRead> {
    let document = reader
        .get(archive.key().clone())
        .await?
        .ok_or(FailedCikRead::MissingFile)?;
    Ok(distinct_ciks_of_notes_archive(document.bytes())?)
}

/// Error indicating that the `submissions` step could not read the CIKs of a stored Notes file.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum FailedCikRead {
    /// The store could not read the file, or the file failed the size or SHA-256 check.
    #[error("[FailedCikRead] Failed to read the stored Notes file, Caused by: {0}")]
    Read(#[from] ReadError),
    /// The store holds no file under the key of the stored Notes file.
    #[error(
        "[FailedCikRead] Failed to read the stored Notes file, Reason: 'the store holds no file under its key'"
    )]
    MissingFile,
    /// The file holds no readable CIK list.
    #[error("[FailedCikRead] Failed to read the CIKs of the stored Notes file, Caused by: {0}")]
    InvalidArchive(#[from] InvalidNotesArchive),
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::extraction::Extraction;
    use crate::tests::fixtures::data::notes_archive::{SAMPLE_NOTES_ZIP, notes_zip};
    use crate::tests::fixtures::data::submissions::SAMPLE_SUBMISSIONS_JSON;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;

    const NOTES_URL: &str =
        "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/";
    const SUBMISSIONS_URL: &str = "https://data.sec.gov/submissions/CIK";

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn repositories(root: &TempDir) -> BackfillRepositories {
        BackfillRepositories {
            notes: FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<RawNotesArchive>>()
                .root(root.path())
                .build(),
            submissions: FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<RawSubmissions>>()
                .root(root.path())
                .build(),
        }
    }

    /// Returns a fake SEC client that answers the Notes file of each period of `notes_files` with
    /// its bytes, and the `submissions` file of each padded CIK of `padded_ciks`. Every other URL
    /// answers 404.
    fn sec_client(notes_files: &[(&str, &[u8])], padded_ciks: &[&str]) -> RoutingSecClient {
        let client =
            notes_files
                .iter()
                .fold(RoutingSecClient::new(), |client, (period, archive)| {
                    client.route(
                        &format!("{NOTES_URL}{period}_notes.zip"),
                        200,
                        "application/zip",
                        archive,
                    )
                });
        padded_ciks.iter().fold(client, |client, cik| {
            client.route(
                &format!("{SUBMISSIONS_URL}{cik}.json"),
                200,
                "application/json",
                SAMPLE_SUBMISSIONS_JSON,
            )
        })
    }

    /// Runs the pipeline over `periods` into `root` without an observer.
    async fn run_all(
        root: &TempDir,
        sec_client: RoutingSecClient,
        periods: &[&str],
    ) -> AllRunSummary {
        Extraction::backfill()
            .all()
            .periods(periods.iter().copied())
            .sec_client(sec_client)
            .repositories(repositories(root))
            .run(|_, _| {})
            .await
    }

    /// Returns the names of the `submissions` files under `root`, sorted, without the metadata
    /// files.
    fn stored_submissions_files(root: &TempDir) -> Vec<String> {
        let entries = std::fs::read_dir(root.path().join("sec/submissions")).expect(
            "Given a run that stored a submissions file, the submissions directory should always exist",
        );
        let mut names: Vec<String> = entries
            .map(|entry| {
                entry
                    .expect("Given a readable directory, each entry should always be readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .filter(|name| !name.ends_with(".meta.json"))
            .collect();
        names.sort();
        names
    }

    const AUGUST_FILE: &str = "sec/notes/2026_08_notes.zip";

    /// Runs the two steps of the pipeline over `2026_08` and `2026_09`, with one CIK per Notes
    /// file. Between the steps, `damage` changes the stored files under `root`, as a disk or
    /// another process can.
    async fn run_with_damaged_files(
        root: &TempDir,
        damage: impl FnOnce(&TempDir),
    ) -> AllRunSummary {
        let august = notes_zip("adsh\tcik\n0001067983-26-000002\t1067983\n");
        let september = notes_zip("adsh\tcik\n0000320193-26-000003\t320193\n");
        let sec_client = sec_client(
            &[("2026_08", &august), ("2026_09", &september)],
            &["0000320193", "0001067983"],
        );
        let repositories = repositories(root);
        let reader = FilesystemRepository::builder()
            .reader()
            .document_type::<RawDocument<RawNotesArchive>>()
            .root(root.path())
            .build();
        let notes = Extraction::backfill()
            .notes()
            .periods(["2026_08", "2026_09"])
            .sec_client(sec_client.clone())
            .repository(repositories.notes)
            .run(|_, _| {})
            .await;
        damage(root);
        let (ciks, cik_reads) = read_ciks(&reader, notes.stored_archives()).await;
        let submissions = Extraction::backfill()
            .submissions()
            .ciks(ciks)
            .sec_client(sec_client)
            .repository(repositories.submissions)
            .run(|_, _| {})
            .await;
        AllRunSummary {
            notes,
            cik_reads,
            submissions,
        }
    }

    /// Deletes the stored Notes file of `2026_08` and its metadata.
    fn delete_august_file(root: &TempDir) {
        for file in [AUGUST_FILE.to_string(), format!("{AUGUST_FILE}.meta.json")] {
            std::fs::remove_file(root.path().join(file))
                .expect("Given a run that stored the file, deleting it should always succeed");
        }
    }

    /// Replaces the stored Notes file of `2026_08` with other bytes, so the SHA-256 check fails.
    fn corrupt_august_file(root: &TempDir) {
        std::fs::write(root.path().join(AUGUST_FILE), b"PK")
            .expect("Given a run that stored the file, overwriting it should always succeed");
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, building it should always succeed")
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_backfill_all() {
        implements_auto_traits::<BackfillAll>();
    }

    #[test]
    const fn should_implement_auto_traits_for_all_batch_builder() {
        implements_auto_traits::<AllBatchBuilder<RoutingSecClient, BackfillRepositories>>();
    }

    #[test]
    const fn should_implement_auto_traits_for_all_run_summary() {
        implements_auto_traits::<AllRunSummary>();
    }

    #[tokio::test]
    async fn should_store_one_submissions_file_per_distinct_cik_when_two_notes_files_share_a_cik() {
        let root = temporary_root();
        let august =
            notes_zip("adsh\tcik\n0000320193-26-000001\t320193\n0001067983-26-000002\t1067983\n");
        let september =
            notes_zip("adsh\tcik\n0000320193-26-000003\t320193\n0000789019-26-000004\t789019\n");
        let sec_client = sec_client(
            &[("2026_08", &august), ("2026_09", &september)],
            &["0000320193", "0000789019", "0001067983"],
        );

        let expected_result = vec![
            "CIK0000320193.json".to_string(),
            "CIK0000789019.json".to_string(),
            "CIK0001067983.json".to_string(),
        ];

        run_all(&root, sec_client, &["2026_08", "2026_09"]).await;
        let result = stored_submissions_files(&root);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_all_backfill_when_a_cik_fails() {
        let root = temporary_root();
        let september =
            notes_zip("adsh\tcik\n0000320193-26-000003\t320193\n0000789019-26-000004\t789019\n");
        let sec_client = sec_client(&[("2026_09", &september)], &["0000320193"]);

        let expected_result = Err(FailedAllBackfill {
            failed_periods: 0,
            periods: 1,
            unreadable_files: 0,
            stored_files: 1,
            failed_ciks: 1,
            ciks: 2,
        });

        let result = run_all(&root, sec_client, &["2026_09"]).await.result();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_cut_over_date_of_the_notes_step_when_a_cik_fails() {
        let root = temporary_root();
        let september =
            notes_zip("adsh\tcik\n0000320193-26-000003\t320193\n0000789019-26-000004\t789019\n");
        let sec_client = sec_client(&[("2026_09", &september)], &["0000320193"]);

        let expected_result = Ok(date(2026, 9, 30));

        let result = run_all(&root, sec_client, &["2026_09"])
            .await
            .cut_over_date();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_read_the_other_files_when_a_stored_notes_file_has_no_sub_tsv() {
        let root = temporary_root();
        let september = notes_zip("adsh\tcik\n0000320193-26-000003\t320193\n");
        let sec_client = sec_client(
            &[("2026_08", SAMPLE_NOTES_ZIP), ("2026_09", &september)],
            &["0000320193"],
        );

        let expected_result = vec![
            json!({
                "period": "2026_08",
                "error": "[FailedCikRead] Failed to read the CIKs of the stored Notes file, Caused by: [InvalidNotesArchive] Invalid Notes file, Reason: 'no entry sub.tsv'",
            }),
            json!({ "period": "2026_09", "ciks": 1 }),
        ];

        let result = run_all(&root, sec_client, &["2026_08", "2026_09"])
            .await
            .cik_read_entries();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_all_backfill_when_a_stored_notes_file_is_unreadable() {
        let root = temporary_root();
        let sec_client = sec_client(&[("2026_09", SAMPLE_NOTES_ZIP)], &[]);

        let expected_result = Err(FailedAllBackfill {
            failed_periods: 0,
            periods: 1,
            unreadable_files: 1,
            stored_files: 1,
            failed_ciks: 0,
            ciks: 0,
        });

        let result = run_all(&root, sec_client, &["2026_09"]).await.result();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_cut_over_date_of_the_notes_step_when_a_stored_notes_file_is_unreadable()
     {
        let root = temporary_root();
        let sec_client = sec_client(&[("2026_09", SAMPLE_NOTES_ZIP)], &[]);

        let expected_result = Ok(date(2026, 9, 30));

        let result = run_all(&root, sec_client, &["2026_09"])
            .await
            .cut_over_date();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_submissions_of_the_stored_periods_when_a_notes_period_fails() {
        let root = temporary_root();
        let september = notes_zip("adsh\tcik\n0000320193-26-000003\t320193\n");
        let sec_client = sec_client(&[("2026_09", &september)], &["0000320193"]);

        let expected_result = vec!["CIK0000320193.json".to_string()];

        run_all(&root, sec_client, &["2026_08", "2026_09"]).await;
        let result = stored_submissions_files(&root);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_read_the_other_file_when_a_stored_notes_file_is_missing() {
        let root = temporary_root();

        let expected_result = vec![
            json!({
                "period": "2026_08",
                "error": "[FailedCikRead] Failed to read the stored Notes file, Reason: 'the store holds no file under its key'",
            }),
            json!({ "period": "2026_09", "ciks": 1 }),
        ];

        let result = run_with_damaged_files(&root, delete_august_file)
            .await
            .cik_read_entries();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_all_backfill_when_a_stored_notes_file_is_missing() {
        let root = temporary_root();

        let expected_result = Err(FailedAllBackfill {
            failed_periods: 0,
            periods: 2,
            unreadable_files: 1,
            stored_files: 2,
            failed_ciks: 0,
            ciks: 1,
        });

        let result = run_with_damaged_files(&root, delete_august_file)
            .await
            .result();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_read_the_other_file_when_a_stored_notes_file_fails_the_sha256_check() {
        let root = temporary_root();

        let expected_result = (true, json!({ "period": "2026_09", "ciks": 1 }));

        let entries = run_with_damaged_files(&root, corrupt_august_file)
            .await
            .cik_read_entries();
        let result = (
            entries[0]["error"].as_str().is_some_and(|error| {
                error.starts_with(
                    "[FailedCikRead] Failed to read the stored Notes file, Caused by: ",
                )
            }),
            entries[1].clone(),
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_all_backfill_when_a_stored_notes_file_fails_the_sha256_check() {
        let root = temporary_root();

        let expected_result = Err(FailedAllBackfill {
            failed_periods: 0,
            periods: 2,
            unreadable_files: 1,
            stored_files: 2,
            failed_ciks: 0,
            ciks: 1,
        });

        let result = run_with_damaged_files(&root, corrupt_august_file)
            .await
            .result();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_cut_over_date_of_the_notes_step_when_a_stored_notes_file_fails_the_sha256_check()
     {
        let root = temporary_root();

        let expected_result = Ok(date(2026, 9, 30));

        let result = run_with_damaged_files(&root, corrupt_august_file)
            .await
            .cut_over_date();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_use_the_notes_default_cap_when_in_flight_is_not_set() {
        let periods = ["2026_09"];

        let expected_result = 2;

        let result = Extraction::backfill().all().periods(periods).in_flight;

        assert_eq!(result, expected_result);
    }
}
