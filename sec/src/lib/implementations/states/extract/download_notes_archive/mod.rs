//! # Download Notes Archive State
//!
//! Provides the [`DownloadNotesArchive`] state, which downloads one Notes file through the
//! [`SecClient`] of its context.
//!
//! The output holds the whole file. The state does not cap the number of files in flight.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`DownloadNotesArchiveContext`] carried alongside the state.
//! - [`data`]: The [`DownloadNotesArchiveInput`] and [`DownloadNotesArchiveOutput`] data types.
//!
//! ## Usage
//!
//! ```no_run
//! use sec::implementations::states::extract::download_notes_archive::*;
//! use sec::prelude::*;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use sec::shared::notes_period::NotesPeriod;
//! use sec::shared::request::implementations::sec_request::SecRequest;
//! use uuid::Uuid;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let period = NotesPeriod::new("2024q1")
//!     .expect("Given a hardcoded published quarter, parsing should always succeed");
//! let request = SecRequest::builder().notes_file().period(period).build();
//! let input = DownloadNotesArchiveInput::new(request);
//! let context = DownloadNotesArchiveContext::new(Uuid::new_v4(), (), SecClient::default());
//! let mut state = DownloadNotesArchive::new(input, context);
//! state
//!     .compute_output_data_async()
//!     .await
//!     .expect("The live SEC download should succeed");
//!
//! let _body = state
//!     .output_data()
//!     .expect("Given a successful computation, the output should always be present")
//!     .response
//!     .body();
//! # }
//! ```

use std::fmt;

use async_trait::async_trait;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::error::state_machine::state::FailedRequestExecution;
use crate::implementations::states::extract::notes_repository::NotesRepository;
use crate::shared::http_client::implementations::sec_client::{SecClient, download_notes_file};
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::{DownloadNotesArchiveContext, NotesFileClient};
pub use data::DownloadNotesArchiveInput;
pub use data::DownloadNotesArchiveOutput;

/// Downloads one Notes file and outputs it as a
/// [`SecNotesFileResponse`](crate::shared::response::implementations::sec_notes_file_response::SecNotesFileResponse).
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct DownloadNotesArchive<R, C = SecClient> {
    input: DownloadNotesArchiveInput,
    context: DownloadNotesArchiveContext<R, C>,
    output: Option<DownloadNotesArchiveOutput>,
}

impl<R, C> DownloadNotesArchive<R, C> {
    /// Creates a new [`DownloadNotesArchive`] state with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::download_notes_archive::*;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::notes_period::NotesPeriod;
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    /// use state_maschine::prelude::State as SMState;
    /// use uuid::Uuid;
    ///
    /// let period = NotesPeriod::new("2024q1")
    ///     .expect("Given a hardcoded published quarter, parsing should always succeed");
    /// let input = DownloadNotesArchiveInput::new(SecRequest::builder().notes_file().period(period).build());
    /// let context = DownloadNotesArchiveContext::new(Uuid::nil(), (), SecClient::default());
    /// let state = DownloadNotesArchive::new(input, context);
    ///
    /// let expected_result = false;
    ///
    /// let result = state.has_output_data_been_computed();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(
        input: DownloadNotesArchiveInput,
        context: DownloadNotesArchiveContext<R, C>,
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
        DownloadNotesArchiveInput,
        Option<DownloadNotesArchiveOutput>,
        DownloadNotesArchiveContext<R, C>,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl<R: NotesRepository, C: NotesFileClient> State for DownloadNotesArchive<R, C> {
    /// Downloads the Notes file of the input request and stores it as output.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::FailedRequestExecution`] if the request fails at the network level,
    /// or if the response does not have status 200, an accepted content type, and a body length
    /// that matches the `Content-Length` header.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let request = self.input.request.clone().into_inner();
        let response = download_notes_file(&self.context.sec_client, request)
            .await
            .map_err(|e| FailedRequestExecution::new(STATE_NAME, e))?;
        self.output = Some(DownloadNotesArchiveOutput::new(response));
        Ok(())
    }
}

impl<R: NotesRepository, C: NotesFileClient> SMState for DownloadNotesArchive<R, C> {
    type InputData = DownloadNotesArchiveInput;
    type OutputData = DownloadNotesArchiveOutput;
    type Context = DownloadNotesArchiveContext<R, C>;

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

impl<R: NotesRepository, C: NotesFileClient> fmt::Display for DownloadNotesArchive<R, C> {
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
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::shared::content_type::ContentType;
    use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::implementations::sec_request::SecRequest;
    use crate::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use crate::shared::status_code::StatusCode;
    use crate::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingReadySecClient;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    type TestState = DownloadNotesArchive<FakeWriteRepository, AlwaysRespondingReadySecClient>;

    fn test_state(status: u16, content_type: &str) -> TestState {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed");
        let request = SecRequest::builder().notes_file().period(period).build();
        let sec_client = AlwaysRespondingReadySecClient::new(
            status,
            vec![
                ("content-type", content_type.to_string()),
                ("content-length", SAMPLE_NOTES_ZIP.len().to_string()),
            ],
            SAMPLE_NOTES_ZIP.to_vec(),
        );
        DownloadNotesArchive::new(
            DownloadNotesArchiveInput::new(request),
            DownloadNotesArchiveContext::new(Uuid::nil(), FakeWriteRepository::new(), sec_client),
        )
    }

    fn failed_download(reason: ErrorReason) -> StateError {
        FailedRequestExecution::new(
            STATE_NAME,
            FailedSecRequest::from(InvalidSecResponse::new(reason)),
        )
        .into()
    }

    #[test]
    fn should_return_state_name_for_download_notes_archive() {
        let state = test_state(200, "application/zip");

        let expected_result = "Download Notes Archive";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let state = test_state(200, "application/zip");

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_zip_body_unchanged_when_the_download_succeeds() {
        let mut state = test_state(200, "application/zip");

        let expected_result = Some(SAMPLE_NOTES_ZIP);

        state
            .compute_output_data_async()
            .await
            .expect("Given status 200, a ZIP content type and a matching length, the download should always succeed");
        let result = state.output_data().map(|output| output.response.body());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_accept_an_octet_stream_when_the_download_succeeds() {
        let mut state = test_state(200, "application/octet-stream");

        let expected_result = true;

        let result = state.compute_output_data_async().await.is_ok();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_request_execution_when_the_notes_file_is_not_found() {
        let mut state = test_state(404, "application/zip");

        let expected_result = Err(failed_download(ErrorReason::InvalidStatusCode {
            status_code: StatusCode::NotFound,
        }));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_request_execution_when_the_content_type_is_html() {
        let mut state = test_state(200, "text/html");

        let expected_result = Err(failed_download(ErrorReason::InvalidContentType {
            content_type: ContentType::Html,
        }));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_the_output_empty_when_the_download_fails() {
        let mut state = test_state(404, "application/zip");

        let expected_result = false;

        let _ = state.compute_output_data_async().await;
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let mut state = test_state(200, "application/zip");

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_output_when_calling_sync_compute_outside_tokio_runtime() {
        let mut state = test_state(200, "application/zip");

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_download_notes_archive() {
        implements_value_traits::<DownloadNotesArchive<FakeWriteRepository>>();
    }
}
