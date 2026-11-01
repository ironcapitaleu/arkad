//! # Run
//!
//! Provides [`run`], which makes the two facade calls of one `backfill` run, and
//! [`dates_between`], which lists the dates of its catch-up.
//!
//! The first call stores the Notes periods. The second call, the catch-up, reads each daily index
//! from the cut-over date to yesterday. See "Cut-Over Between the Paths" in
//! `sec/design/pipeline/ingestion_paths.md`.

use std::sync::Arc;

use chrono::NaiveDate;
use sec::implementations::extraction::{
    Backfill, DailyClient, DailyFilingsRepositories, DailyRunSummary, Extraction,
    MissingCutOverDate, NotesRunSummary, RunEvent, Unit,
};
use sec::implementations::states::extract::daily_index::DetectFilingsSuperState;
use sec::implementations::states::extract::filing::ExtractFilingSuperState;
use sec::implementations::states::extract::notes::ExtractNotesSuperState;
use sec::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
use sec::implementations::states::extract::submissions::ExtractSubmissionsSuperState;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::implementations::states::extract::validate_index_date::ValidateIndexDate;
use sec::implementations::states::extract::validate_notes_period::ValidateNotesPeriod;
use sec::prelude::IntoStateMachineStream;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::{
    RawDailyIndex, RawNotesArchive, RawSubmissions, RawXbrlInstance, RawXbrlLabels,
};
use sec::shared::request::implementations::sec_request::FilingIndex;
use storage::implementations::repository::filesystem::{FilesystemRepository, Write};

/// The store of the Notes files.
pub type NotesWriter = FilesystemRepository<Write, RawDocument<RawNotesArchive>>;

/// The inputs of one `backfill` run, apart from the observer.
pub struct BackfillInputs<C> {
    /// The handle whose `execution_id` both calls carry.
    pub backfill: Backfill,
    /// The Notes periods of the first call, in order.
    pub periods: Vec<String>,
    /// The most Notes files that the first call downloads at once.
    pub files_in_flight: usize,
    /// The last date of the catch-up: yesterday in New York.
    pub yesterday: NaiveDate,
    /// The SEC client of both calls.
    pub sec_client: C,
    /// The store of the Notes files.
    pub notes_repository: NotesWriter,
    /// The stores of the catch-up.
    pub daily_repositories: DailyFilingsRepositories,
}

/// The summaries of both calls of one `backfill` run.
pub struct BackfillSummaries {
    /// The summary of the Notes periods.
    pub past: NotesRunSummary,
    /// The dates of the catch-up, from the cut-over date to yesterday.
    pub dates: Vec<NaiveDate>,
    /// The summary of the catch-up.
    pub recent: DailyRunSummary,
}

/// Stores the Notes periods of `inputs`, then reads each date from the cut-over date to
/// yesterday, and returns both summaries.
///
/// Calls `on_past` with the Notes summary before the catch-up starts. Passes every event of both
/// calls to `observer`. The catch-up runs even if it has no date, and then its summary is empty.
///
/// # Errors
///
/// Returns [`MissingCutOverDate`] if a Notes period failed or no period is stored. The catch-up
/// then does not run.
pub async fn run<C>(
    inputs: BackfillInputs<C>,
    observer: impl Fn(&Unit, RunEvent<'_>) + Send + Sync + 'static,
    on_past: impl FnOnce(&NotesRunSummary),
) -> Result<BackfillSummaries, MissingCutOverDate>
where
    C: DailyClient,
    ExtractNotesSuperState<ValidateNotesPeriod, NotesWriter, C>: IntoStateMachineStream,
    DetectFilingsSuperState<
        ValidateIndexDate,
        FilesystemRepository<Write, RawDocument<RawDailyIndex>>,
        C,
    >: IntoStateMachineStream,
    ExtractFilingSuperState<
        PrepareSecRequest<FilingIndex>,
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
    let observer = Arc::new(observer);
    let past_observer = Arc::clone(&observer);
    let past = inputs
        .backfill
        .notes()
        .periods(inputs.periods)
        .sec_client(inputs.sec_client.clone())
        .repository(inputs.notes_repository)
        .in_flight(inputs.files_in_flight)
        .run(move |unit, event| past_observer(unit, event))
        .await;
    on_past(&past);
    let dates = dates_between(past.cut_over_date()?, inputs.yesterday);
    let recent = Extraction::incremental()
        .execution_id(inputs.backfill.id())
        .daily_filings()
        .dates(dates.iter().copied())
        .yesterday(inputs.yesterday)
        .sec_client(inputs.sec_client)
        .repositories(inputs.daily_repositories)
        .run(move |unit, event| observer(unit, event))
        .await;
    Ok(BackfillSummaries {
        past,
        dates,
        recent,
    })
}

/// Returns every date from `cut_over` to `yesterday`, both included, oldest first.
///
/// If `cut_over` comes after `yesterday`, the list is empty.
pub fn dates_between(cut_over: NaiveDate, yesterday: NaiveDate) -> Vec<NaiveDate> {
    cut_over
        .iter_days()
        .take_while(|date| *date <= yesterday)
        .collect()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .expect("Given a hardcoded valid date, parsing should always succeed")
    }

    #[test]
    fn should_list_every_date_from_the_cut_over_to_yesterday_when_the_cut_over_is_earlier() {
        let cut_over = date("2026-09-30");
        let yesterday = date("2026-10-03");

        let expected_result = vec![
            date("2026-09-30"),
            date("2026-10-01"),
            date("2026-10-02"),
            date("2026-10-03"),
        ];

        let result = dates_between(cut_over, yesterday);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_one_date_when_the_cut_over_is_yesterday() {
        let cut_over = date("2026-10-31");
        let yesterday = date("2026-10-31");

        let expected_result = vec![date("2026-10-31")];

        let result = dates_between(cut_over, yesterday);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_no_date_when_the_cut_over_is_after_yesterday() {
        let cut_over = date("2026-11-30");
        let yesterday = date("2026-11-01");

        let expected_result: Vec<NaiveDate> = Vec::new();

        let result = dates_between(cut_over, yesterday);

        assert_eq!(result, expected_result);
    }
}
