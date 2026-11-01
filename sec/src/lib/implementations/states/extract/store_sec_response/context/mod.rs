//! # Store SEC Response Context
//!
//! Provides the [`StoreSecResponseContext`] carried alongside the
//! [`StoreSecResponse`](crate::implementations::states::extract::store_sec_response::StoreSecResponse)
//! state, together with its updater and its builder.
//!
//! The context holds the request input that names the stored document, the `execution_id`, the
//! [`IngestionMode`] of the run, the repository that stores the document, and the retry budget.

use std::fmt::{self, Display};

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use state_maschine::prelude::Context as SMContext;
use storage::implementations::repository::filesystem::IngestionMode;
use uuid::Uuid;

#[cfg(doc)]
use crate::shared::cik::Cik;
use crate::shared::raw_repository::RawRepository;
use crate::shared::request::implementations::sec_request::{RequestKind, StoredAs};
use crate::traits::state_machine::state::Context;

/// Context for the [`StoreSecResponse`](super::StoreSecResponse) state.
///
/// Holds the request input of kind `E` that names the stored document, for example a [`Cik`], the
/// `execution_id` and the [`IngestionMode`] that the metadata records, and the repository of type
/// `R` that stores the document. The mode belongs to the run, because one kind can run as a
/// backfill and as an incremental fetch.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoreSecResponseContext<E: RequestKind, R> {
    /// The input of the request, for example the [`Cik`] of a Company Facts request.
    pub request_input: E::Input,
    /// The id of the run. The metadata of the stored document records it.
    pub execution_id: Uuid,
    /// The ingestion mode of the run. The metadata of the stored document records it.
    pub ingestion_mode: IngestionMode,
    /// The repository that stores the document.
    pub repository: R,
    /// Maximum number of times the state may be retried on failure.
    pub max_retries: u32,
}

impl<E: RequestKind, R> StoreSecResponseContext<E, R> {
    /// Creates a new [`StoreSecResponseContext`] with the retry budget at zero.
    #[must_use]
    pub const fn new(
        request_input: E::Input,
        execution_id: Uuid,
        ingestion_mode: IngestionMode,
        repository: R,
    ) -> Self {
        Self {
            request_input,
            execution_id,
            ingestion_mode,
            repository,
            max_retries: 0,
        }
    }

    /// Returns a reference to the request input.
    #[must_use]
    pub const fn request_input(&self) -> &E::Input {
        &self.request_input
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

/// Serializes the request input under the name its kind gives it, for example `cik`.
impl<E: RequestKind, R: Serialize> Serialize for StoreSecResponseContext<E, R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("StoreSecResponseContext", 5)?;
        state.serialize_field(E::INPUT_NAME, &self.request_input)?;
        state.serialize_field("execution_id", &self.execution_id)?;
        state.serialize_field("ingestion_mode", &self.ingestion_mode)?;
        state.serialize_field("repository", &self.repository)?;
        state.serialize_field("max_retries", &self.max_retries)?;
        state.end()
    }
}

impl<E: StoredAs, R: RawRepository<E::Source>> Context for StoreSecResponseContext<E, R> {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl<E: StoredAs, R: RawRepository<E::Source>> SMContext for StoreSecResponseContext<E, R> {
    type UpdateType = StoreSecResponseContextUpdater<E, R>;

    fn context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(request_input) = updates.request_input {
            self.request_input = request_input;
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

impl<E: RequestKind, R: Display> fmt::Display for StoreSecResponseContext<E, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Context Data: {}, Execution ID: {}, Repository: {}",
            self.request_input, self.execution_id, self.repository
        )
    }
}

/// Updater for modifying [`StoreSecResponseContext`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoreSecResponseContextUpdater<E: RequestKind, R> {
    /// Optional new value for the request input.
    pub request_input: Option<E::Input>,
    /// Optional new value for the id of the run.
    pub execution_id: Option<Uuid>,
    /// Optional new value for the ingestion mode.
    pub ingestion_mode: Option<IngestionMode>,
    /// Optional new value for the repository.
    pub repository: Option<R>,
    /// Optional new value for the retry budget.
    pub max_retries: Option<u32>,
}

