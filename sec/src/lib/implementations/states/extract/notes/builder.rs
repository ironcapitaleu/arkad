//! # Builder
//!
//! Provides [`ExtractNotesSuperStateBuilder`], which builds an [`ExtractNotesSuperState`] at its
//! entry state [`ValidateNotesPeriod`].
//!
//! Each setter changes the builder's type. [`build`](ExtractNotesSuperStateBuilder::build) exists
//! only once the builder holds a period, an SEC client, an `execution_id`, and a repository, so a
//! super-state without one of them does not compile.

use std::marker::PhantomData;

use uuid::Uuid;

use crate::implementations::states::extract::notes::{
    ExtractNotesSuperState, ExtractNotesSuperStateContext, ExtractNotesSuperStateData,
};
use crate::implementations::states::extract::notes_repository::NotesRepository;
use crate::implementations::states::extract::validate_notes_period::{
    ValidateNotesPeriod, ValidateNotesPeriodContext, ValidateNotesPeriodInput,
};
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
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

/// Marks a builder that holds no repository of type `R` yet.
///
/// The marker carries `R`. The compiler infers `R` from the value passed to
/// [`repository`](ExtractNotesSuperStateBuilder::repository).
// Deviation: the derives bound `R` on each trait, although the marker holds no `R`. Every
// `NotesRepository` implements these traits, so the bounds cost nothing here.
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
/// and the store with [`repository`](Self::repository), in any order. Then call
/// [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractNotesSuperStateBuilder<P, C, E, R> {
    period: P,
    sec_client: C,
    execution_id: E,
    repository: R,
}

impl<R> ExtractNotesSuperStateBuilder<NoPeriod, NoSecClient, NoExecutionId, NoRepository<R>> {
    /// Creates a new [`ExtractNotesSuperStateBuilder`] that holds no field.
    pub(crate) const fn new() -> Self {
        Self {
            period: NoPeriod,
            sec_client: NoSecClient,
            execution_id: NoExecutionId,
            repository: NoRepository::new(),
        }
    }
}

impl<C, E, R> ExtractNotesSuperStateBuilder<NoPeriod, C, E, R> {
    /// Sets the raw period, for example `2024q1`. The built super-state validates it in
    /// [`ValidateNotesPeriod`].
    #[must_use]
    pub fn period(
        self,
        period: impl Into<String>,
    ) -> ExtractNotesSuperStateBuilder<String, C, E, R> {
        ExtractNotesSuperStateBuilder {
            period: period.into(),
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            repository: self.repository,
        }
    }
}

impl<P, E, R> ExtractNotesSuperStateBuilder<P, NoSecClient, E, R> {
    /// Sets the SEC client that downloads the Notes file.
    ///
    /// The client must return a [`SecResponse<ZipBody>`] and a [`FailedSecRequest`], as
    /// `DownloadNotesArchive` stores them.
    #[must_use]
    pub fn sec_client<C>(self, sec_client: C) -> ExtractNotesSuperStateBuilder<P, C, E, R>
    where
        C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
    {
        ExtractNotesSuperStateBuilder {
            period: self.period,
            sec_client,
            execution_id: self.execution_id,
            repository: self.repository,
        }
    }
}

impl<P, C, R> ExtractNotesSuperStateBuilder<P, C, NoExecutionId, R> {
    /// Sets the `execution_id` that tags the stream and the stored document of the run.
    #[must_use]
    pub fn execution_id(self, execution_id: Uuid) -> ExtractNotesSuperStateBuilder<P, C, Uuid, R> {
        ExtractNotesSuperStateBuilder {
            period: self.period,
            sec_client: self.sec_client,
            execution_id,
            repository: self.repository,
        }
    }
}

impl<P, C, E, R> ExtractNotesSuperStateBuilder<P, C, E, NoRepository<R>> {
    /// Sets the repository that stores the Notes file.
    #[must_use]
    pub fn repository(self, repository: R) -> ExtractNotesSuperStateBuilder<P, C, E, R> {
        ExtractNotesSuperStateBuilder {
            period: self.period,
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            repository,
        }
    }
}

impl<C: SecClientTrait, R: NotesRepository> ExtractNotesSuperStateBuilder<String, C, Uuid, R> {
    /// Builds an [`ExtractNotesSuperState`] at [`ValidateNotesPeriod`] with the fields this
    /// builder holds.
    ///
    /// # Examples
    ///
    /// A builder without an `execution_id` has no `build` method, so this does not compile:
    ///
    /// ```compile_fail,E0599
    /// use sec::implementations::states::extract::notes::ExtractNotesSuperState;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::raw_document::RawDocument;
    /// use sec::shared::raw_source::RawNotesArchive;
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
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
    ///     .repository(repository)
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> ExtractNotesSuperState<ValidateNotesPeriod<R>, C> {
        let input = ValidateNotesPeriodInput::new(self.period);
        let state_context = ValidateNotesPeriodContext::new(self.execution_id, self.repository);

        ExtractNotesSuperState {
            current_state: ValidateNotesPeriod::new(input, state_context),
            input: ExtractNotesSuperStateData,
            output: None,
            context: ExtractNotesSuperStateContext::new(self.sec_client, self.execution_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_write_repository::FakeNotesRepository;

    type EmptyBuilder = ExtractNotesSuperStateBuilder<
        NoPeriod,
        NoSecClient,
        NoExecutionId,
        NoRepository<FakeNotesRepository>,
    >;

    fn sec_client() -> AlwaysRespondingSecClient {
        AlwaysRespondingSecClient::new(404, Vec::new(), Vec::new())
    }

    fn expected_super_state(
        raw_period: &str,
        execution_id: Uuid,
    ) -> ExtractNotesSuperState<ValidateNotesPeriod<FakeNotesRepository>, AlwaysRespondingSecClient>
    {
        ExtractNotesSuperState {
            current_state: ValidateNotesPeriod::new(
                ValidateNotesPeriodInput::new(raw_period),
                ValidateNotesPeriodContext::new(execution_id, FakeNotesRepository::new()),
            ),
            input: ExtractNotesSuperStateData,
            output: None,
            context: ExtractNotesSuperStateContext::new(sec_client(), execution_id),
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
        let builder = ExtractNotesSuperState::builder()
            .period("2024q1")
            .sec_client(sec_client())
            .execution_id(Uuid::from_u128(7));

        let expected_result = expected_super_state("2024q1", Uuid::from_u128(7));

        let result = builder.repository(FakeNotesRepository::new()).build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_same_super_state_when_setting_the_fields_in_reverse_order() {
        let builder = ExtractNotesSuperState::builder()
            .repository(FakeNotesRepository::new())
            .execution_id(Uuid::from_u128(7))
            .sec_client(sec_client());

        let expected_result = expected_super_state("2024q1", Uuid::from_u128(7));

        let result = builder.period("2024q1").build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_compare_unequal_when_two_builders_hold_different_periods() {
        let expected_result =
            ExtractNotesSuperState::<ValidateNotesPeriod<FakeNotesRepository>>::builder()
                .period("2024q1");

        let result = ExtractNotesSuperState::<ValidateNotesPeriod<FakeNotesRepository>>::builder()
            .period("2024q2");

        assert_ne!(result, expected_result);
    }
}
