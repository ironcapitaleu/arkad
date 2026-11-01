//! # Submissions
//!
//! Provides [`BackfillSubmissions`], the `submissions` source of a backfill run, and the batch run
//! over many CIKs with its [`SubmissionsRunSummary`].
//!
//! [`BackfillSubmissions::cik`] returns the builder of one [`ExtractSubmissionsSuperState`].
//! [`BackfillSubmissions::ciks`] returns a [`SubmissionsBatchBuilder`], whose [`run`] extracts the
//! `submissions` JSON of each CIK into the store.
//!
//! [`run`]: SubmissionsBatchBuilder::run

use std::sync::Arc;

use futures_util::StreamExt;
use storage::implementations::repository::filesystem::IngestionMode;
use uuid::Uuid;

use super::DEFAULT_CIKS_IN_FLIGHT;
use super::run_event::{RunEvent, Unit, drain};
use crate::implementations::states::extract::submissions::ExtractSubmissionsSuperState;
use crate::implementations::states::extract::submissions::builder::{
    ExtractSubmissionsSuperStateBuilder, NoRepository, NoSecClient,
};
use crate::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawSubmissions;
use crate::shared::response::implementations::sec_response::{JsonBody, SecResponse};
use crate::traits::state_machine::stream::IntoStateMachineStream;

mod summary;

pub use summary::{FailedSubmissionsBackfill, SubmissionsRunSummary};

/// The `submissions` source of a backfill run, before the choice of one CIK or many.
///
/// Every super-state built from it carries the `execution_id` of its
/// [`Backfill`](super::Backfill) handle and [`IngestionMode::Backfill`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BackfillSubmissions {
    execution_id: Uuid,
}

impl BackfillSubmissions {
    /// Creates a new [`BackfillSubmissions`] for the run with `execution_id`.
    pub(super) const fn new(execution_id: Uuid) -> Self {
        Self { execution_id }
    }

    /// Returns the builder of an [`ExtractSubmissionsSuperState`] for one raw CIK, for example
    /// `320193`.
    ///
    /// The builder still needs an SEC client and a repository of type `R`.
    #[must_use]
    pub fn cik<R: RawRepository<RawSubmissions>>(
        self,
        cik: impl Into<String>,
    ) -> ExtractSubmissionsSuperStateBuilder<
        String,
        NoSecClient,
        Uuid,
        IngestionMode,
        NoRepository<R>,
    > {
        ExtractSubmissionsSuperState::builder()
            .execution_id(self.execution_id)
            .ingestion_mode(IngestionMode::Backfill)
            .cik(cik)
    }

    /// Returns the builder of a batch run over `ciks`. The summary lists the CIKs in the order of
    /// the list.
    ///
    /// The builder still needs an SEC client and a repository of type `R`.
    #[must_use]
    pub fn ciks<R: RawRepository<RawSubmissions>>(
        self,
        ciks: impl IntoIterator<Item = impl Into<String>>,
    ) -> SubmissionsBatchBuilder<NoSecClient, NoRepository<R>> {
        SubmissionsBatchBuilder {
            submissions: self,
            ciks: ciks.into_iter().map(Into::into).collect(),
            sec_client: NoSecClient,
            repository: NoRepository::new(),
            in_flight: DEFAULT_CIKS_IN_FLIGHT,
        }
    }
}

/// Builds and runs a batch of CIKs, one [`ExtractSubmissionsSuperState`] per CIK.
///
/// Set the client with [`sec_client`](Self::sec_client) and the store with
/// [`repository`](Self::repository), in any order. [`in_flight`](Self::in_flight) is optional.
/// Then call [`run`](Self::run).
///
/// Only a batch has `run`, and only one CIK has `build`. So `build` does not follow `ciks`:
///
/// ```compile_fail,E0599
/// use sec::implementations::extraction::Extraction;
/// use sec::shared::raw_document::RawDocument;
/// use sec::shared::raw_source::RawSubmissions;
/// use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
///
/// let _state = Extraction::backfill()
///     .submissions()
///     .ciks::<FilesystemRepository<Write, RawDocument<RawSubmissions>>>(["320193"])
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct SubmissionsBatchBuilder<C, R> {
    submissions: BackfillSubmissions,
    ciks: Vec<String>,
    sec_client: C,
    repository: R,
    in_flight: usize,
}

