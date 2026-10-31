//! # Store Submissions Context
//!
//! Provides the [`StoreSubmissionsContext`] carried alongside the
//! [`StoreSubmissions`](crate::implementations::states::extract::store_submissions::StoreSubmissions)
//! state, together with its updater and its builder.
//!
//! The context holds the [`Cik`] being processed, the `execution_id`, the [`IngestionMode`] of the
//! run, the repository that stores the document, and the retry budget.

use std::fmt::{self, Debug, Display};
use std::hash::Hash;

use serde::Serialize;
use state_maschine::prelude::Context as SMContext;
use storage::implementations::repository::filesystem::IngestionMode;
use uuid::Uuid;

use crate::shared::cik::Cik;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawSubmissions;
use crate::traits::state_machine::state::Context;

/// Context for the [`StoreSubmissions`](super::StoreSubmissions) state.
///
/// Holds the [`Cik`] that names the stored document, the `execution_id` and the [`IngestionMode`]
/// that the metadata records, and the repository that stores the document. Both ingestion paths
/// store `submissions`, so the caller sets the mode.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct StoreSubmissionsContext<R> {
    /// The Central Index Key (CIK) of the company whose `submissions` JSON the state stores.
    pub cik: Cik,
    /// The id of the run. The metadata of the stored document records it.
    pub execution_id: Uuid,
    /// The ingestion mode of the run. The metadata of the stored document records it.
    pub ingestion_mode: IngestionMode,
    /// The repository that stores the document.
    pub repository: R,
    /// Maximum number of times the state may be retried on failure.
    pub max_retries: u32,
}

impl<R> StoreSubmissionsContext<R> {
    /// Creates a new [`StoreSubmissionsContext`] with the retry budget at zero.
    #[must_use]
    pub const fn new(
        cik: Cik,
        execution_id: Uuid,
        ingestion_mode: IngestionMode,
        repository: R,
    ) -> Self {
        Self {
            cik,
            execution_id,
            ingestion_mode,
            repository,
            max_retries: 0,
        }
    }

    /// Returns a reference to the CIK being processed.
    #[must_use]
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }

    /// Returns the id of the run.
    #[must_use]
    pub const fn execution_id(&self) -> Uuid {
        self.execution_id
    }

    /// Returns the ingestion mode of the run.
    #[must_use]
    pub const fn ingestion_mode(&self) -> IngestionMode {
        self.ingestion_mode
    }

    /// Returns a reference to the repository that stores the document.
    #[must_use]
    pub const fn repository(&self) -> &R {
        &self.repository
    }
}

impl<R: RawRepository<RawSubmissions>> Context for StoreSubmissionsContext<R> {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl<R: RawRepository<RawSubmissions>> SMContext for StoreSubmissionsContext<R> {
    type UpdateType = StoreSubmissionsContextUpdater<R>;

    fn context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.cik {
            self.cik = cik;
        }
        if let Some(execution_id) = updates.execution_id {
            self.execution_id = execution_id;
        }
        if let Some(ingestion_mode) = updates.ingestion_mode {
            self.ingestion_mode = ingestion_mode;
        }
        if let Some(repository) = updates.repository {
            self.repository = repository;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl<R: Display> fmt::Display for StoreSubmissionsContext<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Context Data: {}, Execution ID: {}, Ingestion Mode: {}, Repository: {}",
            self.cik, self.execution_id, self.ingestion_mode, self.repository
        )
    }
}

/// Updater for modifying [`StoreSubmissionsContext`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoreSubmissionsContextUpdater<R> {
    /// Optional new value for the CIK.
    pub cik: Option<Cik>,
    /// Optional new value for the id of the run.
    pub execution_id: Option<Uuid>,
    /// Optional new value for the ingestion mode.
    pub ingestion_mode: Option<IngestionMode>,
    /// Optional new value for the repository.
    pub repository: Option<R>,
    /// Optional new value for the retry budget.
    pub max_retries: Option<u32>,
}

impl<R> StoreSubmissionsContextUpdater<R> {
    /// Creates a new builder for constructing [`StoreSubmissionsContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> StoreSubmissionsContextUpdaterBuilder<R> {
        StoreSubmissionsContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreSubmissionsContextUpdater`].
pub struct StoreSubmissionsContextUpdaterBuilder<R> {
    cik: Option<Cik>,
    execution_id: Option<Uuid>,
    ingestion_mode: Option<IngestionMode>,
    repository: Option<R>,
    max_retries: Option<u32>,
}

impl<R> StoreSubmissionsContextUpdaterBuilder<R> {
    /// Creates a new [`StoreSubmissionsContextUpdaterBuilder`] with all fields initialized to
    /// `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cik: None,
            execution_id: None,
            ingestion_mode: None,
            repository: None,
            max_retries: None,
        }
    }

    /// Sets the CIK field.
    #[must_use]
    pub fn cik(mut self, cik: Cik) -> Self {
        self.cik = Some(cik);
        self
    }

    /// Sets the `execution_id` field.
    #[must_use]
    pub const fn execution_id(mut self, execution_id: Uuid) -> Self {
        self.execution_id = Some(execution_id);
        self
    }

    /// Sets the ingestion mode field.
    #[must_use]
    pub const fn ingestion_mode(mut self, ingestion_mode: IngestionMode) -> Self {
        self.ingestion_mode = Some(ingestion_mode);
        self
    }

    /// Sets the repository field.
    #[must_use]
    pub fn repository(mut self, repository: R) -> Self {
        self.repository = Some(repository);
        self
    }

    /// Sets the `max_retries` field.
    #[must_use]
    pub const fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Builds the [`StoreSubmissionsContextUpdater`].
    #[must_use]
    pub fn build(self) -> StoreSubmissionsContextUpdater<R> {
        StoreSubmissionsContextUpdater {
            cik: self.cik,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            repository: self.repository,
            max_retries: self.max_retries,
        }
    }
}

