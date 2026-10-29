//! # Builder
//!
//! Provides [`ExtractCompanyFactsSuperStateBuilder`], which builds an
//! [`ExtractCompanyFactsSuperState`] at its entry state [`ValidateCikFormat`].
//!
//! Each setter changes the builder's type. [`build`](ExtractCompanyFactsSuperStateBuilder::build)
//! exists only once the builder holds a CIK, an [`SecClient`], an `execution_id`, and a repository,
//! so a super-state without one of them does not compile.

use std::marker::PhantomData;

use uuid::Uuid;

use crate::implementations::states::extract::store_company_facts::RawCompanyFactsRepository;
use crate::implementations::states::extract::validate_cik_format::{
    ValidateCikFormat, ValidateCikFormatContext, ValidateCikFormatInput,
};
use crate::implementations::states::extract::{
    ExtractCompanyFactsSuperState, ExtractCompanyFactsSuperStateContext,
    ExtractCompanyFactsSuperStateData,
};
use crate::shared::http_client::implementations::sec_client::SecClient;

/// Marks a builder that holds no CIK yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoCik;

/// Marks a builder that holds no [`SecClient`] yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoSecClient;

/// Marks a builder that holds no `execution_id` yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoExecutionId;

/// Marks a builder that holds no repository of type `R` yet.
///
/// The marker carries `R`. The compiler infers `R` from the value passed to
/// [`repository`](ExtractCompanyFactsSuperStateBuilder::repository).
// Deviation: the derives bound `R` on each trait, although the marker holds no `R`. Every
// `RawCompanyFactsRepository` implements these traits, so the bounds cost nothing here.
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

/// Builds an [`ExtractCompanyFactsSuperState`] at its entry state [`ValidateCikFormat`].
///
/// Set the raw CIK with [`cik`](Self::cik), the client with [`sec_client`](Self::sec_client), the
/// run identifier with [`execution_id`](Self::execution_id), and the store with
/// [`repository`](Self::repository), in any order. Then call [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractCompanyFactsSuperStateBuilder<C, S, E, R> {
    cik: C,
    sec_client: S,
    execution_id: E,
    repository: R,
}

impl<R> ExtractCompanyFactsSuperStateBuilder<NoCik, NoSecClient, NoExecutionId, NoRepository<R>> {
    /// Creates a new [`ExtractCompanyFactsSuperStateBuilder`] that holds no field.
    pub(crate) const fn new() -> Self {
        Self {
            cik: NoCik,
            sec_client: NoSecClient,
            execution_id: NoExecutionId,
            repository: NoRepository::new(),
        }
    }
}

impl<S, E, R> ExtractCompanyFactsSuperStateBuilder<NoCik, S, E, R> {
    /// Sets the raw CIK. The built super-state validates it in [`ValidateCikFormat`].
    #[must_use]
    pub fn cik(
        self,
        cik: impl Into<String>,
    ) -> ExtractCompanyFactsSuperStateBuilder<String, S, E, R> {
        ExtractCompanyFactsSuperStateBuilder {
            cik: cik.into(),
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            repository: self.repository,
        }
    }
}

impl<C, E, R> ExtractCompanyFactsSuperStateBuilder<C, NoSecClient, E, R> {
    /// Sets the [`SecClient`] that every state of the extract phase shares.
    #[must_use]
    pub fn sec_client(
        self,
        sec_client: SecClient,
    ) -> ExtractCompanyFactsSuperStateBuilder<C, SecClient, E, R> {
        ExtractCompanyFactsSuperStateBuilder {
            cik: self.cik,
            sec_client,
            execution_id: self.execution_id,
            repository: self.repository,
        }
    }
}

impl<C, S, R> ExtractCompanyFactsSuperStateBuilder<C, S, NoExecutionId, R> {
    /// Sets the `execution_id` that tags the stream and the stored document of the run.
    #[must_use]
    pub fn execution_id(
        self,
        execution_id: Uuid,
    ) -> ExtractCompanyFactsSuperStateBuilder<C, S, Uuid, R> {
        ExtractCompanyFactsSuperStateBuilder {
            cik: self.cik,
            sec_client: self.sec_client,
            execution_id,
            repository: self.repository,
        }
    }
}

impl<C, S, E, R> ExtractCompanyFactsSuperStateBuilder<C, S, E, NoRepository<R>> {
    /// Sets the repository that stores the raw Company Facts.
    #[must_use]
    pub fn repository(self, repository: R) -> ExtractCompanyFactsSuperStateBuilder<C, S, E, R> {
        ExtractCompanyFactsSuperStateBuilder {
            cik: self.cik,
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            repository,
        }
    }
}

