//! # Prepare Notes Request Output
//!
//! Provides the [`PrepareNotesRequestOutput`] produced by the
//! [`PrepareNotesRequest`](crate::implementations::states::extract::prepare_notes_request::PrepareNotesRequest)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::traits::state_machine::state::StateData;

/// Output data of the [`PrepareNotesRequest`](super::super::PrepareNotesRequest) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct PrepareNotesRequestOutput {
    /// The request for the Notes file.
    pub request: SecRequest,
}

impl PrepareNotesRequestOutput {
    /// Creates a new [`PrepareNotesRequestOutput`] from a prepared request.
    #[must_use]
    pub const fn new(request: SecRequest) -> Self {
        Self { request }
    }
}

impl StateData for PrepareNotesRequestOutput {
    /// Replaces the request if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(request) = updates.request {
            self.request = request;
        }
        Ok(())
    }
}

impl SMStateData for PrepareNotesRequestOutput {
    type UpdateType = PrepareNotesRequestOutputUpdater;

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

impl fmt::Display for PrepareNotesRequestOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tURL: {}", self.request.inner.url())
    }
}

/// Updater for [`PrepareNotesRequestOutput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug)]
pub struct PrepareNotesRequestOutputUpdater {
    /// The new request.
    pub request: Option<SecRequest>,
}

impl PrepareNotesRequestOutputUpdater {
    /// Returns a [`PrepareNotesRequestOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> PrepareNotesRequestOutputUpdaterBuilder {
        PrepareNotesRequestOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`PrepareNotesRequestOutputUpdater`].
#[derive(Debug, Default)]
pub struct PrepareNotesRequestOutputUpdaterBuilder {
    request: Option<SecRequest>,
}

impl PrepareNotesRequestOutputUpdaterBuilder {
    /// Creates a [`PrepareNotesRequestOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self { request: None }
    }

    /// Sets the request.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn request(mut self, request: SecRequest) -> Self {
        self.request = Some(request);
        self
    }

    /// Builds the [`PrepareNotesRequestOutputUpdater`].
    #[must_use]
    pub fn build(self) -> PrepareNotesRequestOutputUpdater {
        PrepareNotesRequestOutputUpdater {
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
    use crate::shared::notes_period::NotesPeriod;

    fn notes_request(period: &str) -> SecRequest {
        let period = NotesPeriod::new(period)
            .expect("Given a hardcoded published period, parsing should always succeed");
        SecRequest::builder().notes_file().period(period).build()
    }

    #[test]
    fn should_update_request_when_update_contains_request() {
        let mut output = PrepareNotesRequestOutput::new(notes_request("2024q1"));
        let update = PrepareNotesRequestOutputUpdater::builder()
            .request(notes_request("2025_10"))
            .build();

        let expected_result = &PrepareNotesRequestOutput::new(notes_request("2025_10"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_unchanged_when_update_is_empty() {
        let mut output = PrepareNotesRequestOutput::new(notes_request("2024q1"));
        let update = PrepareNotesRequestOutputUpdater::builder().build();

        let expected_result = &PrepareNotesRequestOutput::new(notes_request("2024q1"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_prepare_notes_request_output() {
        implements_value_traits::<PrepareNotesRequestOutput>();
    }
}
