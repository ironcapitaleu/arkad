//! # Download Daily Index Context
//!
//! Provides the [`DownloadDailyIndexContext`] carried alongside the
//! [`DownloadDailyIndex`](crate::implementations::states::extract::download_daily_index::DownloadDailyIndex)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::Context as SMContext;

use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::index_date::IndexDate;
use crate::traits::state_machine::state::Context;

/// Context for the [`DownloadDailyIndex`](super::DownloadDailyIndex) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct DownloadDailyIndexContext<C = SecClient> {
    /// The date of the daily index to download.
    pub index_date: IndexDate,
    /// The SEC client that downloads the daily index and the quarter listings.
    pub sec_client: C,
    /// The maximum number of retries of the state.
    pub max_retries: u32,
}

impl<C> DownloadDailyIndexContext<C> {
    /// Creates a new [`DownloadDailyIndexContext`] with a retry budget of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::download_daily_index::DownloadDailyIndexContext;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::index_date::IndexDate;
    ///
    /// let date = IndexDate::new("2026-10-27").expect("A hardcoded past date should always parse");
    /// let context = DownloadDailyIndexContext::new(date, SecClient::default());
    ///
    /// let expected_result = 0;
    ///
    /// let result = context.max_retries;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(index_date: IndexDate, sec_client: C) -> Self {
        Self {
            index_date,
            sec_client,
            max_retries: 0,
        }
    }
}

impl<C: SecClientTrait> Context for DownloadDailyIndexContext<C> {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl<C: SecClientTrait> SMContext for DownloadDailyIndexContext<C> {
    type UpdateType = DownloadDailyIndexContextUpdater<C>;

    fn context(&self) -> &Self {
        self
    }

    /// Replaces each field that the updater sets.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(index_date) = updates.index_date {
            self.index_date = index_date;
        }
        if let Some(sec_client) = updates.sec_client {
            self.sec_client = sec_client;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl<C> fmt::Display for DownloadDailyIndexContext<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tIndex Date: {}", self.index_date)
    }
}

/// Updater for [`DownloadDailyIndexContext`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct DownloadDailyIndexContextUpdater<C> {
    /// The new index date.
    pub index_date: Option<IndexDate>,
    /// The new SEC client.
    pub sec_client: Option<C>,
    /// The new retry budget.
    pub max_retries: Option<u32>,
}

impl<C> DownloadDailyIndexContextUpdater<C> {
    /// Returns a [`DownloadDailyIndexContextUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> DownloadDailyIndexContextUpdaterBuilder<C> {
        DownloadDailyIndexContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`DownloadDailyIndexContextUpdater`].
#[derive(Debug)]
pub struct DownloadDailyIndexContextUpdaterBuilder<C> {
    index_date: Option<IndexDate>,
    sec_client: Option<C>,
    max_retries: Option<u32>,
}

impl<C> DownloadDailyIndexContextUpdaterBuilder<C> {
    /// Creates a [`DownloadDailyIndexContextUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            index_date: None,
            sec_client: None,
            max_retries: None,
        }
    }

    /// Sets the index date.
    #[must_use]
    pub const fn index_date(mut self, index_date: IndexDate) -> Self {
        self.index_date = Some(index_date);
        self
    }

    /// Sets the SEC client.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn sec_client(mut self, sec_client: C) -> Self {
        self.sec_client = Some(sec_client);
        self
    }

    /// Sets the retry budget.
    #[must_use]
    pub const fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Builds the [`DownloadDailyIndexContextUpdater`].
    #[must_use]
    pub fn build(self) -> DownloadDailyIndexContextUpdater<C> {
        DownloadDailyIndexContextUpdater {
            index_date: self.index_date,
            sec_client: self.sec_client,
            max_retries: self.max_retries,
        }
    }
}

impl<C> Default for DownloadDailyIndexContextUpdaterBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;

    type TestContext = DownloadDailyIndexContext<RoutingSecClient>;

    fn index_date(date: &str) -> IndexDate {
        IndexDate::new(date).expect("Given a hardcoded past date, validation should always succeed")
    }

    fn test_context() -> TestContext {
        DownloadDailyIndexContext::new(index_date("2026-10-27"), RoutingSecClient::new())
    }

    #[test]
    fn should_update_index_date_when_update_contains_index_date() {
        let mut context = test_context();
        let update = DownloadDailyIndexContextUpdater::builder()
            .index_date(index_date("2026-10-28"))
            .build();

        let expected_result = index_date("2026-10-28");

        context.update_context(update);
        let result = context.index_date;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_sec_client_when_update_contains_sec_client() {
        let mut context = test_context();
        let other = RoutingSecClient::new().route("https://example.com/", 200, "text/plain", b"");
        let update = DownloadDailyIndexContextUpdater::builder()
            .sec_client(other.clone())
            .build();

        let expected_result = other;

        context.update_context(update);
        let result = context.sec_client;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_update_contains_max_retries() {
        let mut context = test_context();
        let update = DownloadDailyIndexContextUpdater::builder()
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
        let update = DownloadDailyIndexContextUpdater::builder().build();

        let expected_result = &test_context();

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_index_date_for_download_daily_index_context() {
        let context = test_context();

        let expected_result = "\tIndex Date: 2026-10-27";

        let result = context.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_download_daily_index_context() {
        implements_value_traits::<DownloadDailyIndexContext>();
    }
}
