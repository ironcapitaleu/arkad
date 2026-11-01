//! # Company Facts
//!
//! Provides [`BackfillCompanyFacts`], the Company Facts source of a backfill run, and the batch run
//! over many CIKs with its [`CompanyFactsRunSummary`].
//!
//! [`BackfillCompanyFacts::cik`] returns the builder of one [`ExtractCompanyFactsSuperState`].
//! [`BackfillCompanyFacts::ciks`] returns a [`CompanyFactsBatchBuilder`], whose [`run`] extracts
//! the `companyfacts` JSON of each CIK into the store.
//!
//! [`run`]: CompanyFactsBatchBuilder::run

use std::sync::Arc;

use futures_util::StreamExt;
use storage::implementations::repository::filesystem::IngestionMode;
use uuid::Uuid;

use super::DEFAULT_CIKS_IN_FLIGHT;
use super::run_event::{RunEvent, Unit, drain};
use crate::implementations::states::extract::company_facts::ExtractCompanyFactsSuperState;
use crate::implementations::states::extract::company_facts::builder::{
    ExtractCompanyFactsSuperStateBuilder, NoRepository, NoSecClient,
};
use crate::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawCompanyFacts;
use crate::shared::response::implementations::sec_response::{JsonBody, SecResponse};
use crate::traits::state_machine::stream::IntoStateMachineStream;

mod summary;

pub use summary::{CompanyFactsRunSummary, FailedCompanyFactsBackfill};

/// The Company Facts source of a backfill run, before the choice of one CIK or many.
///
/// Every super-state built from it carries the `execution_id` of its
/// [`Backfill`](super::Backfill) handle and [`IngestionMode::Backfill`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BackfillCompanyFacts {
    execution_id: Uuid,
}

impl BackfillCompanyFacts {
    /// Creates a new [`BackfillCompanyFacts`] for the run with `execution_id`.
    pub(super) const fn new(execution_id: Uuid) -> Self {
        Self { execution_id }
    }

    /// Returns the builder of an [`ExtractCompanyFactsSuperState`] for one raw CIK, for example
    /// `320193`.
    ///
    /// The builder still needs an SEC client and a repository of type `R`.
    #[must_use]
    pub fn cik<R: RawRepository<RawCompanyFacts>>(
        self,
        cik: impl Into<String>,
    ) -> ExtractCompanyFactsSuperStateBuilder<
        String,
        NoSecClient,
        Uuid,
        IngestionMode,
        NoRepository<R>,
    > {
        ExtractCompanyFactsSuperState::builder()
            .execution_id(self.execution_id)
            .ingestion_mode(IngestionMode::Backfill)
            .cik(cik)
    }

    /// Returns the builder of a batch run over `ciks`. The summary lists the CIKs in the order of
    /// the list.
    ///
    /// The builder still needs an SEC client and a repository of type `R`.
    #[must_use]
    pub fn ciks<R: RawRepository<RawCompanyFacts>>(
        self,
        ciks: impl IntoIterator<Item = impl Into<String>>,
    ) -> CompanyFactsBatchBuilder<NoSecClient, NoRepository<R>> {
        CompanyFactsBatchBuilder {
            company_facts: self,
            ciks: ciks.into_iter().map(Into::into).collect(),
            sec_client: NoSecClient,
            repository: NoRepository::new(),
            in_flight: DEFAULT_CIKS_IN_FLIGHT,
        }
    }
}

/// Builds and runs a batch of CIKs, one [`ExtractCompanyFactsSuperState`] per CIK.
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
/// use sec::shared::raw_source::RawCompanyFacts;
/// use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
///
/// let _state = Extraction::backfill()
///     .company_facts()
///     .ciks::<FilesystemRepository<Write, RawDocument<RawCompanyFacts>>>(["320193"])
///     .build();
/// ```
///
/// And `run` does not follow `cik`:
///
/// ```compile_fail,E0599
/// use sec::implementations::extraction::Extraction;
/// use sec::shared::raw_document::RawDocument;
/// use sec::shared::raw_source::RawCompanyFacts;
/// use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
///
/// let _run = Extraction::backfill()
///     .company_facts()
///     .cik::<FilesystemRepository<Write, RawDocument<RawCompanyFacts>>>("320193")
///     .run(|_, _| {});
/// ```
#[derive(Debug, Clone)]
pub struct CompanyFactsBatchBuilder<C, R> {
    company_facts: BackfillCompanyFacts,
    ciks: Vec<String>,
    sec_client: C,
    repository: R,
    in_flight: usize,
}

impl<R> CompanyFactsBatchBuilder<NoSecClient, R> {
    /// Sets the SEC client that downloads the Company Facts.
    #[must_use]
    pub fn sec_client<C>(self, sec_client: C) -> CompanyFactsBatchBuilder<C, R>
    where
        C: SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>, Error = FailedSecRequest>,
    {
        CompanyFactsBatchBuilder {
            company_facts: self.company_facts,
            ciks: self.ciks,
            sec_client,
            repository: self.repository,
            in_flight: self.in_flight,
        }
    }
}

