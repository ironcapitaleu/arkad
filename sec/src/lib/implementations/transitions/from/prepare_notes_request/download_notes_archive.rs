//! # Transition: `PrepareNotesRequest` → `DownloadNotesArchive`
//!
//! Implements the [`TryFrom`] conversion producing a [`DownloadNotesArchive`] from a
//! [`PrepareNotesRequest`] and an SEC client.
//!
//! It moves the request from the output of the source state, the Notes period from its input, and
//! the `execution_id` and the storage repository from its context. The source state holds no
//! client, so the caller passes the one from the state machine context.
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::download_notes_archive::constants::STATE_NAME as DOWNLOAD_NOTES_ARCHIVE;
use crate::implementations::states::extract::download_notes_archive::{
    DownloadNotesArchive, DownloadNotesArchiveContext, DownloadNotesArchiveInput,
};
use crate::implementations::states::extract::prepare_notes_request::PrepareNotesRequest;
use crate::implementations::states::extract::prepare_notes_request::constants::STATE_NAME as PREPARE_NOTES_REQUEST;

impl<R, C> TryFrom<(PrepareNotesRequest<R>, C)> for DownloadNotesArchive<R, C> {
    type Error = TransitionError;

    /// Builds a [`DownloadNotesArchive`] from a computed [`PrepareNotesRequest`] and the client
    /// that sends the download.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    fn try_from((state, sec_client): (PrepareNotesRequest<R>, C)) -> Result<Self, TransitionError> {
        let (input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(PREPARE_NOTES_REQUEST, DOWNLOAD_NOTES_ARCHIVE)
        })?;

        let new_input = DownloadNotesArchiveInput::new(output_data.request);
        let new_context = DownloadNotesArchiveContext::new(
            input.notes_period,
            context.execution_id,
            context.repository,
            sec_client,
        );

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use storage::WriteError;
    use uuid::Uuid;

    use super::*;
    use crate::implementations::states::extract::prepare_notes_request::{
        PrepareNotesRequestContext, PrepareNotesRequestInput,
    };
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::implementations::sec_request::SecRequest;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingReadySecClient;
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

    fn sec_client() -> AlwaysRespondingReadySecClient {
        AlwaysRespondingReadySecClient::new(404, Vec::new(), Vec::new())
    }

    fn prepare_notes_request() -> PrepareNotesRequest<FakeNotesRepository> {
        PrepareNotesRequest::new(
            PrepareNotesRequestInput::new(sample_period()),
            PrepareNotesRequestContext::new(SAMPLE_EXECUTION_ID, rejecting_repository()),
        )
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

        let result = DownloadNotesArchive::try_from((state, sec_client()));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = prepare_notes_request();

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(PREPARE_NOTES_REQUEST, DOWNLOAD_NOTES_ARCHIVE),
        ));

        let result = DownloadNotesArchive::try_from((state, sec_client()));

        assert_eq!(result, expected_result);
    }
}