impl<E: RequestKind, R> StoreSecResponseContextUpdater<E, R> {
    /// Creates a new builder for constructing [`StoreSecResponseContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> StoreSecResponseContextUpdaterBuilder<E, R> {
        StoreSecResponseContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreSecResponseContextUpdater`].
pub struct StoreSecResponseContextUpdaterBuilder<E: RequestKind, R> {
    request_input: Option<E::Input>,
    execution_id: Option<Uuid>,
    ingestion_mode: Option<IngestionMode>,
    repository: Option<R>,
    max_retries: Option<u32>,
}

impl<E: RequestKind, R> StoreSecResponseContextUpdaterBuilder<E, R> {
    /// Creates a new [`StoreSecResponseContextUpdaterBuilder`] with all fields initialized to
    /// `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            request_input: None,
            execution_id: None,
            ingestion_mode: None,
            repository: None,
            max_retries: None,
        }
    }

    /// Sets the request input field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn request_input(mut self, request_input: E::Input) -> Self {
        self.request_input = Some(request_input);
        self
    }

    /// Sets the `execution_id` field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn execution_id(mut self, execution_id: Uuid) -> Self {
        self.execution_id = Some(execution_id);
        self
    }

    /// Sets the ingestion mode field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn ingestion_mode(mut self, ingestion_mode: IngestionMode) -> Self {
        self.ingestion_mode = Some(ingestion_mode);
        self
    }

    /// Sets the repository field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn repository(mut self, repository: R) -> Self {
        self.repository = Some(repository);
        self
    }

    /// Sets the `max_retries` field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Builds the [`StoreSecResponseContextUpdater`].
    #[must_use]
    pub fn build(self) -> StoreSecResponseContextUpdater<E, R> {
        StoreSecResponseContextUpdater {
            request_input: self.request_input,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            repository: self.repository,
            max_retries: self.max_retries,
        }
    }
}

impl<E: RequestKind, R> Default for StoreSecResponseContextUpdaterBuilder<E, R> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use storage::WriteError;

    use super::*;
    use crate::implementations::states::extract::store_company_facts::StoreCompanyFactsContext;
    use crate::shared::cik::Cik;
    use crate::shared::request::implementations::sec_request::CompanyFacts;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    fn sample_cik(cik: &str) -> Cik {
        Cik::new(cik).expect("Given a hardcoded numeric CIK, its creation should always succeed")
    }

    fn sample_context() -> StoreCompanyFactsContext<FakeWriteRepository> {
        StoreCompanyFactsContext::new(
            sample_cik("1067983"),
            Uuid::nil(),
            IngestionMode::Backfill,
            FakeWriteRepository::new(),
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
    fn should_update_request_input_when_updater_contains_request_input() {
        let mut context = sample_context();
        let updater =
            StoreSecResponseContextUpdater::<CompanyFacts, FakeWriteRepository>::builder()
                .request_input(sample_cik("320193"))
                .build();

        let expected_result = &sample_cik("320193");

        context.update_context(updater);
        let result = context.request_input();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_execution_id_when_updater_contains_execution_id() {
        let mut context = sample_context();
        let execution_id = Uuid::from_u128(1);
        let updater =
            StoreSecResponseContextUpdater::<CompanyFacts, FakeWriteRepository>::builder()
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
        let updater =
            StoreSecResponseContextUpdater::<CompanyFacts, FakeWriteRepository>::builder()
                .ingestion_mode(IngestionMode::Incremental)
                .build();

        let expected_result = IngestionMode::Incremental;

        context.update_context(updater);
        let result = context.ingestion_mode();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_repository_when_updater_contains_repository() {
        let mut context = sample_context();
        let repository = FakeWriteRepository::rejecting(WriteError::conflicting_write("taken"));
        let updater =
            StoreSecResponseContextUpdater::<CompanyFacts, FakeWriteRepository>::builder()
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
        let updater =
            StoreSecResponseContextUpdater::<CompanyFacts, FakeWriteRepository>::builder()
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
        let updater =
            StoreSecResponseContextUpdater::<CompanyFacts, FakeWriteRepository>::builder().build();

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
            "ingestion_mode": "backfill",
            "repository": { "kind": "fake", "rejection": null },
            "max_retries": 0,
        });

        let result = serde_json::to_value(context)
            .expect("Given a context of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_cik_the_execution_id_and_the_repository_when_formatted() {
        let context = sample_context();

        let expected_result = "Context Data: 0001067983, Execution ID: 00000000-0000-0000-0000-000000000000, Repository: Fake write repository";

        let result = context.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_store_company_facts_context() {
        implements_auto_traits::<StoreCompanyFactsContext<FakeWriteRepository>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_be_thread_safe_for_store_company_facts_context() {
        implements_send::<StoreCompanyFactsContext<FakeWriteRepository>>();
        implements_sync::<StoreCompanyFactsContext<FakeWriteRepository>>();
    }

    const fn implements_context<T: Context>() {}
    #[test]
    const fn should_implement_context_for_store_company_facts_context() {
        implements_context::<StoreCompanyFactsContext<FakeWriteRepository>>();
    }
}
