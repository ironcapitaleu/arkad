//! # Store Daily Index State
//!
//! Provides the [`StoreDailyIndex`] state, which writes the downloaded daily index of one day to
//! the store and outputs its key and its text.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`StoreDailyIndexContext`] carried alongside the state.
//! - [`data`]: The [`StoreDailyIndexInput`] and [`StoreDailyIndexOutput`] data types.
//!
//! ## See Also
//!
//! - [`crate::shared::raw_document`]: The [`RawDocument`](crate::shared::raw_document::RawDocument)
//!   type the state writes.

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::implementations::states::extract::daily_index_repository::DailyIndexRepository;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::StoreDailyIndexContext;
pub use data::{StoreDailyIndexInput, StoreDailyIndexOutput, StoredDailyIndex};

/// Writes the downloaded daily index of one day to the store.
///
/// The store logic follows in a later change. Until then, computing the output panics.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct StoreDailyIndex<R> {
    input: StoreDailyIndexInput,
    context: StoreDailyIndexContext<R>,
    output: Option<StoreDailyIndexOutput>,
}

impl<R> StoreDailyIndex<R> {
    /// Creates a new [`StoreDailyIndex`] state with no output computed yet.
    #[must_use]
    pub const fn new(input: StoreDailyIndexInput, context: StoreDailyIndexContext<R>) -> Self {
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
        StoreDailyIndexInput,
        Option<StoreDailyIndexOutput>,
        StoreDailyIndexContext<R>,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl<R: DailyIndexRepository> State for StoreDailyIndex<R> {
    /// Panics until the store logic lands.
    ///
    /// # Panics
    ///
    /// Always panics, so a machine that wires this state fails at the state, not at a later
    /// transition.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        // Deviation: a stub that panics. Part 2 of STA-370 replaces it with the store logic.
        unimplemented!("{STATE_NAME}: the store logic lands in part 2 of STA-370")
    }
}

impl<R: DailyIndexRepository> SMState for StoreDailyIndex<R> {
    type InputData = StoreDailyIndexInput;
    type OutputData = StoreDailyIndexOutput;
    type Context = StoreDailyIndexContext<R>;

    fn state_name(&self) -> impl ToString {
        STATE_NAME
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
            tokio::task::block_in_place(|| handle.block_on(self.compute_output_data_async()))
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.compute_output_data_async())
        };

        if let Err(e) = result {
            panic!("compute_output_data failed: {e}")
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

impl<R: DailyIndexRepository> fmt::Display for StoreDailyIndex<R> {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use storage::implementations::repository::filesystem::{FilesystemRepository, ReadWrite};
    use uuid::Uuid;

    use super::*;
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::index_date::IndexDate;
    use crate::shared::raw_document::RawDocument;
    use crate::shared::raw_source::RawDailyIndex;
    use crate::shared::response::SecResponse as SecResponseTrait;
    use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_write_repository::FakeDailyIndexRepository;

    type DailyIndexFilesystemRepository =
        FilesystemRepository<ReadWrite, RawDocument<RawDailyIndex>>;

    fn sample_response() -> Arc<SecResponse<ZipBody>> {
        let url: Url = "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261027.idx"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed");
        let response = SecResponse::<ZipBody>::from_parts(
            url,
            Headers::new(HashMap::new()),
            ContentType::OctetStream,
            StatusCode::Ok,
            b"Form Type".to_vec(),
        )
        .expect(
            "Given status 200 and an octet stream, building the response should always succeed",
        );
        Arc::new(response)
    }

    fn sample_state() -> StoreDailyIndex<FakeDailyIndexRepository> {
        let response = sample_response();
        let requested_url = response.url().clone();
        let user_agent = UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        let index_date = IndexDate::new("2026-10-27")
            .expect("Given a hardcoded past date, parsing should always succeed");
        StoreDailyIndex::new(
            StoreDailyIndexInput::new(Some(response), index_date, requested_url, user_agent),
            StoreDailyIndexContext::new(Uuid::nil(), FakeDailyIndexRepository::new()),
        )
    }

    #[test]
    fn should_return_state_name_for_store_daily_index() {
        let state = sample_state();

        let expected_result = "Store Daily Index";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let state = sample_state();

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_daily_index() {
        implements_value_traits::<StoreDailyIndex<FakeDailyIndexRepository>>();
    }

    const fn implements_state<T: State>() {}
    #[test]
    const fn should_implement_state_for_store_daily_index_of_a_filesystem_repository() {
        implements_state::<StoreDailyIndex<DailyIndexFilesystemRepository>>();
    }
}
