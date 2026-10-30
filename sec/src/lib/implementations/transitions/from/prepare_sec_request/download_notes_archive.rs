//! # Transition: `PrepareNotesRequest` → `DownloadNotesArchive`
//!
//! Implements the [`TryFrom`] conversion producing a [`DownloadNotesArchive`] from a
//! [`PrepareNotesRequest`], an SEC client, and an `execution_id`.
//!
//! It moves the request from the output of the source state, the Notes period from its input, and
//! the storage repository from its context. The source state holds no client and no `execution_id`, so the
//! caller passes the ones from the state machine context.
//!
//! The source state has the [`NotesFile`] kind. A prepared Company Facts request cannot become a Notes download, so this does not compile:
//!
//! ```compile_fail,E0277
//! use sec::implementations::states::extract::download_notes_archive::DownloadNotesArchive;
//! use sec::implementations::states::extract::prepare_sec_request::PrepareCompanyFactsRequest;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use uuid::Uuid;
//!
//! fn download_as_notes_file<R>(state: PrepareCompanyFactsRequest<R>) {
//!     let _next =
//!         DownloadNotesArchive::<R, SecClient>::try_from((state, SecClient::default(), Uuid::nil()));
//! }
//! ```
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output.

use uuid::Uuid;

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::download_notes_archive::constants::STATE_NAME as DOWNLOAD_NOTES_ARCHIVE;
use crate::implementations::states::extract::download_notes_archive::{
    DownloadNotesArchive, DownloadNotesArchiveContext, DownloadNotesArchiveInput,
};
use crate::implementations::states::extract::prepare_sec_request::PrepareNotesRequest;
use crate::shared::request::implementations::sec_request::{NotesFile, RequestKind};

impl<R, C> TryFrom<(PrepareNotesRequest<R>, C, Uuid)> for DownloadNotesArchive<R, C> {
    type Error = TransitionError;

    /// Builds a [`DownloadNotesArchive`] from a computed [`PrepareNotesRequest`], the client that
    /// sends the download, and the `execution_id` of the run.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    fn try_from(
        (state, sec_client, execution_id): (PrepareNotesRequest<R>, C, Uuid),
    ) -> Result<Self, TransitionError> {
        let (input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(NotesFile::PREPARE_STATE_NAME, DOWNLOAD_NOTES_ARCHIVE)
        })?;

        let new_input = DownloadNotesArchiveInput::new(output_data.request);
        let new_context = DownloadNotesArchiveContext::new(
            input.request_input,
            execution_id,
            context.repository,
            sec_client,
        );

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::State as SMState;
    use storage::WriteError;

    use super::*;
    use crate::implementations::states::extract::prepare_sec_request::{
        PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
    };
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::implementations::sec_request::SecRequest;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_write_repository::FakeNotesRepository;
    use crate::traits::state_machine::state::State;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);

    fn sample_period() -> NotesPeriod {
        NotesPeriod::new("2024q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed")
    }

    fn rejecting_repository() -> FakeNotesRepository {
        FakeNotesRepository::rejecting(WriteError::conflicting_write("Sample conflict"))
    }

    fn sec_client() -> AlwaysRespondingSecClient {
        AlwaysRespondingSecClient::new(404, Vec::new(), Vec::new())
    }

    fn prepare_notes_request() -> PrepareNotesRequest<FakeNotesRepository> {
        PrepareSecRequest::builder()
            .notes_file()
            .period(sample_period())
            .repository(rejecting_repository())
            .build()
    }

    #[tokio::test]
    async fn should_move_request_period_client_and_context_when_source_has_output() {
        let mut state = prepare_notes_request();
        state
            .compute_output_data_async()
            .await
            .expect("Given a validated period, preparing the request should always succeed");

        let expected_result = Ok(DownloadNotesArchive::new(
            DownloadNotesArchiveInput::new(
                SecRequest::builder()
                    .notes_file()
                    .period(sample_period())
                    .build(),
            ),
            DownloadNotesArchiveContext::new(
                sample_period(),
                SAMPLE_EXECUTION_ID,
                rejecting_repository(),
                sec_client(),
            ),
        ));

        let result = DownloadNotesArchive::try_from((state, sec_client(), SAMPLE_EXECUTION_ID));

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_period_from_input_when_input_and_context_periods_differ() {
        let input_period = NotesPeriod::new("2024q2")
            .expect("Given a hardcoded published quarter, parsing should always succeed");
        let mut state = PrepareSecRequest::new(
            PrepareSecRequestInput::<NotesFile>::new(input_period),
            PrepareSecRequestContext::new(sample_period(), rejecting_repository()),
        );
        state
            .compute_output_data_async()
            .await
            .expect("Given a validated period, preparing the request should always succeed");

        let expected_result = input_period;

        let result = DownloadNotesArchive::try_from((state, sec_client(), SAMPLE_EXECUTION_ID))
            .expect("Given a computed state, the transition should always succeed")
            .context_data()
            .notes_period;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = prepare_notes_request();

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new("Prepare Notes Request", DOWNLOAD_NOTES_ARCHIVE),
        ));

        let result = DownloadNotesArchive::try_from((state, sec_client(), SAMPLE_EXECUTION_ID));

        assert_eq!(result, expected_result);
    }
}
