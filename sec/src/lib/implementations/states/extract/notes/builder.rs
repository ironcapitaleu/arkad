//! # Builder
//!
//! Provides [`ExtractNotesSuperStateBuilder`], which builds an [`ExtractNotesSuperState`] at its
//! entry state [`ValidateNotesPeriod`].
//!
//! Each setter changes the builder's type. [`build`](ExtractNotesSuperStateBuilder::build) exists
//! only once the builder holds a period, an SEC client, an `execution_id`, an ingestion mode, and a
//! repository, so a super-state without one of them does not compile.

use std::marker::PhantomData;

use storage::implementations::repository::filesystem::IngestionMode;
use uuid::Uuid;

use crate::implementations::states::extract::notes::{
    ExtractNotesSuperState, ExtractNotesSuperStateContext, ExtractNotesSuperStateData,
};
use crate::implementations::states::extract::validate_notes_period::{
    ValidateNotesPeriod, ValidateNotesPeriodContext, ValidateNotesPeriodInput,
};
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawNotesArchive;
use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};

/// Marks a builder that holds no period yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoPeriod;

/// Marks a builder that holds no SEC client yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoSecClient;

/// Marks a builder that holds no `execution_id` yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoExecutionId;

/// Marks a builder that holds no ingestion mode yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoIngestionMode;

/// Marks a builder that holds no repository of type `R` yet.
///
/// The marker carries `R`. The compiler infers `R` from the value passed to
/// [`repository`](ExtractNotesSuperStateBuilder::repository).
// Deviation: the derives bound `R` on each trait, although the marker holds no `R`. Every
// `RawRepository<RawNotesArchive>` implements these traits, so the bounds cost nothing here.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoRepository<R>(PhantomData<fn() -> R>);

impl<R> NoRepository<R> {
    /// Creates a new [`NoRepository`] marker.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<R> Default for NoRepository<R> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds an [`ExtractNotesSuperState`] at its entry state [`ValidateNotesPeriod`].
///
/// Set the raw period with [`period`](Self::period), the client with
/// [`sec_client`](Self::sec_client), the run identifier with [`execution_id`](Self::execution_id),
/// the mode of the run with [`ingestion_mode`](Self::ingestion_mode), and the store with
/// [`repository`](Self::repository), in any order. Then call [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractNotesSuperStateBuilder<P, C, E, M, R> {
    period: P,
    sec_client: C,
    execution_id: E,
    ingestion_mode: M,
    repository: R,
}

impl<R>
    ExtractNotesSuperStateBuilder<
        NoPeriod,
        NoSecClient,
        NoExecutionId,
        NoIngestionMode,
        NoRepository<R>,
    >
{
    /// Creates a new [`ExtractNotesSuperStateBuilder`] that holds no field.
    pub(crate) const fn new() -> Self {
        Self {
            period: NoPeriod,
            sec_client: NoSecClient,
            execution_id: NoExecutionId,
            ingestion_mode: NoIngestionMode,
            repository: NoRepository::new(),
        }
    }
}

impl<C, E, M, R> ExtractNotesSuperStateBuilder<NoPeriod, C, E, M, R> {
    /// Sets the raw period, for example `2024q1`. The built super-state validates it in
    /// [`ValidateNotesPeriod`].
    #[must_use]
    pub fn period(
        self,
        period: impl Into<String>,
    ) -> ExtractNotesSuperStateBuilder<String, C, E, M, R> {
        ExtractNotesSuperStateBuilder {
            period: period.into(),
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            repository: self.repository,
        }
    }
}

impl<P, E, M, R> ExtractNotesSuperStateBuilder<P, NoSecClient, E, M, R> {
    /// Sets the SEC client that downloads the Notes file.
    ///
    /// The client must return an [`SecResponse<ZipBody>`] and a [`FailedSecRequest`]. The output of
    /// `ExecuteSecRequest<NotesFile, …>` holds the response, and its error wraps the failure.
    #[must_use]
    pub fn sec_client<C>(self, sec_client: C) -> ExtractNotesSuperStateBuilder<P, C, E, M, R>
    where
        C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
    {
        ExtractNotesSuperStateBuilder {
            period: self.period,
            sec_client,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            repository: self.repository,
        }
    }
}

