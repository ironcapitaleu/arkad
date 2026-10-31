//! # Builder
//!
//! Provides [`ExtractSubmissionsSuperStateBuilder`], which builds an
//! [`ExtractSubmissionsSuperState`] at its entry state [`ValidateCikFormat`].
//!
//! Each setter changes the builder's type. [`build`](ExtractSubmissionsSuperStateBuilder::build)
//! exists only once the builder holds a CIK, an SEC client, an `execution_id`, an ingestion mode,
//! and a repository, so a super-state without one of them does not compile.

use std::marker::PhantomData;

use storage::implementations::repository::filesystem::IngestionMode;
use uuid::Uuid;

use crate::implementations::states::extract::submissions::{
    ExtractSubmissionsSuperState, ExtractSubmissionsSuperStateContext,
    ExtractSubmissionsSuperStateData,
};
use crate::implementations::states::extract::validate_cik_format::{
    ValidateCikFormat, ValidateCikFormatContext, ValidateCikFormatInput,
};
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawSubmissions;
use crate::shared::response::implementations::sec_response::{JsonBody, SecResponse};

/// Marks a builder that holds no CIK yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoCik;

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
/// [`repository`](ExtractSubmissionsSuperStateBuilder::repository).
// Deviation: the derives bound `R` on each trait, although the marker holds no `R`. Every
// `RawRepository<RawSubmissions>` implements these traits, so the bounds cost nothing here.
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

/// Builds an [`ExtractSubmissionsSuperState`] at its entry state [`ValidateCikFormat`].
///
/// Set the raw CIK with [`cik`](Self::cik), the client with [`sec_client`](Self::sec_client), the
/// run identifier with [`execution_id`](Self::execution_id), the mode of the run with
/// [`ingestion_mode`](Self::ingestion_mode), and the store with [`repository`](Self::repository),
/// in any order. Then call [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractSubmissionsSuperStateBuilder<K, C, E, M, R> {
    cik: K,
    sec_client: C,
    execution_id: E,
    ingestion_mode: M,
    repository: R,
}

impl<R>
    ExtractSubmissionsSuperStateBuilder<
        NoCik,
        NoSecClient,
        NoExecutionId,
        NoIngestionMode,
        NoRepository<R>,
    >
{
    /// Creates a new [`ExtractSubmissionsSuperStateBuilder`] that holds no field.
    pub(crate) const fn new() -> Self {
        Self {
            cik: NoCik,
            sec_client: NoSecClient,
            execution_id: NoExecutionId,
            ingestion_mode: NoIngestionMode,
            repository: NoRepository::new(),
        }
    }
}

impl<C, E, M, R> ExtractSubmissionsSuperStateBuilder<NoCik, C, E, M, R> {
    /// Sets the raw CIK, for example `1067983`. The built super-state validates it in
    /// [`ValidateCikFormat`].
    #[must_use]
    pub fn cik(
        self,
        cik: impl Into<String>,
    ) -> ExtractSubmissionsSuperStateBuilder<String, C, E, M, R> {
        ExtractSubmissionsSuperStateBuilder {
            cik: cik.into(),
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            repository: self.repository,
        }
    }
}

impl<K, E, M, R> ExtractSubmissionsSuperStateBuilder<K, NoSecClient, E, M, R> {
    /// Sets the SEC client that fetches the `submissions` JSON.
    ///
    /// The client must return an [`SecResponse<JsonBody>`] and a [`FailedSecRequest`]. The output
    /// of `ExecuteSecRequest<Submissions, …>` holds the response, and its error wraps the failure.
    #[must_use]
    pub fn sec_client<C>(self, sec_client: C) -> ExtractSubmissionsSuperStateBuilder<K, C, E, M, R>
    where
        C: SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>, Error = FailedSecRequest>,
    {
        ExtractSubmissionsSuperStateBuilder {
            cik: self.cik,
            sec_client,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            repository: self.repository,
        }
    }
}

impl<K, C, M, R> ExtractSubmissionsSuperStateBuilder<K, C, NoExecutionId, M, R> {
    /// Sets the `execution_id` that tags the stream and the stored document of the run.
    #[must_use]
    pub fn execution_id(
        self,
        execution_id: Uuid,
    ) -> ExtractSubmissionsSuperStateBuilder<K, C, Uuid, M, R> {
        ExtractSubmissionsSuperStateBuilder {
            cik: self.cik,
            sec_client: self.sec_client,
            execution_id,
            ingestion_mode: self.ingestion_mode,
            repository: self.repository,
        }
    }
}

impl<K, C, E, R> ExtractSubmissionsSuperStateBuilder<K, C, E, NoIngestionMode, R> {
    /// Sets the ingestion mode of the run. The metadata of the stored document records it.
    ///
    /// The backfill passes [`IngestionMode::Backfill`]. The new-filing path passes
    /// [`IngestionMode::Incremental`].
    #[must_use]
    pub fn ingestion_mode(
        self,
        ingestion_mode: IngestionMode,
    ) -> ExtractSubmissionsSuperStateBuilder<K, C, E, IngestionMode, R> {
        ExtractSubmissionsSuperStateBuilder {
            cik: self.cik,
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            ingestion_mode,
            repository: self.repository,
        }
    }
}

impl<K, C, E, M, R> ExtractSubmissionsSuperStateBuilder<K, C, E, M, NoRepository<R>> {
    /// Sets the repository that stores the `submissions` JSON.
    #[must_use]
    pub fn repository(self, repository: R) -> ExtractSubmissionsSuperStateBuilder<K, C, E, M, R> {
        ExtractSubmissionsSuperStateBuilder {
            cik: self.cik,
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            repository,
        }
    }
}

