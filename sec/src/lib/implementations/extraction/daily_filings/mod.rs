//! # Daily Filings
//!
//! Provides [`IncrementalDailyFilings`], the new-filing path of an incremental run over many
//! dates, and the [`DailyRunSummary`] of that run.
//!
//! [`IncrementalDailyFilings::dates`] returns a [`DailyFilingsBuilder`], whose [`run`] runs the
//! dates in order, one date at a time. For each date, the run detects the filings, then extracts
//! each filing, then the `submissions` file of each company. Before it sends a request for a
//! filing, the run reads the store. It skips a filing whose stored XBRL instance passes the size
//! and SHA-256 check, and a filing whose stored folder index lists no XBRL instance.
//!
//! [`run`]: DailyFilingsBuilder::run

use std::time::Duration;

use chrono::NaiveDate;
use futures_util::StreamExt;
use storage::implementations::repository::filesystem::{
    Document, FilesystemRepository, InputDocument, Read, Write,
};
use storage::traits::repository::ReadRepository;

use super::Incremental;
use super::retry::{DEFAULT_RETRY_DELAYS, with_retry};
use super::run_event::{ErrorKind, RunEvent, Unit};
use crate::implementations::states::extract::daily_index::builder::NoSecClient;
use crate::implementations::states::extract::daily_index::{
    DetectFilingsSuperState, DetectedFilings,
};
use crate::implementations::states::extract::filing::ExtractFilingSuperState;
use crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
use crate::implementations::states::extract::submissions::ExtractSubmissionsSuperState;
use crate::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use crate::implementations::states::extract::validate_index_date::ValidateIndexDate;
use crate::prelude::{IntoStateMachineStream, StateMachineStream, StreamError, StreamItem};
use crate::shared::cik::Cik;
use crate::shared::filing_documents::FolderIndex;
use crate::shared::filing_documents_ref::{filing_index_key, instance_key, labels_key};
use crate::shared::filing_ref::FilingRef;
use crate::shared::http_client::Executes;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::{
    RawDailyIndex, RawFilingIndex, RawSubmissions, RawXbrlInstance, RawXbrlLabels,
};
use crate::shared::request::implementations::sec_request::{FilingDocumentSet, FilingIndex};
use crate::shared::response::implementations::sec_response::{JsonBody, SecResponse, ZipBody};
use summary::{
    CompanyOutcome, DateOutcome, Fetch, FilingOutcome, is_missing_instance, is_pending,
    is_unextractable,
};

mod summary;

pub use summary::{DailyRunSummary, FailedDailyRun};

/// The number of filing machines, and of `submissions` machines, that one date runs at once
/// without a call to [`DailyFilingsBuilder::in_flight`].
///
/// The rate limiter of the SEC client caps the requests, so a higher value does not make a run
/// faster.
pub const DEFAULT_MACHINES_IN_FLIGHT: usize = 4;

/// The number of days, from [`as_of`](DailyFilingsBuilder::as_of) back, in the window of the
/// pending rule. A pending date does not fail the run, because a later run reads it again.
pub const PENDING_DAYS: u64 = 3;

/// The SEC client that every machine of a run accepts.
pub trait DailyClient:
    SecClientTrait<
        Response<ZipBody> = SecResponse<ZipBody>,
        Response<JsonBody> = SecResponse<JsonBody>,
        Error = FailedSecRequest,
    > + Executes<FilingIndex>
    + Executes<FilingDocumentSet>
    + 'static
{
}

impl<C> DailyClient for C where
    C: SecClientTrait<
            Response<ZipBody> = SecResponse<ZipBody>,
            Response<JsonBody> = SecResponse<JsonBody>,
            Error = FailedSecRequest,
        > + Executes<FilingIndex>
        + Executes<FilingDocumentSet>
        + 'static
{
}