impl<R> SubmissionsBatchBuilder<NoSecClient, R> {
    /// Sets the SEC client that downloads the `submissions` files.
    #[must_use]
    pub fn sec_client<C>(self, sec_client: C) -> SubmissionsBatchBuilder<C, R>
    where
        C: SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>, Error = FailedSecRequest>,
    {
        SubmissionsBatchBuilder {
            submissions: self.submissions,
            ciks: self.ciks,
            sec_client,
            repository: self.repository,
            in_flight: self.in_flight,
        }
    }
}

impl<C, R: RawRepository<RawSubmissions>> SubmissionsBatchBuilder<C, NoRepository<R>> {
    /// Sets the repository that stores the `submissions` files.
    #[must_use]
    pub fn repository(self, repository: R) -> SubmissionsBatchBuilder<C, R> {
        SubmissionsBatchBuilder {
            submissions: self.submissions,
            ciks: self.ciks,
            sec_client: self.sec_client,
            repository,
            in_flight: self.in_flight,
        }
    }
}

impl<C, R> SubmissionsBatchBuilder<C, R> {
    /// Sets the most CIKs that run at once. The default is [`DEFAULT_CIKS_IN_FLIGHT`], and `0`
    /// counts as `1`.
    #[must_use]
    pub fn in_flight(self, in_flight: usize) -> Self {
        Self {
            in_flight: in_flight.max(1),
            ..self
        }
    }
}

