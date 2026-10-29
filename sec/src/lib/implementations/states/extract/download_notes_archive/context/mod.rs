//! # Download Notes Archive Context
//!
//! Provides the [`DownloadNotesArchiveContext`] carried alongside the
//! [`DownloadNotesArchive`](crate::implementations::states::extract::download_notes_archive::DownloadNotesArchive)
//! state, together with its updater and builder.

use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;

use serde::Serialize;
use state_maschine::prelude::Context as SMContext;
use uuid::Uuid;

use crate::implementations::states::extract::notes_repository::NotesRepository;
use crate::shared::http_client::InnerClient;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::user_agent::UserAgent;
use crate::traits::state_machine::state::Context;

/// An SEC client that can download a Notes file and fits into a state context.
///
/// Its inner client sends `reqwest` requests.
pub trait NotesFileClient:
    SecClientTrait<
        Inner: InnerClient<
            Request = reqwest::Request,
            Response = reqwest::Response,
            Error = reqwest::Error,
        >,
    > + Debug
    + Clone
    + Eq
    + Ord
    + Hash
    + Send
    + Sync
    + Unpin
    + Serialize
{
    /// Returns the [`UserAgent`] that every request of the client carries.
    fn user_agent(&self) -> &UserAgent;
}

impl NotesFileClient for SecClient {
    fn user_agent(&self) -> &UserAgent {
        Self::user_agent(self)
    }
}

/// Context for the [`DownloadNotesArchive`](super::DownloadNotesArchive) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct DownloadNotesArchiveContext<R, C = SecClient> {
    /// The period of the Notes file to download.
    pub notes_period: NotesPeriod,
    /// The ID of the run.
    pub execution_id: Uuid,
    /// The storage repository.
    pub repository: R,
    /// The SEC client that downloads the Notes file.
    pub sec_client: C,
    /// The maximum number of retries of the state.
    pub max_retries: u32,
}

impl<R, C> DownloadNotesArchiveContext<R, C> {
    /// Creates a new [`DownloadNotesArchiveContext`] with a retry budget of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::download_notes_archive::DownloadNotesArchiveContext;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::notes_period::NotesPeriod;
    /// use uuid::Uuid;
    ///
    /// let period = NotesPeriod::new("2024q1")
    ///     .expect("Given a hardcoded published quarter, parsing should always succeed");
    /// let context = DownloadNotesArchiveContext::new(period, Uuid::nil(), (), SecClient::default());
    ///
    /// let expected_result = 0;
    ///
    /// let result = context.max_retries;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(
        notes_period: NotesPeriod,
        execution_id: Uuid,
        repository: R,
        sec_client: C,
    ) -> Self {
        Self {
            notes_period,
            execution_id,
            repository,
            sec_client,
            max_retries: 0,
        }
    }
}

impl<R: NotesRepository, C: NotesFileClient> Context for DownloadNotesArchiveContext<R, C> {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl<R: NotesRepository, C: NotesFileClient> SMContext for DownloadNotesArchiveContext<R, C> {
    type UpdateType = DownloadNotesArchiveContextUpdater<R, C>;

    fn context(&self) -> &Self {
        self
    }

    /// Replaces each field that the updater sets.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(notes_period) = updates.notes_period {
            self.notes_period = notes_period;
        }
        if let Some(execution_id) = updates.execution_id {
            self.execution_id = execution_id;
        }
        if let Some(repository) = updates.repository {
            self.repository = repository;
        }
        if let Some(sec_client) = updates.sec_client {
            self.sec_client = sec_client;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl<R, C> fmt::Display for DownloadNotesArchiveContext<R, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tNotes Period: {}\n\tExecution ID: {}",
            self.notes_period, self.execution_id
        )
    }
}

/// Updater for [`DownloadNotesArchiveContext`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct DownloadNotesArchiveContextUpdater<R, C> {
    /// The new Notes period.
    pub notes_period: Option<NotesPeriod>,
    /// The new `execution_id`.
    pub execution_id: Option<Uuid>,
    /// The new storage repository.
    pub repository: Option<R>,
    /// The new SEC client.
    pub sec_client: Option<C>,
    /// The new retry budget.
    pub max_retries: Option<u32>,
}