type DailyIndexRepository = FilesystemRepository<Write, RawDocument<RawDailyIndex>>;
type FilingIndexRepository = FilesystemRepository<Write, RawDocument<RawFilingIndex>>;
type InstanceRepository = FilesystemRepository<Write, RawDocument<RawXbrlInstance>>;
type InstanceReader = FilesystemRepository<Read, RawDocument<RawXbrlInstance>>;
type FilingIndexReader = FilesystemRepository<Read, RawDocument<RawFilingIndex>>;
type LabelsRepository = FilesystemRepository<Write, RawDocument<RawXbrlLabels>>;
type LabelsReader = FilesystemRepository<Read, RawDocument<RawXbrlLabels>>;
type SubmissionsRepository = FilesystemRepository<Write, RawDocument<RawSubmissions>>;

/// The stores that one run writes to.
#[derive(Debug, Clone)]
pub struct DailyFilingsRepositories {
    /// The store of the daily indexes.
    pub daily_index: DailyIndexRepository,
    /// The store of the folder indexes of the filings.
    pub filing_index: FilingIndexRepository,
    /// The store of the XBRL instances.
    pub instance: InstanceRepository,
    /// The store of the label files.
    pub labels: LabelsRepository,
    /// The store of the `submissions` files.
    pub submissions: SubmissionsRepository,
}

/// Marks a [`DailyFilingsBuilder`] that holds no `as_of` date yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoAsOf;

/// Marks a [`DailyFilingsBuilder`] that holds no stores yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoRepositories;

/// The new-filing path of an incremental run, before the choice of the dates.
///
/// Every super-state of the run carries the `execution_id` of its [`Incremental`] handle and
/// [`IngestionMode::Incremental`](storage::implementations::repository::filesystem::IngestionMode::Incremental).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IncrementalDailyFilings {
    incremental: Incremental,
}

impl IncrementalDailyFilings {
    /// Creates a new [`IncrementalDailyFilings`] for the run of `incremental`.
    pub(super) const fn new(incremental: Incremental) -> Self {
        Self { incremental }
    }

    /// Returns the builder of a run over `dates`. The run reads them oldest first, once each.
    ///
    /// The pending rule never applies to the oldest date. The builder still needs the `as_of`
    /// date, an SEC client, and the stores.
    #[must_use]
    pub fn dates(
        self,
        dates: impl IntoIterator<Item = NaiveDate>,
    ) -> DailyFilingsBuilder<NoAsOf, NoSecClient, NoRepositories> {
        let mut dates: Vec<NaiveDate> = dates.into_iter().collect();
        dates.sort_unstable();
        dates.dedup();
        DailyFilingsBuilder {
            incremental: self.incremental,
            dates,
            as_of: NoAsOf,
            sec_client: NoSecClient,
            repositories: NoRepositories,
            in_flight: DEFAULT_MACHINES_IN_FLIGHT,
            retry_delays: DEFAULT_RETRY_DELAYS.to_vec(),
        }
    }
}

/// Builds and runs the new-filing path over many dates.
///
/// Set [`as_of`](Self::as_of), [`sec_client`](Self::sec_client), and
/// [`repositories`](Self::repositories), in any order. [`in_flight`](Self::in_flight) and
/// [`retry_delays`](Self::retry_delays) are optional. Then call [`run`](Self::run).
///
/// `run` needs the stores:
///
/// ```compile_fail,E0599
/// use chrono::NaiveDate;
/// use sec::implementations::extraction::Extraction;
/// use sec::shared::http_client::implementations::sec_client::SecClient;
///
/// let as_of = NaiveDate::MIN;
/// let _run = Extraction::incremental()
///     .daily_filings()
///     .dates([as_of])
///     .as_of(as_of)
///     .sec_client(SecClient::default())
///     .run(|_, _| {});
/// ```
///
/// `run` needs the SEC client:
///
/// ```compile_fail,E0599
/// use chrono::NaiveDate;
/// use sec::implementations::extraction::{DailyFilingsRepositories, Extraction};
///
/// fn run_without_sec_client(repositories: DailyFilingsRepositories) {
///     let _run = Extraction::incremental()
///         .daily_filings()
///         .dates([NaiveDate::MIN])
///         .as_of(NaiveDate::MIN)
///         .repositories(repositories)
///         .run(|_, _| {});
/// }
/// ```
///
/// And `run` needs the `as_of` date:
///
/// ```compile_fail,E0599
/// use chrono::NaiveDate;
/// use sec::implementations::extraction::{DailyFilingsRepositories, Extraction};
/// use sec::shared::http_client::implementations::sec_client::SecClient;
///
/// fn run_without_as_of(repositories: DailyFilingsRepositories) {
///     let _run = Extraction::incremental()
///         .daily_filings()
///         .dates([NaiveDate::MIN])
///         .sec_client(SecClient::default())
///         .repositories(repositories)
///         .run(|_, _| {});
/// }
/// ```
#[derive(Debug, Clone)]
pub struct DailyFilingsBuilder<A, C, R> {
    incremental: Incremental,
    dates: Vec<NaiveDate>,
    as_of: A,
    sec_client: C,
    repositories: R,
    in_flight: usize,
    retry_delays: Vec<Duration>,
}

