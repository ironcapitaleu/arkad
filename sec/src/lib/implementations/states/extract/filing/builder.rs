//! # Builder
//!
//! Provides [`ExtractFilingSuperStateBuilder`], which builds an [`ExtractFilingSuperState`] at its
//! entry state, the [`PrepareSecRequest`] of the [`FilingIndex`] kind.
//!
//! Each setter changes the builder's type. [`build`](ExtractFilingSuperStateBuilder::build) exists
//! only once the builder holds a [`FilingRef`], an SEC client, an `execution_id`, an ingestion
//! mode, and the three repositories, so a super-state without one of them does not compile.

use std::marker::PhantomData;

use storage::implementations::repository::filesystem::{IngestionMode, InputDocument};
use uuid::Uuid;

use crate::implementations::states::extract::filing::{
    ExtractFilingSuperState, ExtractFilingSuperStateContext, ExtractFilingSuperStateData,
};
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use crate::shared::filing_ref::FilingRef;
use crate::shared::http_client::{Executes, SecClient as SecClientTrait};
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::{RawFilingIndex, RawXbrlInstance, RawXbrlLabels};
use crate::shared::request::implementations::sec_request::{FilingDocumentSet, FilingIndex};

/// Marks a builder that holds no [`FilingRef`] yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoFilingRef;

/// Marks a builder that holds no SEC client yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoSecClient;

/// Marks a builder that holds no `execution_id` yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoExecutionId;

/// Marks a builder that holds no ingestion mode yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoIngestionMode;

/// Marks a builder that holds no repository of type `X` for the folder index yet.
///
/// The marker carries `X`. The compiler infers `X` from the value passed to
/// [`filing_index_repository`](ExtractFilingSuperStateBuilder::filing_index_repository).
// Deviation: the derives bound `X` on each trait, although the marker holds no `X`. Every
// `RawRepository<RawFilingIndex>` implements these traits, so the bounds cost nothing here.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoFilingIndexRepository<X>(PhantomData<fn() -> X>);

impl<X> NoFilingIndexRepository<X> {
    /// Creates a new [`NoFilingIndexRepository`] marker.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<X> Default for NoFilingIndexRepository<X> {
    fn default() -> Self {
        Self::new()
    }
}

/// Marks a builder that holds no repository of type `I` for the instance yet.
///
/// The marker carries `I`. The compiler infers `I` from the value passed to
/// [`instance_repository`](ExtractFilingSuperStateBuilder::instance_repository).
// Deviation: the derives bound `I` on each trait, although the marker holds no `I`. Every
// `RawRepository<RawXbrlInstance>` implements these traits, so the bounds cost nothing here.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoInstanceRepository<I>(PhantomData<fn() -> I>);

impl<I> NoInstanceRepository<I> {
    /// Creates a new [`NoInstanceRepository`] marker.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> Default for NoInstanceRepository<I> {
    fn default() -> Self {
        Self::new()
    }
}

/// Marks a builder that holds no repository of type `L` for the label file yet.
///
/// The marker carries `L`. The compiler infers `L` from the value passed to
/// [`labels_repository`](ExtractFilingSuperStateBuilder::labels_repository).
// Deviation: the derives bound `L` on each trait, although the marker holds no `L`. Every
// `RawRepository<RawXbrlLabels>` implements these traits, so the bounds cost nothing here.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoLabelsRepository<L>(PhantomData<fn() -> L>);

impl<L> NoLabelsRepository<L> {
    /// Creates a new [`NoLabelsRepository`] marker.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<L> Default for NoLabelsRepository<L> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds an [`ExtractFilingSuperState`] at its entry state, the [`PrepareSecRequest`] of the
/// [`FilingIndex`] kind.
///
/// Set the filing with [`filing_ref`](Self::filing_ref), the client with
/// [`sec_client`](Self::sec_client), the run identifier with [`execution_id`](Self::execution_id),
/// the mode of the run with [`ingestion_mode`](Self::ingestion_mode), and the stores with
/// [`instance_repository`](Self::instance_repository) and
/// [`labels_repository`](Self::labels_repository), in any order. Then call [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractFilingSuperStateBuilder<F, C, E, M, X, I, L> {
    filing_ref: F,
    sec_client: C,
    execution_id: E,
    ingestion_mode: M,
    filing_index_repository: X,
    instance_repository: I,
    labels_repository: L,
    inputs: Vec<InputDocument>,
}

