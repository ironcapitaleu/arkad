//! # Store Filing Documents Context
//!
//! Provides the [`StoreFilingDocumentsContext`] carried alongside the
//! [`StoreFilingDocuments`](crate::implementations::states::extract::store_filing_documents::StoreFilingDocuments)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::Context as SMContext;
use uuid::Uuid;

use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::{RawXbrlInstance, RawXbrlLabels};
use crate::traits::state_machine::state::Context;

/// Context for the [`StoreFilingDocuments`](super::StoreFilingDocuments) state.
///
/// A repository writes the documents of one source, so the instance and the label file each get
/// a repository.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct StoreFilingDocumentsContext<I, L> {
    /// The ID of the run.
    pub execution_id: Uuid,
    /// The repository of the XBRL instances.
    pub instance_repository: I,
    /// The repository of the label files.
    pub labels_repository: L,
    /// The maximum number of retries of the state.
    ///
    /// A retry after a failed write returns [`StateError::InvalidInput`], because the failed run
    /// consumed the responses. A retry needs a new input from
    /// `ExecuteSecRequest<FilingDocumentSet, …>`.
    ///
    /// [`StateError::InvalidInput`]: crate::error::State::InvalidInput
    pub max_retries: u32,
}

impl<I, L> StoreFilingDocumentsContext<I, L> {
    /// Creates a new [`StoreFilingDocumentsContext`] with a retry budget of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::store_filing_documents::StoreFilingDocumentsContext;
    /// use uuid::Uuid;
    ///
    /// let context = StoreFilingDocumentsContext::new(Uuid::nil(), (), ());
    ///
    /// let expected_result = 0;
    ///
    /// let result = context.max_retries;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(execution_id: Uuid, instance_repository: I, labels_repository: L) -> Self {
        Self {
            execution_id,
            instance_repository,
            labels_repository,
            max_retries: 0,
        }
    }
}

impl<I, L> Context for StoreFilingDocumentsContext<I, L>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
{
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl<I, L> SMContext for StoreFilingDocumentsContext<I, L>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
{
    type UpdateType = StoreFilingDocumentsContextUpdater<I, L>;

    fn context(&self) -> &Self {
        self
    }

    /// Replaces each field that the updater sets.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(execution_id) = updates.execution_id {
            self.execution_id = execution_id;
        }
        if let Some(instance_repository) = updates.instance_repository {
            self.instance_repository = instance_repository;
        }
        if let Some(labels_repository) = updates.labels_repository {
            self.labels_repository = labels_repository;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl<I, L> fmt::Display for StoreFilingDocumentsContext<I, L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tExecution ID: {}", self.execution_id)
    }
}

/// Updater for [`StoreFilingDocumentsContext`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoreFilingDocumentsContextUpdater<I, L> {
    /// The new `execution_id`.
    pub execution_id: Option<Uuid>,
    /// The new repository of the XBRL instances.
    pub instance_repository: Option<I>,
    /// The new repository of the label files.
    pub labels_repository: Option<L>,
    /// The new retry budget.
    pub max_retries: Option<u32>,
}

impl<I, L> StoreFilingDocumentsContextUpdater<I, L> {
    /// Returns a [`StoreFilingDocumentsContextUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> StoreFilingDocumentsContextUpdaterBuilder<I, L> {
        StoreFilingDocumentsContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreFilingDocumentsContextUpdater`].
#[derive(Debug)]
pub struct StoreFilingDocumentsContextUpdaterBuilder<I, L> {
    execution_id: Option<Uuid>,
    instance_repository: Option<I>,
    labels_repository: Option<L>,
    max_retries: Option<u32>,
}

impl<I, L> StoreFilingDocumentsContextUpdaterBuilder<I, L> {
    /// Creates a [`StoreFilingDocumentsContextUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            execution_id: None,
            instance_repository: None,
            labels_repository: None,
            max_retries: None,
        }
    }

    /// Sets the `execution_id`.
    #[must_use]
    pub const fn execution_id(mut self, execution_id: Uuid) -> Self {
        self.execution_id = Some(execution_id);
        self
    }

    /// Sets the repository of the XBRL instances.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn instance_repository(mut self, instance_repository: I) -> Self {
        self.instance_repository = Some(instance_repository);
        self
    }

    /// Sets the repository of the label files.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn labels_repository(mut self, labels_repository: L) -> Self {
        self.labels_repository = Some(labels_repository);
        self
    }

    /// Sets the retry budget.
    #[must_use]
    pub const fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Builds the [`StoreFilingDocumentsContextUpdater`].
    #[must_use]
    pub fn build(self) -> StoreFilingDocumentsContextUpdater<I, L> {
        StoreFilingDocumentsContextUpdater {
            execution_id: self.execution_id,
            instance_repository: self.instance_repository,
            labels_repository: self.labels_repository,
            max_retries: self.max_retries,
        }
    }
}

impl<I, L> Default for StoreFilingDocumentsContextUpdaterBuilder<I, L> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use storage::WriteError;

    use super::*;
    use crate::tests::fixtures::sample_write_repository::{
        FakeXbrlInstanceRepository, FakeXbrlLabelsRepository,
    };

    type TestContext =
        StoreFilingDocumentsContext<FakeXbrlInstanceRepository, FakeXbrlLabelsRepository>;

    fn test_context() -> TestContext {
        StoreFilingDocumentsContext::new(
            Uuid::nil(),
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        )
    }

    #[test]
    fn should_update_execution_id_when_update_contains_execution_id() {
        let mut context = test_context();
        let update = StoreFilingDocumentsContextUpdater::builder()
            .execution_id(Uuid::from_u128(7))
            .build();

        let expected_result = Uuid::from_u128(7);

        context.update_context(update);
        let result = context.execution_id;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_instance_repository_when_update_contains_instance_repository() {
        let mut context = test_context();
        let rejecting =
            FakeXbrlInstanceRepository::rejecting(WriteError::conflicting_write("Sample"));
        let update = StoreFilingDocumentsContextUpdater::builder()
            .instance_repository(rejecting.clone())
            .build();

        let expected_result = rejecting;

        context.update_context(update);
        let result = context.instance_repository;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_labels_repository_when_update_contains_labels_repository() {
        let mut context = test_context();
        let rejecting =
            FakeXbrlLabelsRepository::rejecting(WriteError::conflicting_write("Sample"));
        let update = StoreFilingDocumentsContextUpdater::builder()
            .labels_repository(rejecting.clone())
            .build();

        let expected_result = rejecting;

        context.update_context(update);
        let result = context.labels_repository;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_update_contains_max_retries() {
        let mut context = test_context();
        let update = StoreFilingDocumentsContextUpdater::builder()
            .max_retries(5)
            .build();

        let expected_result = 5;

        context.update_context(update);
        let result = context.max_retries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_context_unchanged_when_update_is_empty() {
        let mut context = test_context();
        let update = StoreFilingDocumentsContextUpdater::builder().build();

        let expected_result = &test_context();

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_filing_documents_context() {
        implements_value_traits::<TestContext>();
    }
}