impl<C, R> DailyFilingsBuilder<NoAsOf, C, R> {
    /// Sets `as_of`, the date that the pending rule counts back from.
    ///
    /// A missing daily index counts as pending if its date is not the first date of the run, and
    /// is on or after the date [`PENDING_DAYS`] - 1 days before `as_of`. A date after `as_of` also
    /// counts. A pending date does not fail the run. The binaries pass the day before today in New
    /// York.
    #[must_use]
    pub fn as_of(self, as_of: NaiveDate) -> DailyFilingsBuilder<NaiveDate, C, R> {
        DailyFilingsBuilder {
            incremental: self.incremental,
            dates: self.dates,
            as_of,
            sec_client: self.sec_client,
            repositories: self.repositories,
            in_flight: self.in_flight,
            retry_delays: self.retry_delays,
        }
    }
}

impl<A, R> DailyFilingsBuilder<A, NoSecClient, R> {
    /// Sets the SEC client of every machine of the run.
    #[must_use]
    pub fn sec_client<C: DailyClient>(self, sec_client: C) -> DailyFilingsBuilder<A, C, R> {
        DailyFilingsBuilder {
            incremental: self.incremental,
            dates: self.dates,
            as_of: self.as_of,
            sec_client,
            repositories: self.repositories,
            in_flight: self.in_flight,
            retry_delays: self.retry_delays,
        }
    }
}

impl<A, C> DailyFilingsBuilder<A, C, NoRepositories> {
    /// Sets the stores of the daily indexes, the folder indexes and the documents of the filings,
    /// and the `submissions` files.
    #[must_use]
    pub fn repositories(
        self,
        repositories: DailyFilingsRepositories,
    ) -> DailyFilingsBuilder<A, C, DailyFilingsRepositories> {
        DailyFilingsBuilder {
            incremental: self.incremental,
            dates: self.dates,
            as_of: self.as_of,
            sec_client: self.sec_client,
            repositories,
            in_flight: self.in_flight,
            retry_delays: self.retry_delays,
        }
    }
}

impl<A, C, R> DailyFilingsBuilder<A, C, R> {
    /// Sets the most filing machines, and the most `submissions` machines, that one date runs at
    /// once. The default is [`DEFAULT_MACHINES_IN_FLIGHT`], and `0` counts as `1`.
    #[must_use]
    pub fn in_flight(self, in_flight: usize) -> Self {
        Self {
            in_flight: in_flight.max(1),
            ..self
        }
    }

    /// Sets the waits before each new attempt of a machine that failed with a transient SEC
    /// failure. The default is [`DEFAULT_RETRY_DELAYS`].
    #[must_use]
    pub fn retry_delays(self, retry_delays: impl IntoIterator<Item = Duration>) -> Self {
        Self {
            retry_delays: retry_delays.into_iter().collect(),
            ..self
        }
    }
}