impl<R: RawCompanyFactsRepository>
    ExtractCompanyFactsSuperStateBuilder<String, SecClient, Uuid, R>
{
    /// Builds an [`ExtractCompanyFactsSuperState`] at [`ValidateCikFormat`] with the fields this
    /// builder holds.
    ///
    /// # Examples
    ///
    /// A builder without an `execution_id` has no `build` method, so this does not compile:
    ///
    /// ```compile_fail,E0599
    /// use sec::implementations::states::extract::ExtractCompanyFactsSuperState;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::raw_document::RawDocument;
    /// use sec::shared::raw_source::RawCompanyFacts;
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
    ///
    /// let repository = FilesystemRepository::builder()
    ///     .writer()
    ///     .document_type::<RawDocument<RawCompanyFacts>>()
    ///     .root("data")
    ///     .build();
    ///
    /// let extract_state = ExtractCompanyFactsSuperState::builder()
    ///     .cik("1067983")
    ///     .sec_client(SecClient::default())
    ///     .repository(repository)
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> ExtractCompanyFactsSuperState<ValidateCikFormat<R>, R> {
        let input_data = ValidateCikFormatInput::new(self.cik.clone());
        let state_context = ValidateCikFormatContext::new(
            self.cik,
            self.sec_client.clone(),
            self.repository.clone(),
        );

        ExtractCompanyFactsSuperState {
            current_state: ValidateCikFormat::new(input_data, state_context),
            input: ExtractCompanyFactsSuperStateData,
            output: None,
            context: ExtractCompanyFactsSuperStateContext::new(
                self.sec_client,
                self.execution_id,
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

    use super::*;
    use crate::shared::http_client::implementations::reqwest_client::ReqwestClient;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    type EmptyBuilder = ExtractCompanyFactsSuperStateBuilder<
        NoCik,
        NoSecClient,
        NoExecutionId,
        NoRepository<FakeWriteRepository>,
    >;

    fn sample_sec_client() -> SecClient {
        let user_agent = UserAgent::new("Sample Company contact@example.com")
            .expect("Given a hardcoded SEC-format string, validation should always succeed");
        SecClient::new(
            ReqwestClient::new(user_agent)
                .expect("Given a valid user agent, building the HTTP client should always succeed"),
        )
    }

    fn expected_super_state(
        cik: &str,
        sec_client: SecClient,
        execution_id: Uuid,
    ) -> ExtractCompanyFactsSuperState<ValidateCikFormat<FakeWriteRepository>, FakeWriteRepository>
    {
        let repository = FakeWriteRepository::new();
        let state_context =
            ValidateCikFormatContext::new(cik, sec_client.clone(), repository.clone());

        ExtractCompanyFactsSuperState {
            current_state: ValidateCikFormat::new(ValidateCikFormatInput::new(cik), state_context),
            input: ExtractCompanyFactsSuperStateData,
            output: None,
            context: ExtractCompanyFactsSuperStateContext::new(
                sec_client,
                execution_id,
                repository,
            ),
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_extract_company_facts_super_state_builder() {
        implements_auto_traits::<EmptyBuilder>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_extract_company_facts_super_state_builder() {
        implements_value_traits::<EmptyBuilder>();
    }

    #[test]
    const fn should_implement_value_traits_for_no_repository() {
        implements_value_traits::<NoRepository<FakeWriteRepository>>();
    }

    const fn holds_no_field(_: EmptyBuilder) {}
    #[test]
    const fn should_hold_no_field_when_starting_a_builder() {
        holds_no_field(ExtractCompanyFactsSuperState::builder());
    }

    #[test]
    fn should_build_a_super_state_at_validate_cik_format_when_setting_every_field() {
        let builder = ExtractCompanyFactsSuperState::builder()
            .cik("1067983")
            .sec_client(sample_sec_client())
            .execution_id(Uuid::from_u128(7));

        let expected_result =
            expected_super_state("1067983", sample_sec_client(), Uuid::from_u128(7));

        let result = builder.repository(FakeWriteRepository::new()).build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_same_super_state_when_setting_the_fields_in_reverse_order() {
        let builder = ExtractCompanyFactsSuperState::builder()
            .repository(FakeWriteRepository::new())
            .execution_id(Uuid::from_u128(7))
            .sec_client(sample_sec_client());

        let expected_result =
            expected_super_state("1067983", sample_sec_client(), Uuid::from_u128(7));

        let result = builder.cik("1067983").build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_compare_unequal_when_two_builders_hold_different_ciks() {
        let expected_result =
            ExtractCompanyFactsSuperState::<_, FakeWriteRepository>::builder().cik("1");

        let result = ExtractCompanyFactsSuperState::<_, FakeWriteRepository>::builder().cik("2");

        assert_ne!(result, expected_result);
    }
}
