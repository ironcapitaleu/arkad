//! # Extraction
//!
//! Provides [`Extraction`], the one entry point that builds the extract super-states.
//!
//! The first call picks the ingestion mode: [`Extraction::backfill`] or
//! [`Extraction::incremental`]. The second call picks the source and returns the typestate builder
//! of that source's super-state, with the run's `execution_id` and ingestion mode already set.
//! Each [`build`] returns the concrete super-state.
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
//!
//! An incremental run builds the machines of the new-filing path:
//!
//! ```no_run
//! use sec::implementations::extraction::Extraction;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use sec::shared::raw_document::RawDocument;
//! use sec::shared::raw_source::RawDailyIndex;
//! use storage::implementations::repository::filesystem::FilesystemRepository;
//!
//! let repository = FilesystemRepository::builder()
//!     .writer()
//!     .document_type::<RawDocument<RawDailyIndex>>()
//!     .root("data")
//!     .build();
//! let detection = Extraction::incremental()
//!     .daily_index()
//!     .date("2026-09-09")
//!     .sec_client(SecClient::default())
//!     .repository(repository)
//!     .build();
//! ```

use storage::implementations::repository::filesystem::IngestionMode;
use uuid::Uuid;

use crate::implementations::states::extract::daily_index::DetectFilingsSuperState;
use crate::implementations::states::extract::daily_index::builder::{
    DetectFilingsSuperStateBuilder, NoDate,
};
use crate::implementations::states::extract::filing::ExtractFilingSuperState;
use crate::implementations::states::extract::filing::builder::{
    ExtractFilingSuperStateBuilder, NoFilingRef, NoInstanceRepository, NoLabelsRepository,
};
use crate::implementations::states::extract::notes::ExtractNotesSuperState;
use crate::implementations::states::extract::notes::builder::{
    ExtractNotesSuperStateBuilder, NoPeriod,
};
use crate::implementations::states::extract::submissions::ExtractSubmissionsSuperState;
use crate::implementations::states::extract::submissions::builder::{
    ExtractSubmissionsSuperStateBuilder, NoCik,
};
use crate::implementations::states::extract::{daily_index, filing, notes, submissions};
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::{
    RawDailyIndex, RawNotesArchive, RawSubmissions, RawXbrlInstance, RawXbrlLabels,
};

/// Builds the extract super-states by ingestion mode and source.
///
/// Each mode offers only its own sources. [`Incremental`] offers no `notes()`, so this does not
/// compile:
///
/// ```compile_fail,E0599
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

    /// Starts an incremental run with a new random `execution_id`.
    #[must_use]
    pub fn incremental() -> Incremental {
        Incremental {
            execution_id: Uuid::new_v4(),
        }
    }
}

/// Builds the super-states of one backfill run.
///
/// Every super-state built from the same handle carries the handle's `execution_id` and
/// [`IngestionMode::Backfill`].
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

    /// Returns the builder of an [`ExtractNotesSuperState`] that holds the run's `execution_id`
    /// and [`IngestionMode::Backfill`].
    ///
    /// The builder still needs a period, an SEC client, and a repository of type `R`.
    #[must_use]
    pub fn notes<R: RawRepository<RawNotesArchive>>(
        &self,
    ) -> ExtractNotesSuperStateBuilder<
        NoPeriod,
        notes::builder::NoSecClient,
        Uuid,
        IngestionMode,
        notes::builder::NoRepository<R>,
    > {
        ExtractNotesSuperState::builder()
            .execution_id(self.execution_id)
            .ingestion_mode(IngestionMode::Backfill)
    }
}

/// Builds the super-states of one incremental run, the path of new filings.
///
/// Every super-state built from the same handle carries the handle's `execution_id` and
/// [`IngestionMode::Incremental`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Incremental {
    execution_id: Uuid,
}

impl Incremental {
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

    /// Returns the builder of a [`DetectFilingsSuperState`] for the run.
    ///
    /// The builder still needs a date, an SEC client, and a repository of type `R`.
    #[must_use]
    pub fn daily_index<R: RawRepository<RawDailyIndex>>(
        &self,
    ) -> DetectFilingsSuperStateBuilder<
        NoDate,
        daily_index::builder::NoSecClient,
        Uuid,
        IngestionMode,
        daily_index::builder::NoRepository<R>,
    > {
        DetectFilingsSuperState::builder()
            .execution_id(self.execution_id)
            .ingestion_mode(IngestionMode::Incremental)
    }

    /// Returns the builder of an [`ExtractFilingSuperState`] for the run.
    ///
    /// The builder still needs a filing, an SEC client, an instance repository of type `I`, and
    /// a labels repository of type `L`.
    #[must_use]
    pub fn filing<I: RawRepository<RawXbrlInstance>, L: RawRepository<RawXbrlLabels>>(
        &self,
    ) -> ExtractFilingSuperStateBuilder<
        NoFilingRef,
        filing::builder::NoSecClient,
        Uuid,
        IngestionMode,
        NoInstanceRepository<I>,
        NoLabelsRepository<L>,
    > {
        ExtractFilingSuperState::builder()
            .execution_id(self.execution_id)
            .ingestion_mode(IngestionMode::Incremental)
    }

