//! # All
//!
//! Provides [`BackfillAll`], the pipeline of a backfill run over every source, and its
//! [`AllRunSummary`].
//!
//! [`BackfillAll::periods`] returns an [`AllBatchBuilder`]. Its [`run`] first finds the cut-over
//! date: the last day of the newest Notes period that the SEC publishes, confirmed with one `HEAD`
//! request. It then runs two steps at the same time. The Notes step runs
//! [`BackfillNotes`](super::BackfillNotes) over the periods. The catch-up runs
//! [`IncrementalDailyFilings`](super::IncrementalDailyFilings) for each date from
//! [`CATCH_UP_LOOKBACK_DAYS`] days before the cut-over date to the date of
//! [`until`](AllBatchBuilder::until). It finds the filings of those dates in the full index of each
//! quarter that the dates touch. Both steps carry the `execution_id` of the run.
//!
//! [`run`]: AllBatchBuilder::run

use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{Days, NaiveDate};
use futures_util::future::join;
use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
use thiserror::Error;

use super::Backfill;
use super::daily_filings::{DailyClient, DailyFilingsRepositories, NoRepositories};
use super::notes::published_periods::{FailedNotesProbe, is_published};
use super::retry::DEFAULT_RETRY_DELAYS;
use super::run_event::{RunEvent, Unit};
use crate::implementations::states::extract::daily_index::DetectFilingsSuperState;
use crate::implementations::states::extract::filing::ExtractFilingSuperState;
use crate::implementations::states::extract::full_index::DetectQuarterFilingsSuperState;
use crate::implementations::states::extract::notes::ExtractNotesSuperState;
use crate::implementations::states::extract::notes::builder::NoSecClient;
use crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
use crate::implementations::states::extract::submissions::ExtractSubmissionsSuperState;
use crate::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use crate::implementations::states::extract::validate_index_date::ValidateIndexDate;
use crate::implementations::states::extract::validate_notes_period::ValidateNotesPeriod;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::{
    RawDailyIndex, RawFilingIndex, RawFullIndex, RawNotesArchive, RawSubmissions, RawXbrlInstance,
    RawXbrlLabels,
};
use crate::shared::request::implementations::sec_request::{FilingIndex, FullIndex};
use crate::shared::response::implementations::sec_response::{SecResponse, ZipHeadBody};
use crate::traits::state_machine::stream::IntoStateMachineStream;

mod summary;

pub use summary::{AllRunSummary, FailedAllBackfill};

/// The store of the Notes files.
type NotesWriter = FilesystemRepository<Write, RawDocument<RawNotesArchive>>;

/// The store of the full indexes of the catch-up.
type FullIndexWriter = FilesystemRepository<Write, RawDocument<RawFullIndex>>;

/// The number of days before the cut-over date at which the catch-up starts.
///
/// The full index lists a filing with the date filed of the last time the SEC sent it. A filing
/// that the SEC sends after the cut-over date can carry an earlier date filed, and a range that
/// starts at the cut-over date would miss it. ADR 0003, decision 4, records the measured lag. The
/// overlap with the Notes files costs only requests, because every write is idempotent and a
/// repeat run skips the stored filings.
pub const CATCH_UP_LOOKBACK_DAYS: u64 = 7;

/// The SEC client that the cut-over probe and both steps of the pipeline accept.
pub trait AllClient:
    DailyClient
    + SecClientTrait<Response<ZipHeadBody> = SecResponse<ZipHeadBody>, Error = FailedSecRequest>
{
}

impl<C> AllClient for C where
    C: DailyClient
        + SecClientTrait<Response<ZipHeadBody> = SecResponse<ZipHeadBody>, Error = FailedSecRequest>
{
}

/// The stores that the pipeline writes to.
#[derive(Debug, Clone)]
pub struct BackfillRepositories {
    /// The store of the Notes files.
    pub notes: NotesWriter,
    /// The stores of the catch-up.
    pub daily_filings: DailyFilingsRepositories,
    /// The store of the full indexes that the catch-up reads.
    pub full_index: FullIndexWriter,
}

