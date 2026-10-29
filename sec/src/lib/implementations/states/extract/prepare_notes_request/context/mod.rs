//! # Prepare Notes Request Context
//!
//! Provides the [`PrepareNotesRequestContext`] carried alongside the
//! [`PrepareNotesRequest`](crate::implementations::states::extract::prepare_notes_request::PrepareNotesRequest)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::Context as SMContext;
use uuid::Uuid;

use crate::implementations::states::extract::notes_repository::NotesRepository;
use crate::traits::state_machine::state::Context;

/// Context for the [`PrepareNotesRequest`](super::PrepareNotesRequest) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct PrepareNotesRequestContext<R> {
    /// The ID of the run.
    pub execution_id: Uuid,
    /// The storage repository.
    pub repository: R,
    /// The maximum number of retries of the state.
    pub max_retries: u32,
}

impl<R> PrepareNotesRequestContext<R> {
    /// Creates a new [`PrepareNotesRequestContext`] with a retry budget of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::prepare_notes_request::PrepareNotesRequestContext;
    /// use uuid::Uuid;
    ///
    /// let context = PrepareNotesRequestContext::new(Uuid::nil(), ());
    ///
    /// let expected_result = 0;
    ///
    /// let result = context.max_retries;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(execution_id: Uuid, repository: R) -> Self {
        Self {
            execution_id,
            repository,
            max_retries: 0,
        }
    }
}

impl<R: NotesRepository> Context for PrepareNotesRequestContext<R> {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl<R: NotesRepository> SMContext for PrepareNotesRequestContext<R> {
    type UpdateType = PrepareNotesRequestContextUpdater<R>;

    fn context(&self) -> &Self {
        self
    }

    /// Replaces each field that the updater sets.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(execution_id) = updates.execution_id {
            self.execution_id = execution_id;
        }
        if let Some(repository) = updates.repository {
            self.repository = repository;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl<R> fmt::Display for PrepareNotesRequestContext<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tExecution ID: {}", self.execution_id)
    }
}

/// Updater for [`PrepareNotesRequestContext`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareNotesRequestContextUpdater<R> {
    /// The new `execution_id`.
    pub execution_id: Option<Uuid>,
    /// The new storage repository.
    pub repository: Option<R>,
    /// The new retry budget.
    pub max_retries: Option<u32>,
}

impl<R> PrepareNotesRequestContextUpdater<R> {
    /// Returns a [`PrepareNotesRequestContextUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> PrepareNotesRequestContextUpdaterBuilder<R> {
        PrepareNotesRequestContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`PrepareNotesRequestContextUpdater`].
#[derive(Debug)]
pub struct PrepareNotesRequestContextUpdaterBuilder<R> {
    execution_id: Option<Uuid>,
    repository: Option<R>,
    max_retries: Option<u32>,
}

impl<R> PrepareNotesRequestContextUpdaterBuilder<R> {
    /// Creates a [`PrepareNotesRequestContextUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            execution_id: None,
            repository: None,
            max_retries: None,
        }
    }

    /// Sets the `execution_id`.
    #[must_use]
    pub const fn execution_id(mut self, execution_id: Uuid) -> Self {
        self.execution_id = Some(execution_id);
        self
    }

    /// Sets the storage repository.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn repository(mut self, repository: R) -> Self {
        self.repository = Some(repository);
        self
    }

    /// Sets the retry budget.
    #[must_use]
    pub const fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Builds the [`PrepareNotesRequestContextUpdater`].
    #[must_use]
    pub fn build(self) -> PrepareNotesRequestContextUpdater<R> {
        PrepareNotesRequestContextUpdater {
            execution_id: self.execution_id,
            repository: self.repository,
            max_retries: self.max_retries,
        }
    }
}

impl<R> Default for PrepareNotesRequestContextUpdaterBuilder<R> {
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
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    fn test_context() -> PrepareNotesRequestContext<FakeWriteRepository> {
        PrepareNotesRequestContext::new(Uuid::nil(), FakeWriteRepository::new())
    }

    #[test]
    fn should_update_execution_id_when_update_contains_execution_id() {
        let mut context = test_context();
        let update = PrepareNotesRequestContextUpdater::builder()
            .execution_id(Uuid::from_u128(7))
            .build();

        let expected_result = Uuid::from_u128(7);

        context.update_context(update);
        let result = context.execution_id;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_repository_when_update_contains_repository() {
        let mut context = test_context();
        let rejecting = FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample"));
        let update = PrepareNotesRequestContextUpdater::builder()
            .repository(rejecting.clone())
            .build();

        let expected_result = rejecting;

        context.update_context(update);
        let result = context.repository;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_update_contains_max_retries() {
        let mut context = test_context();
        let update = PrepareNotesRequestContextUpdater::builder()
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
        let update = PrepareNotesRequestContextUpdater::builder().build();

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
    const fn should_implement_value_traits_for_prepare_notes_request_context() {
        implements_value_traits::<PrepareNotesRequestContext<FakeWriteRepository>>();
    }
}
