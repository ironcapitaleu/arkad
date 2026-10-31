//! # Select Filings Context
//!
//! Provides the [`SelectFilingsContext`] carried alongside the
//! [`SelectFilings`](crate::implementations::states::extract::select_filings::SelectFilings)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::Context as SMContext;
use uuid::Uuid;

use crate::traits::state_machine::state::Context;

/// Context for the [`SelectFilings`](super::SelectFilings) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct SelectFilingsContext {
    /// The ID of the run.
    pub execution_id: Uuid,
    /// The maximum number of retries of the state.
    pub max_retries: u32,
}

impl SelectFilingsContext {
    /// Creates a new [`SelectFilingsContext`] with a retry budget of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::select_filings::SelectFilingsContext;
    /// use uuid::Uuid;
    ///
    /// let context = SelectFilingsContext::new(Uuid::nil());
    ///
    /// let expected_result = 0;
    ///
    /// let result = context.max_retries;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(execution_id: Uuid) -> Self {
        Self {
            execution_id,
            max_retries: 0,
        }
    }
}

impl Context for SelectFilingsContext {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContext for SelectFilingsContext {
    type UpdateType = SelectFilingsContextUpdater;

    fn context(&self) -> &Self {
        self
    }

    /// Replaces each field that the updater sets.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(execution_id) = updates.execution_id {
            self.execution_id = execution_id;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl fmt::Display for SelectFilingsContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tExecution ID: {}", self.execution_id)
    }
}

/// Updater for [`SelectFilingsContext`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SelectFilingsContextUpdater {
    /// The new `execution_id`.
    pub execution_id: Option<Uuid>,
    /// The new retry budget.
    pub max_retries: Option<u32>,
}

impl SelectFilingsContextUpdater {
    /// Returns a [`SelectFilingsContextUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> SelectFilingsContextUpdaterBuilder {
        SelectFilingsContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`SelectFilingsContextUpdater`].
#[derive(Debug, Default)]
pub struct SelectFilingsContextUpdaterBuilder {
    execution_id: Option<Uuid>,
    max_retries: Option<u32>,
}

impl SelectFilingsContextUpdaterBuilder {
    /// Creates a [`SelectFilingsContextUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            execution_id: None,
            max_retries: None,
        }
    }

    /// Sets the `execution_id`.
    #[must_use]
    pub const fn execution_id(mut self, execution_id: Uuid) -> Self {
        self.execution_id = Some(execution_id);
        self
    }

    /// Sets the retry budget.
    #[must_use]
    pub const fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Builds the [`SelectFilingsContextUpdater`].
    #[must_use]
    pub const fn build(self) -> SelectFilingsContextUpdater {
        SelectFilingsContextUpdater {
            execution_id: self.execution_id,
            max_retries: self.max_retries,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;

    fn test_context() -> SelectFilingsContext {
        SelectFilingsContext::new(Uuid::nil())
    }

    #[test]
    fn should_update_execution_id_when_update_contains_execution_id() {
        let mut context = test_context();
        let update = SelectFilingsContextUpdater::builder()
            .execution_id(Uuid::from_u128(7))
            .build();

        let expected_result = Uuid::from_u128(7);

        context.update_context(update);
        let result = context.execution_id;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_update_contains_max_retries() {
        let mut context = test_context();
        let update = SelectFilingsContextUpdater::builder()
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
        let update = SelectFilingsContextUpdater::builder().build();

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
    const fn should_implement_value_traits_for_select_filings_context() {
        implements_value_traits::<SelectFilingsContext>();
    }
}
