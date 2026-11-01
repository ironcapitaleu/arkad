//! # Store SEC Response State
//!
//! Provides the [`StoreSecResponse`] state, which writes the response to a request of any
//! [`StoredAs`] kind to the store as one raw document.
//!
//! The kind builds the document and the output, so a new kind adds a [`StoredAs`] impl instead of
//! a new state. The state writes through the [`WriteRepository`](storage::WriteRepository) port
//! that its context holds, so it names no concrete backend.
//!
//! ## Modules
//!
//! - [`context`]: The [`StoreSecResponseContext`] carried alongside the state.
//! - [`data`]: The [`StoreSecResponseInput`] data type.
//!
//! ## Usage
//!
//! The repository must store the source of the kind:
//!
//! ```rust
//! use sec::implementations::states::extract::store_sec_response::StoreSecResponse;
//! use sec::shared::raw_document::RawDocument;
//! use sec::shared::raw_source::RawCompanyFacts;
//! use sec::shared::request::implementations::sec_request::CompanyFacts;
//! use sec::traits::state_machine::state::State;
//! use storage::implementations::repository::filesystem::{FilesystemRepository, ReadWrite};
//!
//! const fn implements_state<S: State>() {}
//!
//! implements_state::<
//!     StoreSecResponse<CompanyFacts, FilesystemRepository<ReadWrite, RawDocument<RawCompanyFacts>>>,
//! >();
//! ```
//!
//! A repository of another source does not compile:
//!
//! ```compile_fail,E0271
//! use sec::implementations::states::extract::store_sec_response::StoreSecResponse;
//! use sec::shared::raw_document::RawDocument;
//! use sec::shared::raw_source::RawNotesArchive;
//! use sec::shared::request::implementations::sec_request::CompanyFacts;
//! use sec::traits::state_machine::state::State;
//! use storage::implementations::repository::filesystem::{FilesystemRepository, ReadWrite};
//!
//! const fn implements_state<S: State>() {}
//!
//! implements_state::<
//!     StoreSecResponse<CompanyFacts, FilesystemRepository<ReadWrite, RawDocument<RawNotesArchive>>>,
//! >();
//! ```
//!
//! ## See Also
//!
//! - [`crate::shared::raw_document`]: The [`RawDocument`](crate::shared::raw_document::RawDocument)
//!   type the state writes.
//! - [`crate::traits::state_machine::state::State`]: The trait implemented by [`StoreSecResponse`].

use std::fmt;

use async_trait::async_trait;
use chrono::Utc;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::error::state_machine::state::FailedDocumentWrite;
use crate::shared::build_info::producer;
use crate::shared::document_storage::FailedDocumentStorage;
use crate::shared::raw_repository::RawRepository;
use crate::shared::request::implementations::sec_request::StoredAs;
use crate::shared::run_fields::RunFields;
use crate::traits::state_machine::state::State;

pub mod context;
pub mod data;

pub use context::StoreSecResponseContext;
pub use data::StoreSecResponseInput;

/// Writes the response to a request of kind `E` to the store as one raw document.
///
/// [`StoredAs::store`] builds the document and the output. The state writes the document through
/// the repository of type `R` and keeps the output once the write succeeds. The metadata names
/// [`StoredAs::STATE_NAME`] as the producer and records the ingestion mode of the input.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
#[serde(bound(serialize = "R: Serialize"))]
pub struct StoreSecResponse<E: StoredAs, R> {
    input: StoreSecResponseInput<E>,
    context: StoreSecResponseContext<E, R>,
    output: Option<E::Output>,
}

impl<E: StoredAs, R> StoreSecResponse<E, R> {
    /// Creates a new [`StoreSecResponse`] state from its input and context, with no output
    /// computed yet.
    #[must_use]
    pub const fn new(
        input: StoreSecResponseInput<E>,
        context: StoreSecResponseContext<E, R>,
    ) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        StoreSecResponseInput<E>,
        Option<E::Output>,
        StoreSecResponseContext<E, R>,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl<E: StoredAs, R: RawRepository<E::Source>> State for StoreSecResponse<E, R> {
    /// Writes the raw document to the repository and stores the output of the kind.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::FailedDocumentWrite`] when the repository rejects the write.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let (document, output) = E::store(
            &self.input.response,
            &self.context.request_input,
            &self.input.requested_url,
            &self.input.user_agent,
            RunFields {
                produced_at: Utc::now(),
                execution_id: self.context.execution_id,
                producer: producer(E::STATE_NAME),
                ingestion_mode: self.input.ingestion_mode,
            },
        );

        self.context
            .repository
            .persist(document)
            .await
            .map_err(|e| FailedDocumentWrite::new(E::STATE_NAME, FailedDocumentStorage::from(e)))?;
        self.output = Some(output);
        Ok(())
    }
}

impl<E: StoredAs, R: RawRepository<E::Source>> SMState for StoreSecResponse<E, R> {
    type InputData = StoreSecResponseInput<E>;
    type OutputData = E::Output;
    type Context = StoreSecResponseContext<E, R>;

    /// Returns the human-readable name of this state.
    fn state_name(&self) -> impl ToString {
        E::STATE_NAME
    }

    /// Blocking wrapper around [`compute_output_data_async`](crate::traits::state_machine::state::State::compute_output_data_async).
    ///
    /// Detects whether a tokio runtime is available and runs the async computation
    /// synchronously. This allows SEC states to be used as regular `SMState` implementations.
    ///
    /// # Panics
    /// - Panics if the async computation returns an error.
    /// - Panics if called inside a `current_thread` runtime (`block_in_place`
    ///   requires a multi-thread runtime). All binaries and tests in this project
    ///   use `flavor = "multi_thread"`.
    fn compute_output_data(&mut self) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(self.compute_output_data_async()))
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.compute_output_data_async())
        };

        if let Err(e) = result {
            let state_err: StateError = e;
            panic!("compute_output_data failed: {state_err}")
        }
    }

    fn context_data(&self) -> &Self::Context {
        &self.context
    }

    fn input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl<E: StoredAs, R: RawRepository<E::Source>> fmt::Display for StoreSecResponse<E, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}