impl<C, R> SubmissionsBatchBuilder<C, R>
where
    C: SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>, Error = FailedSecRequest>,
    R: RawRepository<RawSubmissions>,
    ExtractSubmissionsSuperState<ValidateCikFormat, R, C>: IntoStateMachineStream,
{
    /// Extracts the `submissions` file of each CIK into the repository and returns the summary.
    ///
    /// At most [`in_flight`](Self::in_flight) CIKs run at once. A failed CIK does not stop the
    /// others.
    ///
    /// Calls `observer` with the CIK and each successful stream item, and with the CIK and the
    /// error of a failed CIK. Every error gives
    /// [`ErrorKind::Failed`](super::ErrorKind::Failed).
    pub async fn run(
        self,
        observer: impl Fn(&Unit, RunEvent<'_>) + Send + Sync + 'static,
    ) -> SubmissionsRunSummary {
        let observer = Arc::new(observer);
        let extractions = self.ciks.into_iter().enumerate().map(|(index, cik)| {
            let super_state = self
                .submissions
                .cik(cik.as_str())
                .sec_client(self.sec_client.clone())
                .repository(self.repository.clone())
                .build();
            let observer = Arc::clone(&observer);
            async move {
                let unit = Unit::Cik(cik);
                let outcome = drain(super_state.into_run_stream(), &unit, observer.as_ref()).await;
                (index, unit, outcome)
            }
        });
        let mut outcomes: Vec<_> = futures_util::stream::iter(extractions)
            .buffer_unordered(self.in_flight)
            .collect()
            .await;
        outcomes.sort_by_key(|(index, _, _)| *index);
        SubmissionsRunSummary(
            outcomes
                .into_iter()
                .map(|(_, unit, outcome)| (unit.to_string(), outcome))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;
    use std::sync::atomic::Ordering;

    use pretty_assertions::assert_eq;
    use storage::implementations::repository::filesystem::Document;

    use super::*;
    use crate::implementations::extraction::Extraction;
    use crate::tests::fixtures::data::submissions::SAMPLE_SUBMISSIONS_JSON;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_rate_limiter::HoldingRateLimiter;
    use crate::tests::fixtures::sample_write_repository::FakeSubmissionsRepository;

    fn sec_client() -> AlwaysRespondingSecClient {
        AlwaysRespondingSecClient::new(
            200,
            vec![("content-type", "application/json".to_string())],
            SAMPLE_SUBMISSIONS_JSON.to_vec(),
        )
    }

    /// Runs a batch over `ciks` into `repository` without an observer.
    async fn run_batch(
        repository: FakeSubmissionsRepository,
        ciks: &[&str],
    ) -> SubmissionsRunSummary {
        Extraction::backfill()
            .submissions()
            .ciks(ciks.iter().copied())
            .sec_client(sec_client())
            .repository(repository)
            .run(|_, _| {})
            .await
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_backfill_submissions() {
        implements_auto_traits::<BackfillSubmissions>();
    }

    #[test]
    const fn should_implement_auto_traits_for_submissions_batch_builder() {
        implements_auto_traits::<
            SubmissionsBatchBuilder<AlwaysRespondingSecClient, FakeSubmissionsRepository>,
        >();
    }

    #[test]
    fn should_set_the_execution_id_and_the_backfill_mode_when_building_one_cik() {
        let backfill = Extraction::backfill().execution_id(Uuid::from_u128(7));
        let cik = "320193";

        let expected_result = ExtractSubmissionsSuperState::builder()
            .cik(cik)
            .sec_client(sec_client())
            .execution_id(Uuid::from_u128(7))
            .ingestion_mode(IngestionMode::Backfill)
            .repository(FakeSubmissionsRepository::new())
            .build();

        let result = backfill
            .submissions()
            .cik(cik)
            .sec_client(sec_client())
            .repository(FakeSubmissionsRepository::new())
            .build();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_one_file_per_cik_with_the_backfill_mode_when_running_a_batch() {
        let repository = FakeSubmissionsRepository::new();
        let ciks = ["320193", "1067983"];

        let expected_result = vec![
            (Uuid::from_u128(7), IngestionMode::Backfill),
            (Uuid::from_u128(7), IngestionMode::Backfill),
        ];

        Extraction::backfill()
            .execution_id(Uuid::from_u128(7))
            .submissions()
            .ciks(ciks)
            .sec_client(sec_client())
            .repository(repository.clone())
            .run(|_, _| {})
            .await;
        let result: Vec<(Uuid, IngestionMode)> = repository
            .persisted()
            .iter()
            .map(|document| {
                let metadata = document.metadata();
                (metadata.execution_id(), metadata.ingestion_mode())
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_other_ciks_when_one_cik_fails() {
        let repository = FakeSubmissionsRepository::new();
        let ciks = ["ABC", "320193", "1067983"];

        let expected_result = 2;

        run_batch(repository.clone(), &ciks).await;
        let result = repository.persisted().len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_submissions_backfill_when_a_cik_fails() {
        let ciks = ["ABC", "320193"];

        let expected_result = Err(FailedSubmissionsBackfill {
            failures: 1,
            ciks: 2,
        });

        let result = run_batch(FakeSubmissionsRepository::new(), &ciks)
            .await
            .result();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_start_at_most_the_default_cap_of_ciks_when_in_flight_is_not_set() {
        let ciks = ["1", "2", "3", "4", "5", "6"];
        let rate_limiter = HoldingRateLimiter::forever();
        let turns = Arc::clone(&rate_limiter.turns);
        let sec_client = sec_client().with_rate_limiter(rate_limiter);
        let mut run = pin!(
            Extraction::backfill()
                .submissions()
                .ciks(ciks)
                .sec_client(sec_client)
                .repository(FakeSubmissionsRepository::new())
                .run(|_, _| {})
        );

        let expected_result = 4;

        let _pending = futures_util::poll!(run.as_mut());
        let result = turns.load(Ordering::SeqCst);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_zero_as_one_when_setting_in_flight() {
        let in_flight = 0;

        let expected_result = 1;

        let result = Extraction::backfill()
            .submissions()
            .ciks::<FakeSubmissionsRepository>(["320193"])
            .in_flight(in_flight)
            .in_flight;

        assert_eq!(result, expected_result);
    }
}
