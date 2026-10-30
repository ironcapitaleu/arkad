//! # Extraction
//!
//! Provides [`Extraction`], the one entry point that builds the extract super-states.
//!
//! The first call picks the ingestion mode. The second call picks the source and returns the
//! typestate builder of that source's super-state. Each [`build`] returns the concrete super-state.
//!
//! [`build`]: ExtractNotesSuperStateBuilder::build
//!
//! ## Usage
//!
//! ```no_run
//! use sec::implementations::extraction::Extraction;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use sec::shared::raw_document::RawDocument;
//! use sec::shared::raw_source::RawNotesArchive;
//! use storage::implementations::repository::filesystem::FilesystemRepository;
//!
//! let repository = FilesystemRepository::builder()
//!     .writer()
//!     .document_type::<RawDocument<RawNotesArchive>>()
//!     .root("data")
//!     .build();
//! let extract_state = Extraction::backfill()
//!     .notes()
//!     .period("2024q1")
//!     .sec_client(SecClient::default())
//!     .repository(repository)
//!     .build();
//! ```

use uuid::Uuid;

use crate::implementations::states::extract::notes::ExtractNotesSuperState;
use crate::implementations::states::extract::notes::builder::{
    ExtractNotesSuperStateBuilder, NoPeriod, NoRepository, NoSecClient,
};
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawNotesArchive;

/// Builds the extract super-states by ingestion mode and source.
///
/// Each mode offers only its own sources. A source that a mode does not offer does not compile:
///
/// ```compile_fail
/// use sec::implementations::extraction::Extraction;
///
/// let _builder = Extraction::incremental().notes();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Extraction;

impl Extraction {
    /// Starts a backfill run with a new random `execution_id`.
    #[must_use]
    pub fn backfill() -> Backfill {
        Backfill {
            execution_id: Uuid::new_v4(),
        }
    }
}

/// Builds the super-states of one backfill run.
///
/// Every super-state built from the same handle carries the handle's `execution_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Backfill {
    execution_id: Uuid,
}

impl Backfill {
    /// Replaces the `execution_id` of the run.
    #[must_use]
    pub const fn execution_id(self, execution_id: Uuid) -> Self {
        Self { execution_id }
    }

    /// Returns the `execution_id` of the run.
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.execution_id
    }

    /// Returns the builder of an [`ExtractNotesSuperState`] that holds the run's `execution_id`.
    ///
    /// The builder still needs a period, an SEC client, and a repository of type `R`.
    #[must_use]
    pub fn notes<R: RawRepository<RawNotesArchive>>(
        &self,
    ) -> ExtractNotesSuperStateBuilder<NoPeriod, NoSecClient, Uuid, NoRepository<R>> {
        ExtractNotesSuperState::builder().execution_id(self.execution_id)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use futures_util::StreamExt;
    use pretty_assertions::{assert_eq, assert_ne};
    use storage::implementations::repository::filesystem::FilesystemRepository;
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::states::extract::validate_notes_period::ValidateNotesPeriod;
    use crate::shared::raw_document::RawDocument;
    use crate::shared::raw_source::RawNotesArchive;
    use crate::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_write_repository::FakeNotesRepository;

    type NotesSuperState =
        ExtractNotesSuperState<ValidateNotesPeriod<FakeNotesRepository>, AlwaysRespondingSecClient>;

    fn sec_client() -> AlwaysRespondingSecClient {
        AlwaysRespondingSecClient::new(
            200,
            vec![("content-type", "application/zip".to_string())],
            SAMPLE_NOTES_ZIP.to_vec(),
        )
    }

    fn notes_super_state(backfill: &Backfill) -> NotesSuperState {
        backfill
            .notes()
            .period("2024q1")
            .sec_client(sec_client())
            .repository(FakeNotesRepository::new())
            .build()
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_extraction() {
        implements_auto_traits::<Extraction>();
    }

    #[test]
    const fn should_implement_auto_traits_for_backfill() {
        implements_auto_traits::<Backfill>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_extraction() {
        implements_value_traits::<Extraction>();
    }

    #[test]
    const fn should_implement_value_traits_for_backfill() {
        implements_value_traits::<Backfill>();
    }

    #[test]
    fn should_create_a_different_execution_id_when_starting_two_backfill_runs() {
        let expected_result = Extraction::backfill();

        let result = Extraction::backfill();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_give_every_super_state_the_same_execution_id_when_built_from_one_handle() {
        let backfill = Extraction::backfill();

        let expected_result = notes_super_state(&backfill);

        let result = notes_super_state(&backfill);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_super_state_with_the_given_execution_id_when_overriding_it() {
        let backfill = Extraction::backfill().execution_id(Uuid::from_u128(7));

        let expected_result = ExtractNotesSuperState::builder()
            .period("2024q1")
            .sec_client(sec_client())
            .execution_id(Uuid::from_u128(7))
            .repository(FakeNotesRepository::new())
            .build();

        let result = notes_super_state(&backfill);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_given_execution_id_when_reading_the_id_of_the_run() {
        let backfill = Extraction::backfill().execution_id(Uuid::from_u128(7));

        let expected_result = Uuid::from_u128(7);

        let result = backfill.id();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_backfill_as_the_ingestion_mode_when_running_a_notes_super_state() {
        let root = TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        );
        let repository = FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<RawNotesArchive>>()
            .root(root.path())
            .build();
        let super_state = Extraction::backfill()
            .notes()
            .period("2024q1")
            .sec_client(sec_client())
            .repository(repository)
            .build();

        let expected_result = serde_json::json!("backfill");

        let mut stream = super_state.into_run_stream();
        while let Some(item) = stream.next().await {
            item.expect(
                "Given a published period, the fake client, and a writable store, every stream item should be a success",
            );
        }
        let metadata = std::fs::read(root.path().join("sec/notes/2024q1_notes.zip.meta.json"))
            .expect("Given a stream that completed without an error, the metadata file should always exist");
        let metadata: serde_json::Value = serde_json::from_slice(&metadata)
            .expect("Given metadata written by the repository, it should always be valid JSON");
        let result = metadata["ingestion_mode"].clone();

        assert_eq!(result, expected_result);
    }
}