impl<R, C> DownloadNotesArchiveContextUpdater<R, C> {
    /// Returns a [`DownloadNotesArchiveContextUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> DownloadNotesArchiveContextUpdaterBuilder<R, C> {
        DownloadNotesArchiveContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`DownloadNotesArchiveContextUpdater`].
#[derive(Debug)]
pub struct DownloadNotesArchiveContextUpdaterBuilder<R, C> {
    notes_period: Option<NotesPeriod>,
    execution_id: Option<Uuid>,
    repository: Option<R>,
    sec_client: Option<C>,
    max_retries: Option<u32>,
}

impl<R, C> DownloadNotesArchiveContextUpdaterBuilder<R, C> {
    /// Creates a [`DownloadNotesArchiveContextUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            notes_period: None,
            execution_id: None,
            repository: None,
            sec_client: None,
            max_retries: None,
        }
    }

    /// Sets the Notes period.
    #[must_use]
    pub const fn notes_period(mut self, notes_period: NotesPeriod) -> Self {
        self.notes_period = Some(notes_period);
        self
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

    /// Builds the [`DownloadNotesArchiveContextUpdater`].
    #[must_use]
    pub fn build(self) -> DownloadNotesArchiveContextUpdater<R, C> {
        DownloadNotesArchiveContextUpdater {
            notes_period: self.notes_period,
            execution_id: self.execution_id,
            repository: self.repository,
            sec_client: self.sec_client,
            max_retries: self.max_retries,
        }
    }
}

impl<R, C> Default for DownloadNotesArchiveContextUpdaterBuilder<R, C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use storage::WriteError;

    use super::*;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingReadySecClient;
    use crate::tests::fixtures::sample_write_repository::FakeNotesRepository;

    type TestContext =
        DownloadNotesArchiveContext<FakeNotesRepository, AlwaysRespondingReadySecClient>;

    fn sec_client(status: u16) -> AlwaysRespondingReadySecClient {
        AlwaysRespondingReadySecClient::new(status, Vec::new(), Vec::new())
    }

    fn sample_period(raw_period: &str) -> NotesPeriod {
        NotesPeriod::new(raw_period)
            .expect("Given a hardcoded published period, parsing should always succeed")
    }

    fn test_context() -> TestContext {
        DownloadNotesArchiveContext::new(
            sample_period("2024q1"),
            Uuid::nil(),
            FakeNotesRepository::new(),
            sec_client(200),
        )
    }

    #[test]
    fn should_update_notes_period_when_update_contains_notes_period() {
        let mut context = test_context();
        let update = DownloadNotesArchiveContextUpdater::builder()
            .notes_period(sample_period("2025_10"))
            .build();

        let expected_result = sample_period("2025_10");

        context.update_context(update);
        let result = context.notes_period;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_notes_period_and_execution_id_for_download_notes_archive_context() {
        let context = test_context();

        let expected_result =
            "\tNotes Period: 2024q1\n\tExecution ID: 00000000-0000-0000-0000-000000000000";

        let result = context.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_execution_id_when_update_contains_execution_id() {
        let mut context = test_context();
        let update = DownloadNotesArchiveContextUpdater::builder()
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
        let rejecting = FakeNotesRepository::rejecting(WriteError::conflicting_write("Sample"));
        let update = DownloadNotesArchiveContextUpdater::builder()
            .repository(rejecting.clone())
            .build();

        let expected_result = rejecting;

        context.update_context(update);
        let result = context.repository;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_sec_client_when_update_contains_sec_client() {
        let mut context = test_context();
        let update = DownloadNotesArchiveContextUpdater::builder()
            .sec_client(sec_client(404))
            .build();

        let expected_result = sec_client(404);

        context.update_context(update);
        let result = context.sec_client;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_update_contains_max_retries() {
        let mut context = test_context();
        let update = DownloadNotesArchiveContextUpdater::builder()
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
        let update = DownloadNotesArchiveContextUpdater::builder().build();

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
    const fn should_implement_value_traits_for_download_notes_archive_context() {
        implements_value_traits::<DownloadNotesArchiveContext<FakeNotesRepository>>();
    }
}