    /// Returns the builder of an [`ExtractSubmissionsSuperState`] for the run.
    ///
    /// The builder still needs a CIK, an SEC client, and a repository of type `R`.
    #[must_use]
    pub fn submissions<R: RawRepository<RawSubmissions>>(
        &self,
    ) -> ExtractSubmissionsSuperStateBuilder<
        NoCik,
        submissions::builder::NoSecClient,
        Uuid,
        IngestionMode,
        submissions::builder::NoRepository<R>,
    > {
        ExtractSubmissionsSuperState::builder()
            .execution_id(self.execution_id)
            .ingestion_mode(IngestionMode::Incremental)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use futures_util::StreamExt;
    use pretty_assertions::{assert_eq, assert_ne};
    use storage::implementations::repository::filesystem::{Document, FilesystemRepository};
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::states::extract::validate_notes_period::ValidateNotesPeriod;
    use crate::shared::cik::Cik;
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::raw_document::RawDocument;
    use crate::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use crate::tests::fixtures::data::submissions::SAMPLE_SUBMISSIONS_JSON;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_write_repository::{
        FakeDailyIndexRepository, FakeNotesRepository, FakeSubmissionsRepository,
        FakeXbrlInstanceRepository, FakeXbrlLabelsRepository,
    };

    type NotesSuperState =
        ExtractNotesSuperState<ValidateNotesPeriod, FakeNotesRepository, AlwaysRespondingSecClient>;

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

    #[test]
    const fn should_implement_auto_traits_for_incremental() {
        implements_auto_traits::<Incremental>();
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
    const fn should_implement_value_traits_for_incremental() {
        implements_value_traits::<Incremental>();
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
            .ingestion_mode(IngestionMode::Backfill)
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

    fn filing_ref() -> FilingRef {
        FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenQ, false),
            chrono::NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    #[test]
    fn should_create_a_different_execution_id_when_starting_two_incremental_runs() {
        let expected_result = Extraction::incremental();

        let result = Extraction::incremental();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_return_the_given_execution_id_when_reading_the_id_of_an_incremental_run() {
        let incremental = Extraction::incremental().execution_id(Uuid::from_u128(7));

        let expected_result = Uuid::from_u128(7);

        let result = incremental.id();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_set_the_execution_id_and_the_incremental_mode_when_building_a_daily_index_super_state()
     {
        let incremental = Extraction::incremental().execution_id(Uuid::from_u128(7));
        let date = "2026-09-09";

        let expected_result = DetectFilingsSuperState::builder()
            .date(date)
            .sec_client(SecClient::default())
            .execution_id(Uuid::from_u128(7))
            .ingestion_mode(IngestionMode::Incremental)
            .repository(FakeDailyIndexRepository::new())
            .build();

        let result = incremental
            .daily_index()
            .date(date)
            .sec_client(SecClient::default())
            .repository(FakeDailyIndexRepository::new())
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_set_the_execution_id_and_the_incremental_mode_when_building_a_filing_super_state() {
        let incremental = Extraction::incremental().execution_id(Uuid::from_u128(7));
        let filing = filing_ref();

        let expected_result = ExtractFilingSuperState::builder()
            .filing_ref(filing.clone())
            .sec_client(SecClient::default())
            .execution_id(Uuid::from_u128(7))
            .ingestion_mode(IngestionMode::Incremental)
            .instance_repository(FakeXbrlInstanceRepository::new())
            .labels_repository(FakeXbrlLabelsRepository::new())
            .build();

        let result = incremental
            .filing()
            .filing_ref(filing)
            .sec_client(SecClient::default())
            .instance_repository(FakeXbrlInstanceRepository::new())
            .labels_repository(FakeXbrlLabelsRepository::new())
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_set_the_execution_id_and_the_incremental_mode_when_building_a_submissions_super_state()
     {
        let incremental = Extraction::incremental().execution_id(Uuid::from_u128(7));
        let cik = "1067983";

        let expected_result = ExtractSubmissionsSuperState::builder()
            .cik(cik)
            .sec_client(SecClient::default())
            .execution_id(Uuid::from_u128(7))
            .ingestion_mode(IngestionMode::Incremental)
            .repository(FakeSubmissionsRepository::new())
            .build();

        let result = incremental
            .submissions()
            .cik(cik)
            .sec_client(SecClient::default())
            .repository(FakeSubmissionsRepository::new())
            .build();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_incremental_as_the_ingestion_mode_when_running_a_submissions_super_state()
    {
        let repository = FakeSubmissionsRepository::new();
        let super_state = Extraction::incremental()
            .submissions()
            .cik("1067983")
            .sec_client(AlwaysRespondingSecClient::new(
                200,
                vec![("content-type", "application/json".to_string())],
                SAMPLE_SUBMISSIONS_JSON.to_vec(),
            ))
            .repository(repository.clone())
            .build();

        let expected_result = vec![IngestionMode::Incremental];

        let mut stream = super_state.into_run_stream();
        while let Some(item) = stream.next().await {
            item.expect(
                "Given a valid CIK, the fake client, and a fake store, every stream item should be a success",
            );
        }
        let result: Vec<IngestionMode> = repository
            .persisted()
            .iter()
            .map(|document| document.metadata().ingestion_mode())
            .collect();

        assert_eq!(result, expected_result);
    }
}