impl<X, I, L>
    ExtractFilingSuperStateBuilder<
        NoFilingRef,
        NoSecClient,
        NoExecutionId,
        NoIngestionMode,
        NoFilingIndexRepository<X>,
        NoInstanceRepository<I>,
        NoLabelsRepository<L>,
    >
{
    /// Creates a new [`ExtractFilingSuperStateBuilder`] that holds no field.
    pub(crate) const fn new() -> Self {
        Self {
            filing_ref: NoFilingRef,
            sec_client: NoSecClient,
            execution_id: NoExecutionId,
            ingestion_mode: NoIngestionMode,
            filing_index_repository: NoFilingIndexRepository::new(),
            instance_repository: NoInstanceRepository::new(),
            labels_repository: NoLabelsRepository::new(),
            inputs: Vec::new(),
        }
    }
}

impl<C, E, M, X, I, L> ExtractFilingSuperStateBuilder<NoFilingRef, C, E, M, X, I, L> {
    /// Sets the filing whose documents the super-state downloads.
    #[must_use]
    pub fn filing_ref(
        self,
        filing_ref: FilingRef,
    ) -> ExtractFilingSuperStateBuilder<FilingRef, C, E, M, X, I, L> {
        ExtractFilingSuperStateBuilder {
            filing_ref,
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            filing_index_repository: self.filing_index_repository,
            instance_repository: self.instance_repository,
            labels_repository: self.labels_repository,
            inputs: self.inputs,
        }
    }
}

impl<F, E, M, X, I, L> ExtractFilingSuperStateBuilder<F, NoSecClient, E, M, X, I, L> {
    /// Sets the SEC client that downloads the folder index and the filing documents.
    #[must_use]
    pub fn sec_client<C>(self, sec_client: C) -> ExtractFilingSuperStateBuilder<F, C, E, M, X, I, L>
    where
        C: SecClientTrait + Executes<FilingIndex> + Executes<FilingDocumentSet>,
    {
        ExtractFilingSuperStateBuilder {
            filing_ref: self.filing_ref,
            sec_client,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            filing_index_repository: self.filing_index_repository,
            instance_repository: self.instance_repository,
            labels_repository: self.labels_repository,
            inputs: self.inputs,
        }
    }
}

impl<F, C, M, X, I, L> ExtractFilingSuperStateBuilder<F, C, NoExecutionId, M, X, I, L> {
    /// Sets the `execution_id` that tags the stream and the stored documents of the run.
    #[must_use]
    pub fn execution_id(
        self,
        execution_id: Uuid,
    ) -> ExtractFilingSuperStateBuilder<F, C, Uuid, M, X, I, L> {
        ExtractFilingSuperStateBuilder {
            filing_ref: self.filing_ref,
            sec_client: self.sec_client,
            execution_id,
            ingestion_mode: self.ingestion_mode,
            filing_index_repository: self.filing_index_repository,
            instance_repository: self.instance_repository,
            labels_repository: self.labels_repository,
            inputs: self.inputs,
        }
    }
}

impl<F, C, E, X, I, L> ExtractFilingSuperStateBuilder<F, C, E, NoIngestionMode, X, I, L> {
    /// Sets the ingestion mode of the run. The metadata of the stored documents records it.
    ///
    /// The new-filing path passes [`IngestionMode::Incremental`].
    #[must_use]
    pub fn ingestion_mode(
        self,
        ingestion_mode: IngestionMode,
    ) -> ExtractFilingSuperStateBuilder<F, C, E, IngestionMode, X, I, L> {
        ExtractFilingSuperStateBuilder {
            filing_ref: self.filing_ref,
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            ingestion_mode,
            filing_index_repository: self.filing_index_repository,
            instance_repository: self.instance_repository,
            labels_repository: self.labels_repository,
            inputs: self.inputs,
        }
    }
}