/// Marks an [`AllBatchBuilder`] that holds no last date of the catch-up yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoUntil;

/// Error indicating that the run found no cut-over date, so neither step ran.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum FailedCutOverDate {
    /// The `HEAD` request for the newest Notes period failed.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    Probe(#[from] FailedNotesProbe),
    /// The run lists no valid Notes period.
    #[error(
        "[FailedCutOverDate] Failed to find the cut-over date, Reason: 'the run lists no valid Notes period'"
    )]
    MissingPeriod,
    /// The newest Notes period is unpublished, and the run lists no older period.
    #[error(
        "[FailedCutOverDate] Failed to find the cut-over date, Reason: 'the newest Notes period is unpublished, and the run lists no older period'"
    )]
    MissingPublishedPeriod,
}

/// The pipeline of a backfill run over every source, before the choice of the periods.
///
/// Every super-state of the Notes step carries the `execution_id` of its [`Backfill`] handle and
/// [`IngestionMode::Backfill`](storage::implementations::repository::filesystem::IngestionMode::Backfill).
/// Every super-state of the catch-up carries the same `execution_id` and
/// [`IngestionMode::Incremental`](storage::implementations::repository::filesystem::IngestionMode::Incremental).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BackfillAll {
    backfill: Backfill,
}

impl BackfillAll {
    /// Creates a new [`BackfillAll`] for the run of `backfill`.
    pub(super) const fn new(backfill: Backfill) -> Self {
        Self { backfill }
    }

    /// Returns the builder of a pipeline run over `periods`.
    ///
    /// The builder still needs the last date of the catch-up, an SEC client, and the stores.
    #[must_use]
    pub fn periods<P: Into<String>>(
        self,
        periods: impl IntoIterator<Item = P>,
    ) -> AllBatchBuilder<NoUntil, NoSecClient, NoRepositories> {
        AllBatchBuilder {
            backfill: self.backfill,
            periods: periods.into_iter().map(Into::into).collect(),
            until: NoUntil,
            sec_client: NoSecClient,
            repositories: NoRepositories,
            in_flight: super::DEFAULT_FILES_IN_FLIGHT,
            retry_delays: DEFAULT_RETRY_DELAYS.to_vec(),
        }
    }
}

/// Builds and runs the pipeline of a backfill run: the Notes periods and the catch-up.
///
/// Set the last date of the catch-up with [`until`](Self::until), the client with
/// [`sec_client`](Self::sec_client), and the stores with [`repositories`](Self::repositories), in
/// any order. [`in_flight`](Self::in_flight) and [`retry_delays`](Self::retry_delays) are
/// optional. Then call [`run`](Self::run).
///
/// `run` needs the last date of the catch-up:
///
/// ```compile_fail,E0599
/// use sec::implementations::extraction::{BackfillRepositories, Extraction};
/// use sec::shared::http_client::implementations::sec_client::SecClient;
///
/// fn run_without_until(repositories: BackfillRepositories) {
///     let _run = Extraction::backfill()
///         .all()
///         .periods(["2026_09"])
///         .sec_client(SecClient::default())
///         .repositories(repositories)
///         .run(|_, _| {});
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AllBatchBuilder<U, C, R> {
    backfill: Backfill,
    periods: Vec<String>,
    until: U,
    sec_client: C,
    repositories: R,
    in_flight: usize,
    retry_delays: Vec<Duration>,
}

impl<C, R> AllBatchBuilder<NoUntil, C, R> {
    /// Sets the last date of the catch-up. The catch-up also counts back from it for its pending
    /// rule, as [`DailyFilingsBuilder::as_of`](super::DailyFilingsBuilder::as_of) does.
    #[must_use]
    pub fn until(self, until: NaiveDate) -> AllBatchBuilder<NaiveDate, C, R> {
        AllBatchBuilder {
            backfill: self.backfill,
            periods: self.periods,
            until,
            sec_client: self.sec_client,
            repositories: self.repositories,
            in_flight: self.in_flight,
            retry_delays: self.retry_delays,
        }
    }
}