impl<C, R: RawRepository<RawCompanyFacts>> CompanyFactsBatchBuilder<C, NoRepository<R>> {
    /// Sets the repository that stores the raw Company Facts.
    #[must_use]
    pub fn repository(self, repository: R) -> CompanyFactsBatchBuilder<C, R> {
        CompanyFactsBatchBuilder {
            company_facts: self.company_facts,
            ciks: self.ciks,
            sec_client: self.sec_client,
            repository,
            in_flight: self.in_flight,
        }
    }
}

impl<C, R> CompanyFactsBatchBuilder<C, R> {
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

impl<C, R> CompanyFactsBatchBuilder<C, R>
where
    C: SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>, Error = FailedSecRequest>,
    R: RawRepository<RawCompanyFacts>,
    ExtractCompanyFactsSuperState<ValidateCikFormat, R, C>: IntoStateMachineStream,
{
    /// Extracts the Company Facts of each CIK into the repository and returns the summary.
    ///
    /// At most [`in_flight`](Self::in_flight) CIKs run at once. A failed CIK does not stop the others.
    ///
    /// Calls `observer` with the CIK and each successful stream item, and with the CIK and the
    /// error of a failed CIK. Every error gives
    /// [`ErrorKind::Failed`](super::ErrorKind::Failed).
    pub async fn run(
        self,
        observer: impl Fn(&Unit, RunEvent<'_>) + Send + Sync + 'static,
    ) -> CompanyFactsRunSummary {
        let observer = Arc::new(observer);
        let extractions = self.ciks.into_iter().enumerate().map(|(index, cik)| {
            let super_state = self
                .company_facts
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
        CompanyFactsRunSummary(
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
    use std::sync::Mutex;
    use std::sync::atomic::Ordering;

    use pretty_assertions::assert_eq;
    use serde_json::{Value, json};
    use storage::implementations::repository::filesystem::Document;

    use super::*;
    use crate::implementations::extraction::{ErrorKind, Extraction};
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_rate_limiter::HoldingRateLimiter;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    const LOCK_EXPECT: &str = "Given an observer that never panics while it holds the lock, the lock should never be poisoned";

    fn sec_client() -> AlwaysRespondingSecClient {
        AlwaysRespondingSecClient::new(
            200,
            vec![("content-type", "application/json".to_string())],
            br#"{"cik":1067983,"facts":{}}"#.to_vec(),
        )
    }

    /// Runs a batch over `ciks` into `repository` and passes each event to `observer`.
    async fn run_batch(
        repository: FakeWriteRepository,
        ciks: &[&str],
        observer: impl Fn(&Unit, RunEvent<'_>) + Send + Sync + 'static,
    ) -> CompanyFactsRunSummary {
        Extraction::backfill()
            .company_facts()
            .ciks(ciks.iter().copied())
            .sec_client(sec_client())
            .repository(repository)
            .run(observer)
            .await
    }

    /// Returns the `execution_id` and the ingestion mode of each document that `repository` holds.
    fn run_fields_of(repository: &FakeWriteRepository) -> Vec<(Uuid, IngestionMode)> {
        repository
            .persisted()
            .iter()
            .map(|document| {
                let metadata = document.metadata();
                (metadata.execution_id(), metadata.ingestion_mode())
            })
            .collect()
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_backfill_company_facts() {
        implements_auto_traits::<BackfillCompanyFacts>();
    }

    #[test]
    const fn should_implement_auto_traits_for_company_facts_batch_builder() {
        implements_auto_traits::<
            CompanyFactsBatchBuilder<AlwaysRespondingSecClient, FakeWriteRepository>,
        >();
    }

    #[test]
    fn should_set_the_execution_id_and_the_backfill_mode_when_building_one_cik() {
        let backfill = Extraction::backfill().execution_id(Uuid::from_u128(7));
        let cik = "320193";

        let expected_result = ExtractCompanyFactsSuperState::builder()
            .cik(cik)
            .sec_client(sec_client())
            .execution_id(Uuid::from_u128(7))
            .ingestion_mode(IngestionMode::Backfill)
            .repository(FakeWriteRepository::new())
            .build();

        let result = backfill
            .company_facts()
            .cik(cik)
            .sec_client(sec_client())
            .repository(FakeWriteRepository::new())
            .build();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_give_two_machines_the_execution_id_of_the_handle_when_both_come_from_one_handle()
     {
        let backfill = Extraction::backfill();
        let execution_id = backfill.id();
        let repository = FakeWriteRepository::new();
        let first = backfill
            .company_facts()
            .cik("320193")
            .sec_client(sec_client())
            .repository(repository.clone())
            .build();
        let second = backfill
            .company_facts()
            .cik("1067983")
            .sec_client(sec_client())
            .repository(repository.clone())
            .build();

        let expected_result = vec![
            (execution_id, IngestionMode::Backfill),
            (execution_id, IngestionMode::Backfill),
        ];

        first.into_run_stream().collect::<Vec<_>>().await;
        second.into_run_stream().collect::<Vec<_>>().await;
        let result = run_fields_of(&repository);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_execution_id_and_the_backfill_mode_when_running_a_batch() {
        let repository = FakeWriteRepository::new();
        let ciks = ["320193", "1067983"];

        let expected_result = vec![
            (Uuid::from_u128(7), IngestionMode::Backfill),
            (Uuid::from_u128(7), IngestionMode::Backfill),
        ];

        Extraction::backfill()
            .execution_id(Uuid::from_u128(7))
            .company_facts()
            .ciks(ciks)
            .sec_client(sec_client())
            .repository(repository.clone())
            .run(|_, _| {})
            .await;
        let result = run_fields_of(&repository);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_company_facts_backfill_when_a_cik_fails() {
        let ciks = ["ABC", "320193"];

        let expected_result = Err(FailedCompanyFactsBackfill {
            failures: 1,
            ciks: 2,
        });

        let result = run_batch(FakeWriteRepository::new(), &ciks, |_, _| {})
            .await
            .result();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_list_the_entries_in_the_order_of_the_ciks_when_the_first_cik_finishes_last() {
        let ciks = ["320193", "ABC", "1067983"];
        let sec_client = sec_client().with_rate_limiter(HoldingRateLimiter::first_turn_yields(16));

        let expected_result = vec![json!("320193"), json!("ABC"), json!("1067983")];

        let result: Vec<Value> = Extraction::backfill()
            .company_facts()
            .ciks(ciks)
            .sec_client(sec_client)
            .repository(FakeWriteRepository::new())
            .run(|_, _| {})
            .await
            .entries()
            .into_iter()
            .map(|entry| entry["cik"].clone())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_pass_the_failed_kind_to_the_observer_when_the_cik_is_invalid() {
        let ciks = ["ABC"];
        let events = Arc::new(Mutex::new(Vec::new()));
        let observed_events = Arc::clone(&events);
        let observer = move |unit: &Unit, event: RunEvent<'_>| {
            if let RunEvent::Error(_, kind) = event {
                observed_events
                    .lock()
                    .expect(LOCK_EXPECT)
                    .push((unit.clone(), kind));
            }
        };

        let expected_result = vec![(Unit::Cik("ABC".to_string()), ErrorKind::Failed)];

        run_batch(FakeWriteRepository::new(), &ciks, observer).await;
        let result = events.lock().expect(LOCK_EXPECT).clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_pass_each_cik_to_the_observer_when_the_run_extracts_the_ciks() {
        let ciks = ["320193", "1067983"];
        let units = Arc::new(Mutex::new(Vec::new()));
        let observed_units = Arc::clone(&units);
        let observer = move |unit: &Unit, _: RunEvent<'_>| {
            observed_units.lock().expect(LOCK_EXPECT).push(unit.clone());
        };

        let expected_result = vec![
            Unit::Cik("1067983".to_string()),
            Unit::Cik("320193".to_string()),
        ];

        run_batch(FakeWriteRepository::new(), &ciks, observer).await;
        let mut result = units.lock().expect(LOCK_EXPECT).clone();
        result.sort();
        result.dedup();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_start_at_most_in_flight_ciks_when_the_ciks_wait_for_their_turn() {
        let ciks = ["320193", "1067983", "789019"];
        let rate_limiter = HoldingRateLimiter::forever();
        let turns = Arc::clone(&rate_limiter.turns);
        let sec_client = sec_client().with_rate_limiter(rate_limiter);
        let mut run = pin!(
            Extraction::backfill()
                .company_facts()
                .ciks(ciks)
                .sec_client(sec_client)
                .repository(FakeWriteRepository::new())
                .in_flight(2)
                .run(|_, _| {})
        );

        let expected_result = 2;

        let _pending = futures_util::poll!(run.as_mut());
        let result = turns.load(Ordering::SeqCst);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_zero_as_one_when_setting_in_flight() {
        let in_flight = 0;

        let expected_result = 1;

        let result = Extraction::backfill()
            .company_facts()
            .ciks::<FakeWriteRepository>(["320193"])
            .in_flight(in_flight)
            .in_flight;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_use_the_default_cap_when_in_flight_is_not_set() {
        let ciks = ["320193"];

        let expected_result = 4;

        let result = Extraction::backfill()
            .company_facts()
            .ciks::<FakeWriteRepository>(ciks)
            .in_flight;

        assert_eq!(result, expected_result);
    }
}