impl<P, C, M, R> ExtractNotesSuperStateBuilder<P, C, NoExecutionId, M, R> {
    /// Sets the `execution_id` that tags the stream and the stored document of the run.
    #[must_use]
    pub fn execution_id(
        self,
        execution_id: Uuid,
    ) -> ExtractNotesSuperStateBuilder<P, C, Uuid, M, R> {
        ExtractNotesSuperStateBuilder {
            period: self.period,
            sec_client: self.sec_client,
            execution_id,
            ingestion_mode: self.ingestion_mode,
            repository: self.repository,
        }
    }
}

impl<P, C, E, R> ExtractNotesSuperStateBuilder<P, C, E, NoIngestionMode, R> {
    /// Sets the ingestion mode of the run. The metadata of the stored document records it.
    ///
    /// The backfill passes [`IngestionMode::Backfill`].
    #[must_use]
    pub fn ingestion_mode(
        self,
        ingestion_mode: IngestionMode,
    ) -> ExtractNotesSuperStateBuilder<P, C, E, IngestionMode, R> {
        ExtractNotesSuperStateBuilder {
            period: self.period,
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            ingestion_mode,
            repository: self.repository,
        }
    }
}

impl<P, C, E, M, R> ExtractNotesSuperStateBuilder<P, C, E, M, NoRepository<R>> {
    /// Sets the repository that stores the Notes file.
    #[must_use]
    pub fn repository(self, repository: R) -> ExtractNotesSuperStateBuilder<P, C, E, M, R> {
        ExtractNotesSuperStateBuilder {
            period: self.period,
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            repository,
        }
    }
}