impl<U, R> AllBatchBuilder<U, NoSecClient, R> {
    /// Sets the SEC client of the cut-over probe and of both steps.
    #[must_use]
    pub fn sec_client<C: AllClient>(self, sec_client: C) -> AllBatchBuilder<U, C, R> {
        AllBatchBuilder {
            backfill: self.backfill,
            periods: self.periods,
            until: self.until,
            sec_client,
            repositories: self.repositories,
            in_flight: self.in_flight,
            retry_delays: self.retry_delays,
        }
    }
}

impl<U, C> AllBatchBuilder<U, C, NoRepositories> {
    /// Sets the store of the Notes files and the stores of the catch-up.
    #[must_use]
    pub fn repositories(
        self,
        repositories: BackfillRepositories,
    ) -> AllBatchBuilder<U, C, BackfillRepositories> {
        AllBatchBuilder {
            backfill: self.backfill,
            periods: self.periods,
            until: self.until,
            sec_client: self.sec_client,
            repositories,
            in_flight: self.in_flight,
            retry_delays: self.retry_delays,
        }
    }
}

impl<U, C, R> AllBatchBuilder<U, C, R> {
    /// Sets the most Notes periods that run at once. The default is
    /// [`DEFAULT_FILES_IN_FLIGHT`](super::DEFAULT_FILES_IN_FLIGHT), and `0` counts as `1`.
    ///
    /// The catch-up keeps its default,
    /// [`DEFAULT_MACHINES_IN_FLIGHT`](super::DEFAULT_MACHINES_IN_FLIGHT).
    #[must_use]
    pub fn in_flight(self, in_flight: usize) -> Self {
        Self {
            in_flight: in_flight.max(1),
            ..self
        }
    }

    /// Sets the waits before each new attempt of the cut-over probe, a Notes period, or a machine
    /// of the catch-up that failed with a transient SEC failure. The default is
    /// [`DEFAULT_RETRY_DELAYS`].
    #[must_use]
    pub fn retry_delays(self, retry_delays: impl IntoIterator<Item = Duration>) -> Self {
        Self {
            retry_delays: retry_delays.into_iter().collect(),
            ..self
        }
    }
}