impl<C> DailyFilingsBuilder<NaiveDate, C, DailyFilingsRepositories>
where
    C: DailyClient,
    DetectFilingsSuperState<ValidateIndexDate, DailyIndexRepository, C>: IntoStateMachineStream,
    ExtractFilingSuperState<
        PrepareSecRequest<FilingIndex>,
        FilingIndexRepository,
        InstanceRepository,
        LabelsRepository,
        C,
    >: IntoStateMachineStream,
    ExtractSubmissionsSuperState<ValidateCikFormat, SubmissionsRepository, C>:
        IntoStateMachineStream,
{
    /// Runs the new-filing path for each date, in order, and returns the summary.
    ///
    /// The dates run one after another, never at once. For each date, the run runs detection,
    /// then one filing machine per selected filing that the store cannot skip, then one
    /// `submissions` machine per distinct CIK of the selected filings. At most
    /// [`in_flight`](Self::in_flight) filing machines run at once, and the same for `submissions`
    /// machines. The detection of a date, a filing machine, and a `submissions` machine that fail
    /// with a transient SEC failure run again after each retry delay. A failure does not stop the
    /// other machines or the other dates.
    ///
    /// Calls `observer` with the unit and each successful stream item, and with the unit, each
    /// error, and its [`ErrorKind`]. The unit is the date, the filing, or the company. A skipped
    /// filing sends no event to the observer.
    pub async fn run(
        self,
        observer: impl Fn(&Unit, RunEvent<'_>) + Send + Sync + 'static,
    ) -> DailyRunSummary {
        let Some(&first_date) = self.dates.first() else {
            return DailyRunSummary(Vec::new());
        };
        let mut outcomes = Vec::with_capacity(self.dates.len());
        for &date in &self.dates {
            let unit = Unit::Date(date);
            let detection = with_retry(
                &self.retry_delays,
                || {
                    self.incremental
                        .daily_index()
                        .date(date.to_string())
                        .sec_client(self.sec_client.clone())
                        .repository(self.repositories.daily_index.clone())
                        .build()
                        .run_to_detected_filings(|item| observer(&unit, RunEvent::Item(item)))
                },
                |error| observer(&unit, RunEvent::Error(error, ErrorKind::Retried)),
            )
            .await;
            let outcome = match detection {
                Ok(detected) => DateOutcome::Processed {
                    filings: self.extract_filings(date, &detected, &observer).await,
                    companies: self
                        .extract_companies(date, &detected.filings, &observer)
                        .await,
                },
                Err(error) if is_pending(&error, date, first_date, self.as_of) => {
                    observer(&unit, RunEvent::Error(&error, ErrorKind::Pending));
                    DateOutcome::Pending(error)
                }
                Err(error) => {
                    observer(&unit, RunEvent::Error(&error, ErrorKind::Failed));
                    DateOutcome::Failed(error)
                }
            };
            outcomes.push((date, outcome));
        }
        DailyRunSummary(outcomes)
    }

    /// Runs one filing machine per detected filing, at most [`in_flight`](Self::in_flight) at
    /// once, in the order of the filings. Each machine records the daily index of `detected` as the input of
    /// its files.
    ///
    /// A filing runs no machine if its stored instance passes the size and SHA-256 check and its
    /// label file passes it or is not listed, or if its stored folder index lists no XBRL instance
    /// (see [`StoreReaders::skip`]).
    async fn extract_filings(
        &self,
        date: NaiveDate,
        detected: &DetectedFilings,
        observer: &(impl Fn(&Unit, RunEvent<'_>) + Sync),
    ) -> Vec<FilingOutcome> {
        let inputs: Vec<InputDocument> = detected.index.iter().cloned().collect();
        let inputs = &inputs;
        let readers = &StoreReaders {
            instance: FilesystemRepository::builder()
                .reader()
                .document_type::<RawDocument<RawXbrlInstance>>()
                .root(self.repositories.instance.root())
                .build(),
            labels: FilesystemRepository::builder()
                .reader()
                .document_type::<RawDocument<RawXbrlLabels>>()
                .root(self.repositories.labels.root())
                .build(),
            filing_index: FilesystemRepository::builder()
                .reader()
                .document_type::<RawDocument<RawFilingIndex>>()
                .root(self.repositories.filing_index.root())
                .build(),
        };
        let extractions = detected.filings.iter().map(|filing| async move {
            if let Some(skip) = readers.skip(filing).await {
                return FilingOutcome {
                    accession_number: filing.accession_number().clone(),
                    result: Ok(skip),
                };
            }
            let unit = Unit::Filing {
                date,
                accession_number: filing.accession_number().clone(),
            };
            let result = with_retry(
                &self.retry_delays,
                || {
                    self.incremental
                        .filing()
                        .filing_ref(filing.clone())
                        .sec_client(self.sec_client.clone())
                        .filing_index_repository(self.repositories.filing_index.clone())
                        .instance_repository(self.repositories.instance.clone())
                        .labels_repository(self.repositories.labels.clone())
                        .inputs(inputs.clone())
                        .build()
                        .run_to_documents(|item| observer(&unit, RunEvent::Item(item)))
                },
                |error| observer(&unit, RunEvent::Error(error, ErrorKind::Retried)),
            )
            .await
            .map(|_| Fetch::Downloaded)
            .inspect_err(|error| {
                let kind = if is_unextractable(error) {
                    ErrorKind::Unextractable
                } else {
                    ErrorKind::Failed
                };
                observer(&unit, RunEvent::Error(error, kind));
            });
            FilingOutcome {
                accession_number: filing.accession_number().clone(),
                result,
            }
        });
        futures_util::stream::iter(extractions)
            .buffered(self.in_flight)
            .collect()
            .await
    }

    /// Runs one `submissions` machine per distinct CIK of `filings`, at most
    /// [`in_flight`](Self::in_flight) at once, in the order in which each CIK first appears.
    async fn extract_companies(
        &self,
        date: NaiveDate,
        filings: &[FilingRef],
        observer: &(impl Fn(&Unit, RunEvent<'_>) + Sync),
    ) -> Vec<CompanyOutcome> {
        let mut ciks: Vec<&Cik> = Vec::new();
        for filing in filings {
            if !ciks.contains(&filing.cik()) {
                ciks.push(filing.cik());
            }
        }
        let extractions = ciks.into_iter().map(|cik| async move {
            let unit = Unit::Company {
                date,
                cik: cik.clone(),
            };
            let result = with_retry(
                &self.retry_delays,
                || {
                    let stream = self
                        .incremental
                        .submissions()
                        .cik(cik.to_string())
                        .sec_client(self.sec_client.clone())
                        .repository(self.repositories.submissions.clone())
                        .build()
                        .into_run_stream();
                    run_to_end(stream, |item| observer(&unit, RunEvent::Item(item)))
                },
                |error| observer(&unit, RunEvent::Error(error, ErrorKind::Retried)),
            )
            .await
            .inspect_err(|error| observer(&unit, RunEvent::Error(error, ErrorKind::Failed)));
            CompanyOutcome {
                cik: cik.clone(),
                result,
            }
        });
        futures_util::stream::iter(extractions)
            .buffered(self.in_flight)
            .collect()
            .await
    }
}

/// The readers of the stored files that let the run skip a filing before any request.
struct StoreReaders {
    instance: InstanceReader,
    labels: LabelsReader,
    filing_index: FilingIndexReader,
}

impl StoreReaders {
    /// Returns how the run skips `filing`, or `None` if the run must fetch it.
    ///
    /// The run skips a filing whose stored instance passes the size and SHA-256 check, if its
    /// stored label file passes the same check or its stored folder index lists no label file.
    /// It also skips a filing whose stored folder index lists no XBRL instance. Both folder rules
    /// are those of [`FolderIndex::select_documents`]. A read error counts as not stored, so the
    /// run fetches the filing again.
    async fn skip(&self, filing: &FilingRef) -> Option<Fetch> {
        if matches!(self.instance.get(instance_key(filing)).await, Ok(Some(_))) {
            return self
                .has_intact_labels(filing)
                .await
                .then_some(Fetch::AlreadyStored);
        }
        let folder_index = self.folder_index(filing).await?;
        lists_no_instance(folder_index.bytes(), filing).then_some(Fetch::SkippedUnextractable)
    }

    /// Returns `true` if the stored label file of `filing` passes the size and SHA-256 check, or
    /// if its stored folder index lists no label file.
    async fn has_intact_labels(&self, filing: &FilingRef) -> bool {
        if matches!(self.labels.get(labels_key(filing)).await, Ok(Some(_))) {
            return true;
        }
        self.folder_index(filing)
            .await
            .is_some_and(|folder_index| lists_no_label_file(folder_index.bytes(), filing))
    }

    /// Returns the stored folder index of `filing`, or `None` if the store holds none or the
    /// read fails.
    async fn folder_index(&self, filing: &FilingRef) -> Option<RawDocument<RawFilingIndex>> {
        self.filing_index
            .get(filing_index_key(filing))
            .await
            .ok()
            .flatten()
    }
}

/// Returns `true` if `folder_index` parses, and the selection of `filing` succeeds with no label
/// file.
fn lists_no_label_file(folder_index: &[u8], filing: &FilingRef) -> bool {
    FolderIndex::from_json(folder_index).is_ok_and(|index| {
        index
            .select_documents(filing.accession_number())
            .is_ok_and(|documents| documents.label_file().is_none())
    })
}

/// Returns `true` if `folder_index` parses and the selection of `filing` fails because the folder
/// holds no XBRL instance.
///
/// A folder index that lists an instance means an earlier run failed after the folder request,
/// so the run fetches the filing. A folder index that does not parse also returns `false`.
fn lists_no_instance(folder_index: &[u8], filing: &FilingRef) -> bool {
    FolderIndex::from_json(folder_index).is_ok_and(|index| {
        index
            .select_documents(filing.accession_number())
            .is_err_and(|error| is_missing_instance(&error))
    })
}

/// Runs `stream` to its end, and returns the first error.
async fn run_to_end(
    mut stream: StateMachineStream,
    on_item: impl Fn(&StreamItem),
) -> Result<(), StreamError> {
    while let Some(item) = stream.next().await {
        on_item(&item?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::implementations::extraction::Extraction;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};

    fn sample_filing() -> FilingRef {
        FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenQ, false),
            NaiveDate::from_ymd_opt(2026, 9, 8)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    #[test]
    fn should_list_no_instance_when_the_stored_folder_index_holds_only_a_label_file() {
        let folder_index = br#"{"directory":{"item":[{"name":"alco-20260908_lab.xml"}]}}"#;

        let expected_result = true;

        let result = lists_no_instance(folder_index, &sample_filing());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_list_no_instance_when_the_stored_folder_index_is_not_json() {
        let folder_index = b"<html>";

        let expected_result = false;

        let result = lists_no_instance(folder_index, &sample_filing());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_list_no_instance_when_the_stored_folder_index_holds_two_instances() {
        let folder_index = br#"{"directory":{"item":[{"name":"a-20260908_htm.xml"},{"name":"b-20260908_htm.xml"}]}}"#;

        let expected_result = false;

        let result = lists_no_instance(folder_index, &sample_filing());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_list_no_instance_when_the_stored_folder_index_holds_an_instance() {
        let folder_index = br#"{"directory":{"item":[{"name":"alco-20260908_htm.xml"}]}}"#;

        let expected_result = false;

        let result = lists_no_instance(folder_index, &sample_filing());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_zero_as_one_when_setting_in_flight() {
        let in_flight = 0;

        let expected_result = 1;

        let result = Extraction::incremental()
            .daily_filings()
            .dates([NaiveDate::MIN])
            .in_flight(in_flight)
            .in_flight;

        assert_eq!(result, expected_result);
    }
}