impl<F, C, E, M, X, I, L>
    ExtractFilingSuperStateBuilder<F, C, E, M, NoFilingIndexRepository<X>, I, L>
{
    /// Sets the repository that stores the folder index.
    #[must_use]
    pub fn filing_index_repository(
        self,
        filing_index_repository: X,
    ) -> ExtractFilingSuperStateBuilder<F, C, E, M, X, I, L> {
        ExtractFilingSuperStateBuilder {
            filing_ref: self.filing_ref,
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            filing_index_repository,
            instance_repository: self.instance_repository,
            labels_repository: self.labels_repository,
            inputs: self.inputs,
        }
    }
}

impl<F, C, E, M, X, I, L>
    ExtractFilingSuperStateBuilder<F, C, E, M, X, NoInstanceRepository<I>, L>
{
    /// Sets the repository that stores the XBRL instance.
    #[must_use]
    pub fn instance_repository(
        self,
        instance_repository: I,
    ) -> ExtractFilingSuperStateBuilder<F, C, E, M, X, I, L> {
        ExtractFilingSuperStateBuilder {
            filing_ref: self.filing_ref,
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            filing_index_repository: self.filing_index_repository,
            instance_repository,
            labels_repository: self.labels_repository,
            inputs: self.inputs,
        }
    }
}

impl<F, C, E, M, X, I, L> ExtractFilingSuperStateBuilder<F, C, E, M, X, I, NoLabelsRepository<L>> {
    /// Sets the repository that stores the label file.
    #[must_use]
    pub fn labels_repository(
        self,
        labels_repository: L,
    ) -> ExtractFilingSuperStateBuilder<F, C, E, M, X, I, L> {
        ExtractFilingSuperStateBuilder {
            filing_ref: self.filing_ref,
            sec_client: self.sec_client,
            execution_id: self.execution_id,
            ingestion_mode: self.ingestion_mode,
            filing_index_repository: self.filing_index_repository,
            instance_repository: self.instance_repository,
            labels_repository,
            inputs: self.inputs,
        }
    }
}

impl<F, C, E, M, X, I, L> ExtractFilingSuperStateBuilder<F, C, E, M, X, I, L> {
    /// Sets the documents that the filing came from, for example the daily index that listed it.
    /// The metadata of each stored file records them as inputs. Without this call, it records
    /// none.
    #[must_use]
    pub fn inputs(self, inputs: Vec<InputDocument>) -> Self {
        Self { inputs, ..self }
    }
}

