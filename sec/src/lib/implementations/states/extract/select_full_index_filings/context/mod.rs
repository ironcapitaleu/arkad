//! # Select Full Index Filings Context
//!
//! Provides the [`SelectFullIndexFilingsContext`] carried alongside the
//! [`SelectFullIndexFilings`](crate::implementations::states::extract::select_full_index_filings::SelectFullIndexFilings)
//! state, together with its updater and builder.

use std::fmt;

use chrono::NaiveDate;
use serde::Serialize;
use state_maschine::prelude::Context as SMContext;
use uuid::Uuid;

use crate::traits::state_machine::state::Context;

/// Context for the [`SelectFullIndexFilings`](super::SelectFullIndexFilings) state.
///
/// The state keeps the rows with a date filed from `first_filed_date` to `last_filed_date`, both
/// included.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct SelectFullIndexFilingsContext {
    /// The ID of the run.
    pub execution_id: Uuid,
    /// The first date filed that the state keeps.
    pub first_filed_date: NaiveDate,
    /// The last date filed that the state keeps.
    pub last_filed_date: NaiveDate,
    /// The maximum number of retries of the state.
    pub max_retries: u32,
}

impl SelectFullIndexFilingsContext {
    /// Creates a new [`SelectFullIndexFilingsContext`] with a retry budget of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use sec::implementations::states::extract::select_full_index_filings::SelectFullIndexFilingsContext;
    /// use uuid::Uuid;
    ///
    /// let first = NaiveDate::from_ymd_opt(2026, 9, 1)
    ///     .expect("Given a hardcoded calendar date, the date should always build");
    /// let last = NaiveDate::from_ymd_opt(2026, 9, 30)
    ///     .expect("Given a hardcoded calendar date, the date should always build");
    /// let context = SelectFullIndexFilingsContext::new(Uuid::nil(), first, last);
    ///
    /// let expected_result = 0;
    ///
    /// let result = context.max_retries;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(
        execution_id: Uuid,
        first_filed_date: NaiveDate,
        last_filed_date: NaiveDate,
    ) -> Self {
        Self {
            execution_id,
            first_filed_date,
            last_filed_date,
            max_retries: 0,
        }
    }
}

impl Context for SelectFullIndexFilingsContext {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContext for SelectFullIndexFilingsContext {
    type UpdateType = SelectFullIndexFilingsContextUpdater;

    fn context(&self) -> &Self {
        self
    }

    /// Replaces each field that the updater sets.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(first_filed_date) = updates.first_filed_date {
            self.first_filed_date = first_filed_date;
        }
        if let Some(last_filed_date) = updates.last_filed_date {
            self.last_filed_date = last_filed_date;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl fmt::Display for SelectFullIndexFilingsContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tExecution ID: {}, Dates filed: {} to {}",
            self.execution_id, self.first_filed_date, self.last_filed_date
        )
    }
}

/// Updater for [`SelectFullIndexFilingsContext`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SelectFullIndexFilingsContextUpdater {
    /// The new first date filed.
    pub first_filed_date: Option<NaiveDate>,
    /// The new last date filed.
    pub last_filed_date: Option<NaiveDate>,
    /// The new retry budget.
    pub max_retries: Option<u32>,
}

impl SelectFullIndexFilingsContextUpdater {
    /// Returns a [`SelectFullIndexFilingsContextUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> SelectFullIndexFilingsContextUpdaterBuilder {
        SelectFullIndexFilingsContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`SelectFullIndexFilingsContextUpdater`].
#[derive(Debug, Default)]
pub struct SelectFullIndexFilingsContextUpdaterBuilder {
    first_filed_date: Option<NaiveDate>,
    last_filed_date: Option<NaiveDate>,
    max_retries: Option<u32>,
}

impl SelectFullIndexFilingsContextUpdaterBuilder {
    /// Creates a [`SelectFullIndexFilingsContextUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            first_filed_date: None,
            last_filed_date: None,
            max_retries: None,
        }
    }

    /// Sets the first date filed.
    #[must_use]
    pub const fn first_filed_date(mut self, first_filed_date: NaiveDate) -> Self {
        self.first_filed_date = Some(first_filed_date);
        self
    }

    /// Sets the last date filed.
    #[must_use]
    pub const fn last_filed_date(mut self, last_filed_date: NaiveDate) -> Self {
        self.last_filed_date = Some(last_filed_date);
        self
    }

    /// Sets the retry budget.
    #[must_use]
    pub const fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Builds the [`SelectFullIndexFilingsContextUpdater`].
    #[must_use]
    pub const fn build(self) -> SelectFullIndexFilingsContextUpdater {
        SelectFullIndexFilingsContextUpdater {
            first_filed_date: self.first_filed_date,
            last_filed_date: self.last_filed_date,
            max_retries: self.max_retries,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, the date should always build")
    }

    #[test]
    fn should_replace_the_dates_filed_when_update_contains_both() {
        let mut context =
            SelectFullIndexFilingsContext::new(Uuid::nil(), date(2026, 9, 1), date(2026, 9, 30));
        let update = SelectFullIndexFilingsContextUpdater::builder()
            .first_filed_date(date(2026, 10, 1))
            .last_filed_date(date(2026, 10, 31))
            .build();

        let expected_result =
            &SelectFullIndexFilingsContext::new(Uuid::nil(), date(2026, 10, 1), date(2026, 10, 31));

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_the_context_unchanged_when_update_is_empty() {
        let mut context =
            SelectFullIndexFilingsContext::new(Uuid::nil(), date(2026, 9, 1), date(2026, 9, 30));
        let update = SelectFullIndexFilingsContextUpdater::builder().build();

        let expected_result =
            &SelectFullIndexFilingsContext::new(Uuid::nil(), date(2026, 9, 1), date(2026, 9, 30));

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_replace_the_retry_budget_when_update_contains_it() {
        let mut context =
            SelectFullIndexFilingsContext::new(Uuid::nil(), date(2026, 9, 1), date(2026, 9, 30));
        let update = SelectFullIndexFilingsContextUpdater::builder()
            .max_retries(3)
            .build();

        let expected_result = 3;

        context.update_context(update);
        let result = context.max_retries;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_dates_filed_for_select_full_index_filings_context() {
        let context =
            SelectFullIndexFilingsContext::new(Uuid::nil(), date(2026, 9, 24), date(2026, 11, 12));

        let expected_result = "\tExecution ID: 00000000-0000-0000-0000-000000000000, Dates filed: 2026-09-24 to 2026-11-12";

        let result = context.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_select_full_index_filings_context() {
        implements_value_traits::<SelectFullIndexFilingsContext>();
    }
}
