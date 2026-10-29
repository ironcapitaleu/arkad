//! # Store Notes Archive Output
//!
//! Provides the [`StoreNotesArchiveOutput`] produced by the
//! [`StoreNotesArchive`](crate::implementations::states::extract::store_notes_archive::StoreNotesArchive)
//! state, together with its updater and builder.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::notes_archive_ref::NotesArchiveRef;
use crate::traits::state_machine::state::StateData;

/// Output data of the [`StoreNotesArchive`](super::super::StoreNotesArchive) state.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct StoreNotesArchiveOutput {
    /// The reference to the stored Notes file.
    pub archive: NotesArchiveRef,
}

impl StoreNotesArchiveOutput {
    /// Creates a new [`StoreNotesArchiveOutput`] from the reference to a stored Notes file.
    #[must_use]
    pub const fn new(archive: NotesArchiveRef) -> Self {
        Self { archive }
    }
}

impl StateData for StoreNotesArchiveOutput {
    /// Replaces the archive reference if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(archive) = updates.archive {
            self.archive = archive;
        }
        Ok(())
    }
}

impl SMStateData for StoreNotesArchiveOutput {
    type UpdateType = StoreNotesArchiveOutputUpdater;

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

impl fmt::Display for StoreNotesArchiveOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\t{}", self.archive)
    }
}

/// Updater for [`StoreNotesArchiveOutput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug)]
pub struct StoreNotesArchiveOutputUpdater {
    /// The new archive reference.
    pub archive: Option<NotesArchiveRef>,
}

impl StoreNotesArchiveOutputUpdater {
    /// Returns a [`StoreNotesArchiveOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> StoreNotesArchiveOutputUpdaterBuilder {
        StoreNotesArchiveOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreNotesArchiveOutputUpdater`].
#[derive(Debug, Default)]
pub struct StoreNotesArchiveOutputUpdaterBuilder {
    archive: Option<NotesArchiveRef>,
}

impl StoreNotesArchiveOutputUpdaterBuilder {
    /// Creates a [`StoreNotesArchiveOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self { archive: None }
    }

    /// Sets the archive reference.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn archive(mut self, archive: NotesArchiveRef) -> Self {
        self.archive = Some(archive);
        self
    }

    /// Builds the [`StoreNotesArchiveOutputUpdater`].
    #[must_use]
    pub fn build(self) -> StoreNotesArchiveOutputUpdater {
        StoreNotesArchiveOutputUpdater {
            archive: self.archive,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use state_maschine::prelude::StateData as SMStateData;
    use storage::Sha256Digest;
    use storage::implementations::repository::filesystem::DocumentKey;

    use super::*;
    use crate::shared::notes_period::NotesPeriod;

    fn archive(period: &str, body: &[u8]) -> NotesArchiveRef {
        let period = NotesPeriod::new(period)
            .expect("Given a hardcoded published period, parsing should always succeed");
        let key = DocumentKey::new(format!("sec/notes/{}", period.file_name()))
            .expect("Given an SEC file name, the key should always build");
        NotesArchiveRef::new(period, key, body.len() as u64, Sha256Digest::of(body))
    }

    #[test]
    fn should_update_archive_when_update_contains_archive() {
        let mut output = StoreNotesArchiveOutput::new(archive("2024q1", b"PK"));
        let update = StoreNotesArchiveOutputUpdater::builder()
            .archive(archive("2025_10", b"PK!"))
            .build();

        let expected_result = &StoreNotesArchiveOutput::new(archive("2025_10", b"PK!"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_unchanged_when_update_is_empty() {
        let mut output = StoreNotesArchiveOutput::new(archive("2024q1", b"PK"));
        let update = StoreNotesArchiveOutputUpdater::builder().build();

        let expected_result = &StoreNotesArchiveOutput::new(archive("2024q1", b"PK"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_notes_archive_output() {
        implements_value_traits::<StoreNotesArchiveOutput>();
    }
}
