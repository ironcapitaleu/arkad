//! # Download Daily Index Input
//!
//! Provides the [`DownloadDailyIndexInput`] fed into the
//! [`DownloadDailyIndex`](crate::implementations::states::extract::download_daily_index::DownloadDailyIndex)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::request::implementations::sec_request::{DailyIndex, SecRequest};
use crate::traits::state_machine::state::StateData;

/// Input data for the [`DownloadDailyIndex`](super::super::DownloadDailyIndex) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct DownloadDailyIndexInput {
    /// The request for the daily index.
    pub request: SecRequest<DailyIndex>,
}

impl DownloadDailyIndexInput {
    /// Creates a new [`DownloadDailyIndexInput`] from a prepared request.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::download_daily_index::DownloadDailyIndexInput;
    /// use sec::shared::index_date::IndexDate;
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    ///
    /// let date = IndexDate::new("2026-10-27").expect("A hardcoded past date should always parse");
    /// let request = SecRequest::builder().daily_index().date(date).build();
    /// let input = DownloadDailyIndexInput::new(request.clone());
    ///
    /// let expected_result = request;
    ///
    /// let result = input.request;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(request: SecRequest<DailyIndex>) -> Self {
        Self { request }
    }
}

impl StateData for DownloadDailyIndexInput {
    /// Replaces the request if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(request) = updates.request {
            self.request = request;
        }
        Ok(())
    }
}

impl SMStateData for DownloadDailyIndexInput {
    type UpdateType = DownloadDailyIndexInputUpdater;

    fn state(&self) -> &Self {
        self
    }

    /// Delegates to the SEC [`StateData::update_state`] implementation.
    ///
    /// # Panics
    ///
    /// Panics if the fallible SEC update returns an error.
    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Err(e) = <Self as StateData>::update_state(self, updates) {
            panic!("StateData::update_state failed: {e}")
        }
    }
}

impl fmt::Display for DownloadDailyIndexInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tURL: {}", self.request.inner.url())
    }
}

/// Updater for [`DownloadDailyIndexInput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug)]
pub struct DownloadDailyIndexInputUpdater {
    /// The new request.
    pub request: Option<SecRequest<DailyIndex>>,
}

impl DownloadDailyIndexInputUpdater {
    /// Returns a [`DownloadDailyIndexInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> DownloadDailyIndexInputUpdaterBuilder {
        DownloadDailyIndexInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`DownloadDailyIndexInputUpdater`].
#[derive(Debug, Default)]
pub struct DownloadDailyIndexInputUpdaterBuilder {
    request: Option<SecRequest<DailyIndex>>,
}

impl DownloadDailyIndexInputUpdaterBuilder {
    /// Creates a [`DownloadDailyIndexInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self { request: None }
    }

    /// Sets the request.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn request(mut self, request: SecRequest<DailyIndex>) -> Self {
        self.request = Some(request);
        self
    }

    /// Builds the [`DownloadDailyIndexInputUpdater`].
    #[must_use]
    pub fn build(self) -> DownloadDailyIndexInputUpdater {
        DownloadDailyIndexInputUpdater {
            request: self.request,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use state_maschine::prelude::StateData as SMStateData;

    use super::*;
    use crate::shared::index_date::IndexDate;

    fn daily_index_request(date: &str) -> SecRequest<DailyIndex> {
        let date = IndexDate::new(date)
            .expect("Given a hardcoded past date, validation should always succeed");
        SecRequest::builder().daily_index().date(date).build()
    }

    #[test]
    fn should_update_request_when_update_contains_request() {
        let mut input = DownloadDailyIndexInput::new(daily_index_request("2026-10-27"));
        let update = DownloadDailyIndexInputUpdater::builder()
            .request(daily_index_request("2026-10-28"))
            .build();

        let expected_result = &DownloadDailyIndexInput::new(daily_index_request("2026-10-28"));

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_input_unchanged_when_update_is_empty() {
        let mut input = DownloadDailyIndexInput::new(daily_index_request("2026-10-27"));
        let update = DownloadDailyIndexInputUpdater::builder().build();

        let expected_result = &DownloadDailyIndexInput::new(daily_index_request("2026-10-27"));

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_download_daily_index_input() {
        implements_value_traits::<DownloadDailyIndexInput>();
    }
}