impl<C> AllBatchBuilder<NaiveDate, C, BackfillRepositories>
where
    C: AllClient,
    ExtractNotesSuperState<ValidateNotesPeriod, NotesWriter, C>: IntoStateMachineStream,
    DetectFilingsSuperState<
        ValidateIndexDate,
        FilesystemRepository<Write, RawDocument<RawDailyIndex>>,
        C,
    >: IntoStateMachineStream,
    DetectQuarterFilingsSuperState<PrepareSecRequest<FullIndex>, FullIndexWriter, C>:
        IntoStateMachineStream,
    ExtractFilingSuperState<
        PrepareSecRequest<FilingIndex>,
        FilesystemRepository<Write, RawDocument<RawFilingIndex>>,
        FilesystemRepository<Write, RawDocument<RawXbrlInstance>>,
        FilesystemRepository<Write, RawDocument<RawXbrlLabels>>,
        C,
    >: IntoStateMachineStream,
    ExtractSubmissionsSuperState<
        ValidateCikFormat,
        FilesystemRepository<Write, RawDocument<RawSubmissions>>,
        C,
    >: IntoStateMachineStream,
{
    /// Finds the cut-over date, then runs the Notes step and the catch-up at the same time, and
    /// returns the summary of both.
    ///
    /// The run sorts the periods, oldest first. The cut-over date is the last day of the newest
    /// Notes period if the SEC answers
    /// a `HEAD` request for its file. If the SEC answers 404, the cut-over date is the last day of
    /// the next older period of the run. The date of today plays no part.
    ///
    /// The Notes step follows the rules of [`NotesBatchBuilder::run`](super::NotesBatchBuilder::run).
    /// The catch-up runs each date from [`CATCH_UP_LOOKBACK_DAYS`] days before the cut-over date to
    /// the date of [`until`](Self::until), as
    /// [`DailyFilingsBuilder::run_from_full_index`](super::DailyFilingsBuilder::run_from_full_index)
    /// does: one full index per quarter, not one daily index per date. A failed Notes period does
    /// not stop the catch-up. It fails [`AllRunSummary::result`], and a later run downloads
    /// the period again.
    ///
    /// Calls `observer` with every event of both steps. The retries of the probe send no event.
    ///
    /// # Errors
    ///
    /// Returns a [`FailedCutOverDate`] if the run finds no cut-over date. Neither step then runs.
    ///
    /// # Panics
    ///
    /// Panics if the task of a Notes period panics, with the payload of that panic.
    pub async fn run(
        self,
        observer: impl Fn(&Unit, RunEvent<'_>) + Send + Sync + 'static,
    ) -> Result<AllRunSummary, FailedCutOverDate> {
        let mut periods = self.periods;
        // The Notes summary counts only the last period of the list as unpublished, so the newest
        // period must be last. An invalid period sorts first and still fails.
        periods.sort_by_key(|period| NotesPeriod::new(period).ok());
        let CutOver {
            date: cut_over_date,
            newest_published,
        } = cut_over_date(&self.sec_client, &periods, &self.retry_delays).await?;
        let catch_up_dates = dates_between(catch_up_start(cut_over_date), self.until);
        let observer = Arc::new(observer);
        let notes_observer = Arc::clone(&observer);
        let notes = self
            .backfill
            .notes()
            .periods(periods)
            .sec_client(self.sec_client.clone())
            .repository(self.repositories.notes)
            .in_flight(self.in_flight)
            .retry_delays(self.retry_delays.clone())
            .run(move |unit, event| notes_observer(unit, event));
        let catch_up = super::Extraction::incremental()
            .execution_id(self.backfill.id())
            .daily_filings()
            .dates(catch_up_dates.iter().copied())
            .as_of(self.until)
            .sec_client(self.sec_client)
            .repositories(self.repositories.daily_filings)
            .retry_delays(self.retry_delays)
            .run_from_full_index(self.repositories.full_index, move |unit, event| {
                observer(unit, event);
            });
        let ((notes, notes_duration), (catch_up, catch_up_duration)) =
            join(timed(notes), timed(catch_up)).await;
        Ok(AllRunSummary {
            notes,
            notes_duration,
            cut_over_date,
            newest_published,
            catch_up_dates,
            catch_up,
            catch_up_duration,
        })
    }
}

/// Returns the cut-over date of `periods`: the last day of the newest valid period if the SEC
/// publishes its file, else the last day of the next older valid period. Also returns whether the
/// SEC publishes the file of the newest period.
///
/// Sends one `HEAD` request, and repeats it after each wait of `retry_delays` while it fails with
/// a transient SEC failure.
///
/// # Errors
///
/// Returns [`FailedCutOverDate::MissingPeriod`] if `periods` holds no valid period, and
/// [`FailedCutOverDate::MissingPublishedPeriod`] if the newest period is unpublished and no older
/// one is listed. Returns [`FailedCutOverDate::Probe`] if the request fails with an answer other
/// than 200 or 404.
async fn cut_over_date<C: AllClient>(
    sec_client: &C,
    periods: &[String],
    retry_delays: &[Duration],
) -> Result<CutOver, FailedCutOverDate> {
    let mut valid: Vec<NotesPeriod> = periods
        .iter()
        .filter_map(|period| NotesPeriod::new(period).ok())
        .collect();
    valid.sort_unstable();
    valid.dedup();
    let newest = valid.pop().ok_or(FailedCutOverDate::MissingPeriod)?;
    if is_published(sec_client, newest, retry_delays, &|_, _| {}).await? {
        return Ok(CutOver {
            date: newest.last_day(),
            newest_published: true,
        });
    }
    valid
        .pop()
        .map(|older| CutOver {
            date: older.last_day(),
            newest_published: false,
        })
        .ok_or(FailedCutOverDate::MissingPublishedPeriod)
}

/// The cut-over date, and whether the `HEAD` request found the file of the newest period.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CutOver {
    date: NaiveDate,
    newest_published: bool,
}

