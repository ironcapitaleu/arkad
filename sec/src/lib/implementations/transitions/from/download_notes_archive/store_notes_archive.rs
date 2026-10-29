//! # Transition: `DownloadNotesArchive` → `StoreNotesArchive`
//!
//! Implements the [`TryFrom`] conversion producing a [`StoreNotesArchive`] from a
//! [`DownloadNotesArchive`].
//!
//! It moves the [`Arc`](std::sync::Arc) of the response from the output of the source state, the
//! requested URL from its [`SecRequest`], the `User-Agent` from its client, and the Notes period,
//! the `execution_id`, and the storage repository from its context. It keeps no clone of the
//! [`Arc`](std::sync::Arc), so [`StoreNotesArchive`] can take the bytes out of it.
//!
//! # Errors
//!
//! Returns [`TransitionError`] when the source state has no computed output
//! ([`MissingOutput`](crate::error::state_machine::transition::MissingOutput)), or when its
//! request names a different period from its context
//! ([`FailedContextConversion`](crate::error::state_machine::transition::FailedContextConversion)).

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::download_notes_archive::constants::STATE_NAME as DOWNLOAD_NOTES_ARCHIVE;
use crate::implementations::states::extract::download_notes_archive::{
    DownloadNotesArchive, NotesFileClient,
};
use crate::implementations::states::extract::store_notes_archive::constants::STATE_NAME as STORE_NOTES_ARCHIVE;
use crate::implementations::states::extract::store_notes_archive::{
    StoreNotesArchive, StoreNotesArchiveContext, StoreNotesArchiveInput,
};
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::url::Url;

impl<R, C: NotesFileClient> TryFrom<DownloadNotesArchive<R, C>> for StoreNotesArchive<R> {
    type Error = TransitionError;

    /// Builds a [`StoreNotesArchive`] from a computed [`DownloadNotesArchive`].
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    ///
    /// Returns [`TransitionError::FailedContextConversion`] if the request of `state` names a
    /// different period from its context. The stored key follows the context period, so a
    /// mismatch stores a file under the wrong period.
    fn try_from(state: DownloadNotesArchive<R, C>) -> Result<Self, TransitionError> {
        let (input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(DOWNLOAD_NOTES_ARCHIVE, STORE_NOTES_ARCHIVE)
        })?;

        let context_request = SecRequest::builder()
            .notes_file()
            .period(context.notes_period)
            .build();
        if input.request != context_request {
            let reason = format!(
                "The request targets '{}', but the context names period '{}'",
                input.request.inner.url(),
                context.notes_period
            );
            return Err(transition::FailedContextConversion::new(
                DOWNLOAD_NOTES_ARCHIVE,
                STORE_NOTES_ARCHIVE,
                reason,
            )
            .into());
        }

        let requested_url = Url::from(input.request.inner.url().clone());
        let user_agent = context.sec_client.user_agent().clone();
        let new_input = StoreNotesArchiveInput::new(
            output_data.response,
            context.notes_period,
            requested_url,
            user_agent,
        );
        let new_context = StoreNotesArchiveContext::new(context.execution_id, context.repository);

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pretty_assertions::assert_eq;
    use storage::WriteError;
    use uuid::Uuid;

    use super::*;
    use crate::implementations::states::extract::download_notes_archive::{
        DownloadNotesArchiveContext, DownloadNotesArchiveInput,
    };
    use crate::prelude::*;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::response::implementations::sec_notes_file_response::SecNotesFileResponse;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingReadySecClient;
    use crate::tests::fixtures::sample_write_repository::FakeNotesRepository;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);

    const REQUESTED_URL: &str =
        "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2024q1_notes.zip";

    type TestDownload = DownloadNotesArchive<FakeNotesRepository, AlwaysRespondingReadySecClient>;

    fn period(raw_period: &str) -> NotesPeriod {
        NotesPeriod::new(raw_period)
            .expect("Given a hardcoded published period, parsing should always succeed")
    }

    fn rejecting_repository() -> FakeNotesRepository {
        FakeNotesRepository::rejecting(WriteError::conflicting_write("Sample conflict"))
    }

    fn sec_client() -> AlwaysRespondingReadySecClient {
        AlwaysRespondingReadySecClient::new(
            200,
            vec![("content-type", "application/zip".to_string())],
            SAMPLE_NOTES_ZIP.to_vec(),
        )
    }

    fn download_without_output(request_period: &str) -> TestDownload {
        let request = SecRequest::builder()
            .notes_file()
            .period(period(request_period))
            .build();
        DownloadNotesArchive::new(
            DownloadNotesArchiveInput::new(request),
            DownloadNotesArchiveContext::new(
                period("2024q1"),
                SAMPLE_EXECUTION_ID,
                rejecting_repository(),
                sec_client(),
            ),
        )
    }

    async fn download_with_output(request_period: &str) -> TestDownload {
        let mut state = download_without_output(request_period);
        state
            .compute_output_data_async()
            .await
            .expect("Given status 200, a ZIP content type, and the ZIP fixture, the download should always succeed");
        state
    }

    async fn store_notes_archive() -> StoreNotesArchive<FakeNotesRepository> {
        StoreNotesArchive::try_from(download_with_output("2024q1").await)
            .expect("Given a source state with output, the transition should always succeed")
    }

    #[tokio::test]
    async fn should_hand_over_the_only_reference_to_the_response_when_source_has_output() {
        let state = store_notes_archive().await;

        let expected_result = Some(1);

        let result = state.input_data().response.as_ref().map(Arc::strong_count);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_move_response_when_source_has_output() {
        let state = store_notes_archive().await;

        let expected_result = Some(SAMPLE_NOTES_ZIP);

        let result = state
            .input_data()
            .response
            .as_deref()
            .map(SecNotesFileResponse::body);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_requested_url_from_sec_request_when_source_has_output() {
        let state = store_notes_archive().await;

        let expected_result = REQUESTED_URL;

        let result = state.input_data().requested_url.as_str();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_user_agent_from_sec_client_when_source_has_output() {
        let state = store_notes_archive().await;

        let expected_result = UserAgent::new("Fake Notes Client fake@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");

        let result = state.input_data().user_agent.clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_notes_period_from_context_when_source_has_output() {
        let state = store_notes_archive().await;

        let expected_result = period("2024q1");

        let result = state.input_data().notes_period;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_move_execution_id_and_repository_when_source_has_output() {
        let state = store_notes_archive().await;

        let expected_result =
            &StoreNotesArchiveContext::new(SAMPLE_EXECUTION_ID, rejecting_repository());

        let result = state.context_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_context_conversion_when_request_names_another_period() {
        let state = download_with_output("2025_10").await;

        let expected_result = Err(TransitionError::FailedContextConversion(
            transition::FailedContextConversion::new(
                DOWNLOAD_NOTES_ARCHIVE,
                STORE_NOTES_ARCHIVE,
                "The request targets 'https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2025_10_notes.zip', but the context names period '2024q1'",
            ),
        ));

        let result = StoreNotesArchive::try_from(state);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = download_without_output("2024q1");

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(DOWNLOAD_NOTES_ARCHIVE, STORE_NOTES_ARCHIVE),
        ));

        let result = StoreNotesArchive::try_from(state);

        assert_eq!(result, expected_result);
    }
}