impl<R> Default for StoreSubmissionsContextUpdaterBuilder<R> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use storage::WriteError;

    use super::*;
    use crate::tests::fixtures::sample_write_repository::FakeSubmissionsRepository;

    fn sample_cik(cik: &str) -> Cik {
        Cik::new(cik).expect("Given a hardcoded numeric CIK, its creation should always succeed")
    }

    fn sample_context() -> StoreSubmissionsContext<FakeSubmissionsRepository> {
        StoreSubmissionsContext::new(
            sample_cik("1067983"),
            Uuid::nil(),
            IngestionMode::Incremental,
            FakeSubmissionsRepository::new(),
        )
    }

    #[test]
    fn should_set_zero_max_retries_when_created_with_new() {
        let context = sample_context();

        let expected_result = 0;

        let result = context.max_retries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_reference_to_self_when_accessing_context() {
        let context = sample_context();

        let expected_result = &context.clone();

        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_when_updater_contains_cik() {
        let mut context = sample_context();
        let updater = StoreSubmissionsContextUpdater::builder()
            .cik(sample_cik("320193"))
            .build();

        let expected_result = &sample_cik("320193");

        context.update_context(updater);
        let result = context.cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_execution_id_when_updater_contains_execution_id() {
        let mut context = sample_context();
        let execution_id = Uuid::from_u128(1);
        let updater = StoreSubmissionsContextUpdater::builder()
            .execution_id(execution_id)
            .build();

        let expected_result = execution_id;

        context.update_context(updater);
        let result = context.execution_id();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_ingestion_mode_when_updater_contains_ingestion_mode() {
        let mut context = sample_context();
        let updater = StoreSubmissionsContextUpdater::builder()
            .ingestion_mode(IngestionMode::Backfill)
            .build();

        let expected_result = IngestionMode::Backfill;

        context.update_context(updater);
        let result = context.ingestion_mode();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_repository_when_updater_contains_repository() {
        let mut context = sample_context();
        let repository =
            FakeSubmissionsRepository::rejecting(WriteError::conflicting_write("taken"));
        let updater = StoreSubmissionsContextUpdater::builder()
            .repository(repository.clone())
            .build();

        let expected_result = &repository;

        context.update_context(updater);
        let result = context.repository();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_updater_contains_max_retries() {
        let mut context = sample_context();
        let updater = StoreSubmissionsContextUpdater::builder()
            .max_retries(5)
            .build();

        let expected_result = 5;

        context.update_context(updater);
        let result = context.max_retries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_context_unchanged_when_updater_is_empty() {
        let mut context = sample_context();
        let updater = StoreSubmissionsContextUpdater::builder().build();

        let expected_result = sample_context();

        context.update_context(updater);
        let result = context;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_include_the_repository_when_serializing_the_context() {
        let context = sample_context();

        let expected_result = serde_json::json!({
            "cik": { "value": "0001067983" },
            "execution_id": "00000000-0000-0000-0000-000000000000",
            "ingestion_mode": "incremental",
            "repository": { "kind": "fake", "rejection": null },
            "max_retries": 0,
        });

        let result = serde_json::to_value(context)
            .expect("Given a context of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_cik_the_execution_id_the_ingestion_mode_and_the_repository_when_formatted()
     {
        let context = sample_context();

        let expected_result = "Context Data: 0001067983, Execution ID: 00000000-0000-0000-0000-000000000000, Ingestion Mode: incremental, Repository: Fake write repository";

        let result = context.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_store_submissions_context() {
        implements_auto_traits::<StoreSubmissionsContext<FakeSubmissionsRepository>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_be_thread_safe_for_store_submissions_context() {
        implements_send::<StoreSubmissionsContext<FakeSubmissionsRepository>>();
        implements_sync::<StoreSubmissionsContext<FakeSubmissionsRepository>>();
    }

    const fn implements_context<T: Context>() {}
    #[test]
    const fn should_implement_context_for_store_submissions_context() {
        implements_context::<StoreSubmissionsContext<FakeSubmissionsRepository>>();
    }
}