impl<C, X, I, L> ExtractFilingSuperStateBuilder<FilingRef, C, Uuid, IngestionMode, X, I, L>
where
    C: SecClientTrait,
    X: RawRepository<RawFilingIndex>,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
{
    /// Builds an [`ExtractFilingSuperState`] at the [`PrepareSecRequest`] of the [`FilingIndex`]
    /// kind, with the fields this builder holds.
    ///
    /// # Examples
    ///
    /// A builder without an ingestion mode has no `build` method, so this does not compile:
    ///
    /// ```compile_fail,E0599
    /// use chrono::NaiveDate;
    /// use sec::implementations::states::extract::filing::ExtractFilingSuperState;
    /// use sec::shared::cik::Cik;
    /// use sec::shared::filing_ref::FilingRef;
    /// use sec::shared::financial::accession_number::AccessionNumber;
    /// use sec::shared::financial::form::{Form, ParsedForm};
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::raw_document::RawDocument;
    /// use sec::shared::raw_source::{RawFilingIndex, RawXbrlInstance, RawXbrlLabels};
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
    /// use uuid::Uuid;
    ///
    /// let filing_ref = FilingRef::new(
    ///     Cik::new("3545").expect("The CIK is valid"),
    ///     AccessionNumber::new("0000003545-26-000047").expect("The accession number is valid"),
    ///     ParsedForm::new(Form::TenQ, false),
    ///     NaiveDate::from_ymd_opt(2026, 9, 9).expect("The date exists"),
    /// );
    /// let filing_index_repository = FilesystemRepository::builder()
    ///     .writer()
    ///     .document_type::<RawDocument<RawFilingIndex>>()
    ///     .root("data")
    ///     .build();
    /// let instance_repository = FilesystemRepository::builder()
    ///     .writer()
    ///     .document_type::<RawDocument<RawXbrlInstance>>()
    ///     .root("data")
    ///     .build();
    /// let labels_repository = FilesystemRepository::builder()
    ///     .writer()
    ///     .document_type::<RawDocument<RawXbrlLabels>>()
    ///     .root("data")
    ///     .build();
    ///
    /// let extract_state = ExtractFilingSuperState::builder()
    ///     .filing_ref(filing_ref)
    ///     .sec_client(SecClient::default())
    ///     .execution_id(Uuid::nil())
    ///     .filing_index_repository(filing_index_repository)
    ///     .instance_repository(instance_repository)
    ///     .labels_repository(labels_repository)
    ///     .build();
    /// ```
    ///
    /// Each repository must store its own source. So a builder with the instance and label
    /// repositories swapped has no `build` method, and this does not compile:
    ///
    /// ```compile_fail,E0271,E0599
    /// use chrono::NaiveDate;
    /// use sec::implementations::states::extract::filing::ExtractFilingSuperState;
    /// use sec::shared::cik::Cik;
    /// use sec::shared::filing_ref::FilingRef;
    /// use sec::shared::financial::accession_number::AccessionNumber;
    /// use sec::shared::financial::form::{Form, ParsedForm};
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::raw_document::RawDocument;
    /// use sec::shared::raw_source::{RawFilingIndex, RawXbrlInstance, RawXbrlLabels};
    /// use storage::implementations::repository::filesystem::{FilesystemRepository, IngestionMode};
    /// use uuid::Uuid;
    ///
    /// let filing_ref = FilingRef::new(
    ///     Cik::new("3545").expect("The CIK is valid"),
    ///     AccessionNumber::new("0000003545-26-000047").expect("The accession number is valid"),
    ///     ParsedForm::new(Form::TenQ, false),
    ///     NaiveDate::from_ymd_opt(2026, 9, 9).expect("The date exists"),
    /// );
    /// let filing_index_repository = FilesystemRepository::builder()
    ///     .writer()
    ///     .document_type::<RawDocument<RawFilingIndex>>()
    ///     .root("data")
    ///     .build();
    /// let instance_repository = FilesystemRepository::builder()
    ///     .writer()
    ///     .document_type::<RawDocument<RawXbrlInstance>>()
    ///     .root("data")
    ///     .build();
    /// let labels_repository = FilesystemRepository::builder()
    ///     .writer()
    ///     .document_type::<RawDocument<RawXbrlLabels>>()
    ///     .root("data")
    ///     .build();
    ///
    /// let extract_state = ExtractFilingSuperState::builder()
    ///     .filing_ref(filing_ref)
    ///     .sec_client(SecClient::default())
    ///     .execution_id(Uuid::nil())
    ///     .ingestion_mode(IngestionMode::Incremental)
    ///     .filing_index_repository(filing_index_repository)
    ///     .instance_repository(labels_repository)
    ///     .labels_repository(instance_repository)
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> ExtractFilingSuperState<PrepareSecRequest<FilingIndex>, X, I, L, C> {
        let entry_state = PrepareSecRequest::new(
            PrepareSecRequestInput::new(self.filing_ref.clone()),
            PrepareSecRequestContext::new(self.filing_ref.clone()),
        );

        ExtractFilingSuperState {
            current_state: entry_state,
            input: ExtractFilingSuperStateData,
            output: None,
            context: ExtractFilingSuperStateContext::new(
                self.filing_ref,
                self.sec_client,
                self.execution_id,
                self.ingestion_mode,
                self.filing_index_repository,
                self.instance_repository,
                self.labels_repository,
            )
            .with_inputs(self.inputs),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use storage::WriteError;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use crate::tests::fixtures::sample_write_repository::{
        FakeFilingIndexRepository, FakeXbrlInstanceRepository, FakeXbrlLabelsRepository,
    };

    type EmptyBuilder = ExtractFilingSuperStateBuilder<
        NoFilingRef,
        NoSecClient,
        NoExecutionId,
        NoIngestionMode,
        NoFilingIndexRepository<FakeFilingIndexRepository>,
        NoInstanceRepository<FakeXbrlInstanceRepository>,
        NoLabelsRepository<FakeXbrlLabelsRepository>,
    >;

    type TestSuperState = ExtractFilingSuperState<
        PrepareSecRequest<FilingIndex>,
        FakeFilingIndexRepository,
        FakeXbrlInstanceRepository,
        FakeXbrlLabelsRepository,
        RoutingSecClient,
    >;

    const EXECUTION_ID: Uuid = Uuid::from_u128(7);

    fn filing_ref() -> FilingRef {
        FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenQ, false),
            NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    fn sec_client() -> RoutingSecClient {
        RoutingSecClient::new().route("https://www.sec.gov/", 200, "text/xml", b"<xbrl/>")
    }

    fn filing_index_repository() -> FakeFilingIndexRepository {
        FakeFilingIndexRepository::rejecting(WriteError::conflicting_write("Sample index conflict"))
    }

    fn instance_repository() -> FakeXbrlInstanceRepository {
        FakeXbrlInstanceRepository::rejecting(WriteError::conflicting_write("Sample conflict"))
    }

    fn expected_super_state() -> TestSuperState {
        ExtractFilingSuperState {
            current_state: PrepareSecRequest::new(
                PrepareSecRequestInput::new(filing_ref()),
                PrepareSecRequestContext::new(filing_ref()),
            ),
            input: ExtractFilingSuperStateData,
            output: None,
            context: ExtractFilingSuperStateContext::new(
                filing_ref(),
                sec_client(),
                EXECUTION_ID,
                IngestionMode::Incremental,
                filing_index_repository(),
                instance_repository(),
                FakeXbrlLabelsRepository::new(),
            ),
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_extract_filing_super_state_builder() {
        implements_auto_traits::<EmptyBuilder>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_extract_filing_super_state_builder() {
        implements_value_traits::<EmptyBuilder>();
    }

    fn holds_no_field(_: EmptyBuilder) {}
    #[test]
    fn should_hold_no_field_when_starting_a_builder() {
        holds_no_field(ExtractFilingSuperState::builder());
    }

    #[test]
    fn should_build_a_super_state_at_the_folder_index_request_when_setting_every_field() {
        let builder = ExtractFilingSuperState::builder()
            .filing_ref(filing_ref())
            .sec_client(sec_client())
            .execution_id(EXECUTION_ID)
            .ingestion_mode(IngestionMode::Incremental)
            .filing_index_repository(filing_index_repository())
            .instance_repository(instance_repository());

        let expected_result = expected_super_state();

        let result = builder
            .labels_repository(FakeXbrlLabelsRepository::new())
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_carry_the_inputs_into_the_context_when_they_are_set() {
        let inputs = vec![InputDocument::new(
            storage::implementations::repository::filesystem::DocumentKey::new(
                "sec/daily-index/form.20260909.idx",
            )
            .expect("Given a hardcoded valid relative path, the key should always build"),
            storage::Sha256Digest::of(b"index"),
        )];
        let builder = ExtractFilingSuperState::builder()
            .filing_ref(filing_ref())
            .sec_client(sec_client())
            .execution_id(EXECUTION_ID)
            .ingestion_mode(IngestionMode::Incremental)
            .filing_index_repository(filing_index_repository())
            .instance_repository(instance_repository())
            .labels_repository(FakeXbrlLabelsRepository::new());

        let expected_result = inputs.clone();

        let result = builder.inputs(inputs).build().context.inputs;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_same_super_state_when_setting_the_fields_in_reverse_order() {
        let builder = ExtractFilingSuperState::builder()
            .labels_repository(FakeXbrlLabelsRepository::new())
            .instance_repository(instance_repository())
            .filing_index_repository(filing_index_repository())
            .ingestion_mode(IngestionMode::Incremental)
            .execution_id(EXECUTION_ID)
            .sec_client(sec_client());

        let expected_result = expected_super_state();

        let result = builder.filing_ref(filing_ref()).build();

        assert_eq!(result, expected_result);
    }
}