impl<C: SecClientTrait, R: RawRepository<RawSubmissions>>
    ExtractSubmissionsSuperStateBuilder<String, C, Uuid, IngestionMode, R>
{
    /// Builds an [`ExtractSubmissionsSuperState`] at [`ValidateCikFormat`] with the fields this
    /// builder holds.
    ///
    /// # Examples
    ///
    /// A builder without an ingestion mode has no `build` method, so this does not compile:
    ///
    /// ```compile_fail,E0599
    /// use sec::implementations::states::extract::submissions::ExtractSubmissionsSuperState;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::raw_document::RawDocument;
    /// use sec::shared::raw_source::RawSubmissions;
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
    /// use uuid::Uuid;
    ///
    /// let repository = FilesystemRepository::builder()
    ///     .writer()
    ///     .document_type::<RawDocument<RawSubmissions>>()
    ///     .root("data")
    ///     .build();
    ///
    /// let extract_state = ExtractSubmissionsSuperState::builder()
    ///     .cik("1067983")
    ///     .sec_client(SecClient::default())
    ///     .execution_id(Uuid::nil())
    ///     .repository(repository)
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> ExtractSubmissionsSuperState<ValidateCikFormat<R, C>, R, C> {
        let input = ValidateCikFormatInput::new(self.cik.clone());
        let state_context = ValidateCikFormatContext::new(
            self.cik,
            self.sec_client.clone(),
            self.repository.clone(),
        );

        ExtractSubmissionsSuperState {
            current_state: ValidateCikFormat::new(input, state_context),
            input: ExtractSubmissionsSuperStateData,
            output: None,
            context: ExtractSubmissionsSuperStateContext::new(
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
    use crate::tests::fixtures::sample_write_repository::FakeSubmissionsRepository;

    type EmptyBuilder = ExtractSubmissionsSuperStateBuilder<
        NoCik,
        NoSecClient,
        NoExecutionId,
        NoIngestionMode,
        NoRepository<FakeSubmissionsRepository>,
    >;

    type TestSuperState = ExtractSubmissionsSuperState<
        ValidateCikFormat<FakeSubmissionsRepository, AlwaysRespondingSecClient>,
        FakeSubmissionsRepository,
        AlwaysRespondingSecClient,
    >;

    fn sample_sec_client() -> AlwaysRespondingSecClient {
        AlwaysRespondingSecClient::new(404, Vec::new(), Vec::new())
    }

    fn expected_super_state(
        raw_cik: &str,
        sec_client: AlwaysRespondingSecClient,
        execution_id: Uuid,
        ingestion_mode: IngestionMode,
        repository: FakeSubmissionsRepository,
    ) -> TestSuperState {
        ExtractSubmissionsSuperState {
            current_state: ValidateCikFormat::new(
                ValidateCikFormatInput::new(raw_cik),
                ValidateCikFormatContext::new(raw_cik, sec_client.clone(), repository.clone()),
            ),
            input: ExtractSubmissionsSuperStateData,
            output: None,
            context: ExtractSubmissionsSuperStateContext::new(
                sec_client,
                execution_id,
                ingestion_mode,
                repository,
            ),
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_extract_submissions_super_state_builder() {
        implements_auto_traits::<EmptyBuilder>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_extract_submissions_super_state_builder() {
        implements_value_traits::<EmptyBuilder>();
    }

    const fn holds_no_field(_: EmptyBuilder) {}
    #[test]
    const fn should_hold_no_field_when_starting_a_builder() {
        holds_no_field(ExtractSubmissionsSuperState::builder());
    }

    #[test]
    fn should_build_a_super_state_at_validate_cik_format_when_setting_every_field() {
        let cik = "1067983";
        let sec_client = sample_sec_client();
        let execution_id = Uuid::from_u128(7);
        let ingestion_mode = IngestionMode::Incremental;
        let repository =
            FakeSubmissionsRepository::rejecting(WriteError::conflicting_write("Sample conflict"));
        let builder = ExtractSubmissionsSuperState::builder()
            .cik(cik)
            .sec_client(sec_client.clone())
            .execution_id(execution_id)
            .ingestion_mode(ingestion_mode);

        let expected_result = expected_super_state(
            cik,
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
        let cik = "1067983";
        let sec_client = sample_sec_client();
        let execution_id = Uuid::from_u128(7);
        let ingestion_mode = IngestionMode::Backfill;
        let repository =
            FakeSubmissionsRepository::rejecting(WriteError::conflicting_write("Sample conflict"));
        let builder = ExtractSubmissionsSuperState::builder()
            .repository(repository.clone())
            .ingestion_mode(ingestion_mode)
            .execution_id(execution_id)
            .sec_client(sec_client.clone());

        let expected_result =
            expected_super_state(cik, sec_client, execution_id, ingestion_mode, repository);

        let result = builder.cik(cik).build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_compare_unequal_when_two_builders_hold_different_ingestion_modes() {
        let builder = ExtractSubmissionsSuperState::<
            ValidateCikFormat<FakeSubmissionsRepository>,
            FakeSubmissionsRepository,
        >::builder();
        let ingestion_mode = IngestionMode::Incremental;

        let expected_result = ExtractSubmissionsSuperState::<
            ValidateCikFormat<FakeSubmissionsRepository>,
            FakeSubmissionsRepository,
        >::builder()
        .ingestion_mode(IngestionMode::Backfill);

        let result = builder.ingestion_mode(ingestion_mode);

        assert_ne!(result, expected_result);
    }
}
