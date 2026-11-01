//! # Extract Filing Super-State
//!
//! Provides the [`ExtractFilingSuperState`], which downloads the XBRL instance and the label file
//! of one filing and writes them to the store.
//!
//! The super-state owns the [`FilingRef`], the shared SEC client, the `execution_id`, and the two
//! storage repositories.

use std::fmt::{self, Display, Formatter};

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::StateMachine as SMStateMachine;
use uuid::Uuid;

use crate::error::State as StateError;
use crate::prelude::*;
use crate::shared::filing_ref::FilingRef;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::{RawXbrlInstance, RawXbrlLabels};

/// Input and output data for the [`ExtractFilingSuperState`].
///
/// A placeholder with unit update semantics: the super-state delegates all real data handling to
/// its current inner state, so it carries no data of its own.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ExtractFilingSuperStateData;

impl SMStateData for ExtractFilingSuperStateData {
    type UpdateType = ();
    fn state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, (): Self::UpdateType) {}
}

impl StateData for ExtractFilingSuperStateData {
    fn update_state(&mut self, (): Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

/// Context for the [`ExtractFilingSuperState`].
///
/// Holds the [`FilingRef`], the SEC client of type `C`, the `execution_id`, the repository `I`
/// of the instance, and the repository `L` of the label file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractFilingSuperStateContext<I, L, C> {
    pub(crate) filing_ref: FilingRef,
    pub(crate) sec_client: C,
    pub(crate) execution_id: Uuid,
    pub(crate) instance_repository: I,
    pub(crate) labels_repository: L,
}

impl<I, L, C> ExtractFilingSuperStateContext<I, L, C> {
    /// Creates a new [`ExtractFilingSuperStateContext`] from the [`FilingRef`], the SEC client,
    /// the `execution_id`, and the two storage repositories.
    #[must_use]
    pub const fn new(
        filing_ref: FilingRef,
        sec_client: C,
        execution_id: Uuid,
        instance_repository: I,
        labels_repository: L,
    ) -> Self {
        Self {
            filing_ref,
            sec_client,
            execution_id,
            instance_repository,
            labels_repository,
        }
    }
}

impl<I, L, C> SMContext for ExtractFilingSuperStateContext<I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    type UpdateType = ();
    fn context(&self) -> &Self {
        self
    }
    fn update_context(&mut self, (): Self::UpdateType) {}
}

impl<I, L, C> Context for ExtractFilingSuperStateContext<I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    fn max_retries(&self) -> u32 {
        0
    }
}

/// Hierarchical super-state that extracts the XBRL instance and the label file of one filing.
///
/// Wraps the active state in its `S` type parameter. Its context holds the [`FilingRef`], the SEC
/// client of type `C`, the `execution_id`, and the repositories `I` and `L`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractFilingSuperState<
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait = SecClient,
> {
    current_state: S,
    input: ExtractFilingSuperStateData,
    output: Option<ExtractFilingSuperStateData>,
    context: ExtractFilingSuperStateContext<I, L, C>,
}

impl<S, I, L, C> Display for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Extract Filing SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }
}

impl<S, I, L, C> SMState for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    type InputData = ExtractFilingSuperStateData;
    type OutputData = ExtractFilingSuperStateData;
    type Context = ExtractFilingSuperStateContext<I, L, C>;

    fn state_name(&self) -> impl ToString {
        self.to_string()
    }
    fn input_data(&self) -> &Self::InputData {
        &self.input
    }
    /// Blocking wrapper around [`compute_output_data_async`](crate::traits::state_machine::state::State::compute_output_data_async).
    ///
    /// # Panics
    ///
    /// - Panics if the async computation returns an error.
    /// - Panics if called inside a `current_thread` runtime, because `block_in_place` requires a
    ///   multi-thread runtime.
    fn compute_output_data(&mut self) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(self.current_state.compute_output_data_async())
            })
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.current_state.compute_output_data_async())
        };

        if let Err(e) = result {
            let state_err: StateError = e.into();
            panic!("compute_output_data failed: {state_err}")
        }
    }
    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
    fn context_data(&self) -> &Self::Context {
        &self.context
    }
}

#[async_trait]
impl<S, I, L, C> State for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.current_state
            .compute_output_data_async()
            .await
            .map_err(Into::into)
    }
}

impl<S, I, L, C> SMStateMachine<S> for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    fn current_state(&self) -> &S {
        &self.current_state
    }
    fn current_state_mut(&mut self) -> &mut S {
        &mut self.current_state
    }
    fn run(&mut self) {
        unimplemented!(
            "SEC state machines are async-only. \
             Use into_stream or drive states manually"
        )
    }
    fn advance_state(&mut self) {
        unimplemented!(
            "SEC state machines are async-only. \
             Use into_stream or drive states manually"
        )
    }
}

impl<S, I, L, C> StateMachine<S> for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
}

impl<S, I, L, C> SMSuperState<S> for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
}

impl<S, I, L, C> SuperState<S> for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::implementations::states::extract::prepare_sec_request::{
        PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
    };
    use crate::shared::cik::Cik;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::request::implementations::sec_request::FilingIndex;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use crate::tests::fixtures::sample_write_repository::{
        FakeXbrlInstanceRepository, FakeXbrlLabelsRepository,
    };

    type TestSuperState<S> = ExtractFilingSuperState<
        S,
        FakeXbrlInstanceRepository,
        FakeXbrlLabelsRepository,
        RoutingSecClient,
    >;

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

    fn super_state() -> TestSuperState<PrepareSecRequest<FilingIndex>> {
        ExtractFilingSuperState {
            current_state: PrepareSecRequest::new(
                PrepareSecRequestInput::new(filing_ref()),
                PrepareSecRequestContext::new(filing_ref()),
            ),
            input: ExtractFilingSuperStateData,
            output: None,
            context: ExtractFilingSuperStateContext::new(
                filing_ref(),
                RoutingSecClient::new(),
                Uuid::from_u128(7),
                FakeXbrlInstanceRepository::new(),
                FakeXbrlLabelsRepository::new(),
            ),
        }
    }

    #[test]
    #[expect(
        clippy::redundant_clone,
        reason = "False positive: `state_name` returns `impl ToString`, which is a `String` here. Clippy flags `.to_string()` as a clone, but removing it does not compile"
    )]
    fn should_return_super_state_name_with_current_state_when_in_prepare_sec_request_state() {
        let super_state = super_state();

        let expected_result = "Extract Filing SuperState (Current: Prepare SEC Request)";

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_zero_max_retries_for_extract_filing_super_state_context() {
        let super_state = super_state();

        let expected_result = 0;

        let result = super_state.context_data().max_retries();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_extract_filing_super_state() {
        implements_auto_traits::<TestSuperState<PrepareSecRequest<FilingIndex>>>();
    }
}