/// Returns the first date of the catch-up: [`CATCH_UP_LOOKBACK_DAYS`] days before `cut_over`, or
/// `cut_over` itself if that date lies before the first date that [`NaiveDate`] holds.
fn catch_up_start(cut_over: NaiveDate) -> NaiveDate {
    cut_over
        .checked_sub_days(Days::new(CATCH_UP_LOOKBACK_DAYS))
        .unwrap_or(cut_over)
}

/// Returns every date from `cut_over` to `until`, both included, oldest first.
///
/// If `cut_over` comes after `until`, the list is empty.
fn dates_between(cut_over: NaiveDate, until: NaiveDate) -> Vec<NaiveDate> {
    cut_over
        .iter_days()
        .take_while(|date| *date <= until)
        .collect()
}

/// Awaits `step` and returns its output with the time it took.
async fn timed<T>(step: impl Future<Output = T>) -> (T, Duration) {
    let start = Instant::now();
    let output = step.await;
    (output, start.elapsed())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::extraction::Extraction;
    use crate::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;

    const NOTES_URL: &str =
        "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/";

    /// No wait between two attempts, so a test with a retry takes no time.
    const NO_WAIT: [Duration; 1] = [Duration::ZERO];

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, building it should always succeed")
    }

    fn repositories(root: &TempDir) -> BackfillRepositories {
        BackfillRepositories {
            notes: FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<_>>()
                .root(root.path())
                .build(),
            daily_filings: DailyFilingsRepositories {
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
            full_index: FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<_>>()
                .root(root.path())
                .build(),
        }
    }

    /// Returns a client that answers 200 with a Notes file for each of `periods`. Every other URL
    /// answers 404.
    fn sec_with_periods(periods: &[&str]) -> RoutingSecClient {
        periods
            .iter()
            .fold(RoutingSecClient::new(), |sec_client, period| {
                sec_client.route(
                    &format!("{NOTES_URL}{period}_notes.zip"),
                    200,
                    "application/zip",
                    SAMPLE_NOTES_ZIP,
                )
            })
    }

    fn periods(periods: &[&str]) -> Vec<String> {
        periods.iter().map(ToString::to_string).collect()
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_backfill_all() {
        implements_auto_traits::<BackfillAll>();
    }

    #[test]
    const fn should_implement_auto_traits_for_all_batch_builder() {
        implements_auto_traits::<AllBatchBuilder<NaiveDate, RoutingSecClient, BackfillRepositories>>(
        );
    }

    #[test]
    const fn should_implement_auto_traits_for_all_run_summary() {
        implements_auto_traits::<AllRunSummary>();
    }

    #[test]
    const fn should_implement_auto_traits_for_failed_cut_over_date() {
        implements_auto_traits::<FailedCutOverDate>();
    }

    #[tokio::test]
    async fn should_return_the_last_day_of_the_newest_period_when_the_sec_publishes_its_file() {
        let sec_client = sec_with_periods(&["2026_09"]);
        let periods = periods(&["2026_08", "2026_09"]);

        let expected_result = Ok(CutOver {
            date: date(2026, 9, 30),
            newest_published: true,
        });

        let result = cut_over_date(&sec_client, &periods, &NO_WAIT).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_last_day_of_the_older_period_when_the_newest_period_is_unpublished()
    {
        let sec_client = sec_with_periods(&[]);
        let periods = periods(&["2026_08", "2026_09"]);

        let expected_result = Ok(CutOver {
            date: date(2026, 8, 31),
            newest_published: false,
        });

        let result = cut_over_date(&sec_client, &periods, &NO_WAIT).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_probe_the_newest_period_when_the_list_is_not_in_order() {
        let sec_client = sec_with_periods(&["2026_09"]);
        let periods = periods(&["2026_09", "2025q3", "2026_08"]);

        let expected_result = Ok(CutOver {
            date: date(2026, 9, 30),
            newest_published: true,
        });

        let result = cut_over_date(&sec_client, &periods, &NO_WAIT).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_period_when_no_period_is_valid() {
        let sec_client = sec_with_periods(&[]);
        let periods = periods(&["not a period"]);

        let expected_result = Err(FailedCutOverDate::MissingPeriod);

        let result = cut_over_date(&sec_client, &periods, &NO_WAIT).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_published_period_when_the_only_period_is_unpublished() {
        let sec_client = sec_with_periods(&[]);
        let periods = periods(&["2026_10"]);

        let expected_result = Err(FailedCutOverDate::MissingPublishedPeriod);

        let result = cut_over_date(&sec_client, &periods, &NO_WAIT).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_probe_error_when_the_sec_answers_403() {
        let sec_client = RoutingSecClient::new().route(
            &format!("{NOTES_URL}2026_09_notes.zip"),
            403,
            "application/xml",
            b"",
        );
        let periods = periods(&["2026_08", "2026_09"]);

        let expected_result = Some("[FailedNotesProbe] Failed to ask whether the Notes file of '2026_09' exists, Caused by: [InvalidSecResponse] Invalid SEC Response, Reason: 'Status code '403' is not accepted'".to_string());

        let result = cut_over_date(&sec_client, &periods, &NO_WAIT)
            .await
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_send_no_notes_request_when_the_probe_fails() {
        let root = temporary_root();
        let sec_client = RoutingSecClient::new().route(
            &format!("{NOTES_URL}2026_09_notes.zip"),
            403,
            "application/xml",
            b"",
        );
        let counted_client = sec_client.clone();

        let expected_result = BTreeMap::from([(format!("{NOTES_URL}2026_09_notes.zip"), 1)]);

        let _ = Extraction::backfill()
            .all()
            .periods(["2026_08", "2026_09"])
            .until(date(2026, 10, 1))
            .sec_client(sec_client)
            .repositories(repositories(&root))
            .retry_delays(Vec::new())
            .run(|_, _| {})
            .await;
        let result = counted_client.inner.request_counts.per_url();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_every_date_from_the_cut_over_to_until_when_the_cut_over_is_earlier() {
        let cut_over = date(2026, 9, 30);
        let until = date(2026, 10, 3);

        let expected_result = vec![
            date(2026, 9, 30),
            date(2026, 10, 1),
            date(2026, 10, 2),
            date(2026, 10, 3),
        ];

        let result = dates_between(cut_over, until);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_one_date_when_the_cut_over_is_until() {
        let cut_over = date(2026, 10, 31);
        let until = date(2026, 10, 31);

        let expected_result = vec![date(2026, 10, 31)];

        let result = dates_between(cut_over, until);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_no_date_when_the_cut_over_is_after_until() {
        let cut_over = date(2026, 11, 30);
        let until = date(2026, 11, 1);

        let expected_result: Vec<NaiveDate> = Vec::new();

        let result = dates_between(cut_over, until);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_start_the_catch_up_seven_days_before_the_cut_over_date_when_the_date_is_in_range() {
        let cut_over = date(2026, 9, 30);

        let expected_result = date(2026, 9, 23);

        let result = catch_up_start(cut_over);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_start_the_catch_up_on_the_cut_over_date_when_the_lookback_leaves_the_date_range() {
        let cut_over = NaiveDate::MIN;

        let expected_result = NaiveDate::MIN;

        let result = catch_up_start(cut_over);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_use_the_notes_default_cap_when_in_flight_is_not_set() {
        let periods = ["2026_09"];

        let expected_result = 2;

        let result = Extraction::backfill().all().periods(periods).in_flight;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_the_notes_period_when_the_body_stops_early_and_all_gets_no_retry_delays() {
        let root = temporary_root();
        let sec_client = RoutingSecClient::new().route_stopped_body_once(
            &format!("{NOTES_URL}2026_09_notes.zip"),
            "application/zip",
            SAMPLE_NOTES_ZIP,
        );
        let retry_delays: Vec<Duration> = Vec::new();

        let expected_result = Ok(1);

        let result = Extraction::backfill()
            .all()
            .periods(["2026_09", "2026_10"])
            .until(date(2026, 9, 30))
            .sec_client(sec_client)
            .repositories(repositories(&root))
            .retry_delays(retry_delays)
            .run(|_, _| {})
            .await
            .map(|summary| summary.notes().failure_count());

        assert_eq!(result, expected_result);
    }
}
