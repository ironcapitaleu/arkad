//! # Download Notes Archive Input
//!
//! Provides the [`DownloadNotesArchiveInput`] fed into the
//! [`DownloadNotesArchive`](crate::implementations::states::extract::download_notes_archive::DownloadNotesArchive)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::request::implementations::sec_request::{NotesFile, SecRequest};
use crate::traits::state_machine::state::StateData;

/// Input data for the [`DownloadNotesArchive`](super::super::DownloadNotesArchive) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct DownloadNotesArchiveInput {
    /// The request for the Notes file.
    pub request: SecRequest<NotesFile>,
}

impl DownloadNotesArchiveInput {
    /// Creates a new [`DownloadNotesArchiveInput`] from a prepared request.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::download_notes_archive::DownloadNotesArchiveInput;
    /// use sec::shared::notes_period::NotesPeriod;
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    ///
    /// let period = NotesPeriod::new("2024q1")
    ///     .expect("Given a hardcoded published quarter, parsing should always succeed");
    /// let request = SecRequest::builder().notes_file().period(period).build();
    /// let input = DownloadNotesArchiveInput::new(request.clone());
    ///
    /// let expected_result = request;
    ///
    /// let result = input.request;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(request: SecRequest<NotesFile>) -> Self {
        Self { request }
    }
}

impl StateData for DownloadNotesArchiveInput {
    /// Replaces the request if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(request) = updates.request {
            self.request = request;
        }
        Ok(())
    }
}

impl SMStateData for DownloadNotesArchiveInput {
    type UpdateType = DownloadNotesArchiveInputUpdater;

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

impl fmt::Display for DownloadNotesArchiveInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tURL: {}", self.request.inner.url())
    }
}

/// Updater for [`DownloadNotesArchiveInput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug)]
pub struct DownloadNotesArchiveInputUpdater {
    /// The new request.
    pub request: Option<SecRequest<NotesFile>>,
}

impl DownloadNotesArchiveInputUpdater {
    /// Returns a [`DownloadNotesArchiveInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> DownloadNotesArchiveInputUpdaterBuilder {
        DownloadNotesArchiveInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`DownloadNotesArchiveInputUpdater`].
#[derive(Debug, Default)]
pub struct DownloadNotesArchiveInputUpdaterBuilder {
    request: Option<SecRequest<NotesFile>>,
}

impl DownloadNotesArchiveInputUpdaterBuilder {
    /// Creates a [`DownloadNotesArchiveInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self { request: None }
    }

    /// Sets the request.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn request(mut self, request: SecRequest<NotesFile>) -> Self {
        self.request = Some(request);
        self
    }

    /// Builds the [`DownloadNotesArchiveInputUpdater`].
    #[must_use]
    pub fn build(self) -> DownloadNotesArchiveInputUpdater {
        DownloadNotesArchiveInputUpdater {
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

    fn notes_request(period: &str) -> SecRequest<NotesFile> {
        let period = NotesPeriod::new(period)
            .expect("Given a hardcoded published period, parsing should always succeed");
        SecRequest::builder().notes_file().period(period).build()
    }

    #[test]
    fn should_update_request_when_update_contains_request() {
        let mut input = DownloadNotesArchiveInput::new(notes_request("2024q1"));
        let update = DownloadNotesArchiveInputUpdater::builder()
            .request(notes_request("2025_10"))
            .build();

        let expected_result = &DownloadNotesArchiveInput::new(notes_request("2025_10"));

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_input_unchanged_when_update_is_empty() {
        let mut input = DownloadNotesArchiveInput::new(notes_request("2024q1"));
        let update = DownloadNotesArchiveInputUpdater::builder().build();

        let expected_result = &DownloadNotesArchiveInput::new(notes_request("2024q1"));

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_download_notes_archive_input() {
        implements_value_traits::<DownloadNotesArchiveInput>();
    }
}