impl<C: SecClientTrait, R: RawRepository<RawNotesArchive>>
    ExtractNotesSuperStateBuilder<String, C, Uuid, IngestionMode, R>
{
    /// Builds an [`ExtractNotesSuperState`] at [`ValidateNotesPeriod`] with the fields this
    /// builder holds.
    ///
    /// # Examples
    ///
    /// A builder without an ingestion mode has no `build` method, so this does not compile:
    ///
    /// ```compile_fail,E0599
    /// use sec::implementations::states::extract::notes::ExtractNotesSuperState;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::raw_document::RawDocument;
    /// use sec::shared::raw_source::RawNotesArchive;
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
    /// use uuid::Uuid;
    ///
    /// let repository = FilesystemRepository::builder()
    ///     .writer()
    ///     .document_type::<RawDocument<RawNotesArchive>>()
    ///     .root("data")
    ///     .build();
    ///
    /// let extract_state = ExtractNotesSuperState::builder()
    ///     .period("2024q1")
    ///     .sec_client(SecClient::default())
    ///     .execution_id(Uuid::nil())
    ///     .repository(repository)
    ///     .build();
    /// ```
    ///
    /// The repository must store the Notes archive. So a builder with a Company Facts repository has
    /// no `build` method, and this does not compile:
    ///
    /// ```compile_fail,E0271,E0599
    /// use sec::implementations::states::extract::notes::ExtractNotesSuperState;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::raw_document::RawDocument;
    /// use sec::shared::raw_source::RawCompanyFacts;
    /// use storage::implementations::repository::filesystem::{FilesystemRepository, IngestionMode};
    /// use uuid::Uuid;
    ///
    /// let repository = FilesystemRepository::builder()
    ///     .writer()
    ///     .document_type::<RawDocument<RawCompanyFacts>>()
    ///     .root("data")
    ///     .build();
    ///
    /// let extract_state = ExtractNotesSuperState::builder()
    ///     .period("2024q1")
    ///     .sec_client(SecClient::default())
    ///     .execution_id(Uuid::nil())
    ///     .ingestion_mode(IngestionMode::Backfill)
    ///     .repository(repository)
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> ExtractNotesSuperState<ValidateNotesPeriod, R, C> {
        let input = ValidateNotesPeriodInput::new(self.period);
        let state_context = ValidateNotesPeriodContext::new(self.execution_id);

        ExtractNotesSuperState {
            current_state: ValidateNotesPeriod::new(input, state_context),
            input: ExtractNotesSuperStateData,
            output: None,
            context: ExtractNotesSuperStateContext::new(
                self.sec_client,
                self.execution_id,
                self.ingestion_mode,
                self.repository,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::{assert_eq, assert_ne};
    use storage::WriteError;

    use super::*;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_write_repository::FakeNotesRepository;

    type EmptyBuilder = ExtractNotesSuperStateBuilder<
        NoPeriod,
        NoSecClient,
        NoExecutionId,
        NoIngestionMode,
        NoRepository<FakeNotesRepository>,
    >;

    fn sample_sec_client() -> AlwaysRespondingSecClient {
        AlwaysRespondingSecClient::new(404, Vec::new(), Vec::new())
    }

    fn expected_super_state(
        raw_period: &str,
        sec_client: AlwaysRespondingSecClient,
        execution_id: Uuid,
        ingestion_mode: IngestionMode,
        repository: FakeNotesRepository,
    ) -> ExtractNotesSuperState<ValidateNotesPeriod, FakeNotesRepository, AlwaysRespondingSecClient>
    {
        ExtractNotesSuperState {
            current_state: ValidateNotesPeriod::new(
                ValidateNotesPeriodInput::new(raw_period),
                ValidateNotesPeriodContext::new(execution_id),
            ),
            input: ExtractNotesSuperStateData,
            output: None,
            context: ExtractNotesSuperStateContext::new(
                sec_client,
                execution_id,
                ingestion_mode,
                repository,
            ),
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_extract_notes_super_state_builder() {
        implements_auto_traits::<EmptyBuilder>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_extract_notes_super_state_builder() {
        implements_value_traits::<EmptyBuilder>();
    }

    const fn holds_no_field(_: EmptyBuilder) {}
    #[test]
    const fn should_hold_no_field_when_starting_a_builder() {
        holds_no_field(ExtractNotesSuperState::builder());
    }

    #[test]
    fn should_build_a_super_state_at_validate_notes_period_when_setting_every_field() {
        let period = "2024q1";
        let sec_client = sample_sec_client();
        let execution_id = Uuid::from_u128(7);
        let ingestion_mode = IngestionMode::Backfill;
        let repository =
            FakeNotesRepository::rejecting(WriteError::conflicting_write("Sample conflict"));
        let builder = ExtractNotesSuperState::builder()
            .period(period)
            .sec_client(sec_client.clone())
            .execution_id(execution_id)
            .ingestion_mode(ingestion_mode);

        let expected_result = expected_super_state(
            period,
            sec_client,
            execution_id,
            ingestion_mode,
            repository.clone(),
        );

        let result = builder.repository(repository).build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_same_super_state_when_setting_the_fields_in_reverse_order() {
        let period = "2024q1";
        let sec_client = sample_sec_client();
        let execution_id = Uuid::from_u128(7);
        let ingestion_mode = IngestionMode::Backfill;
        let repository =
            FakeNotesRepository::rejecting(WriteError::conflicting_write("Sample conflict"));
        let builder = ExtractNotesSuperState::builder()
            .repository(repository.clone())
            .ingestion_mode(ingestion_mode)
            .execution_id(execution_id)
            .sec_client(sec_client.clone());

        let expected_result =
            expected_super_state(period, sec_client, execution_id, ingestion_mode, repository);

        let result = builder.period(period).build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_compare_unequal_when_two_builders_hold_different_periods() {
        let builder = ExtractNotesSuperState::<ValidateNotesPeriod, FakeNotesRepository>::builder();
        let period = "2024q2";

        let expected_result =
            ExtractNotesSuperState::<ValidateNotesPeriod, FakeNotesRepository>::builder()
                .period("2024q1");

        let result = builder.period(period);

        assert_ne!(result, expected_result);
    }
}
