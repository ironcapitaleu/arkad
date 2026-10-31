//! # Transition: `DownloadDailyIndex` → `StoreDailyIndex`
//!
//! Implements the [`TryFrom`] conversion producing a [`StoreDailyIndex`] from a
//! [`DownloadDailyIndex`], the `execution_id`, and the storage repository.
//!
//! It moves the [`Arc`](std::sync::Arc) of the response from the output of the source state, the
//! requested URL from its [`SecRequest`], the `User-Agent` from its client, and the date from its
//! context. A day without filings has no response, and the next state gets `None`. The source
//! state holds neither the `execution_id` nor a repository, so the caller passes the ones from the
//! state machine context.
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output.

use uuid::Uuid;

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::download_daily_index::DownloadDailyIndex;
use crate::implementations::states::extract::download_daily_index::constants::STATE_NAME as DOWNLOAD_DAILY_INDEX;
use crate::implementations::states::extract::store_daily_index::constants::STATE_NAME as STORE_DAILY_INDEX;
use crate::implementations::states::extract::store_daily_index::{
    StoreDailyIndex, StoreDailyIndexContext, StoreDailyIndexInput,
};
use crate::shared::http_client::SecClient as SecClientTrait;
#[cfg(doc)]
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::url::Url;

impl<R, C: SecClientTrait> TryFrom<(DownloadDailyIndex<C>, Uuid, R)> for StoreDailyIndex<R> {
    type Error = TransitionError;

    /// Builds a [`StoreDailyIndex`] from a computed [`DownloadDailyIndex`], the `execution_id`,
    /// and the storage repository.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    fn try_from(
        (state, execution_id, repository): (DownloadDailyIndex<C>, Uuid, R),
    ) -> Result<Self, TransitionError> {
        let (input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(DOWNLOAD_DAILY_INDEX, STORE_DAILY_INDEX)
        })?;

        let requested_url = Url::from(input.request.inner.url().clone());
        let user_agent = context.sec_client.user_agent().clone();
        let new_input = StoreDailyIndexInput::new(
            output_data.response,
            context.index_date,
            requested_url,
            user_agent,
        );
        let new_context = StoreDailyIndexContext::new(execution_id, repository);

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::implementations::states::extract::download_daily_index::{
        DownloadDailyIndexContext, DownloadDailyIndexInput,
    };
    use crate::prelude::*;
    use crate::shared::index_date::IndexDate;
    use crate::shared::request::implementations::sec_request::SecRequest;
    use crate::shared::response::SecResponse as SecResponseTrait;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use crate::tests::fixtures::sample_write_repository::FakeDailyIndexRepository;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);
    const INDEX_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261027.idx";
    const SUNDAY_INDEX_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261025.idx";
    const QTR4_LISTING_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/index.json";
    const INDEX_BODY: &[u8] = b"Form Type   Company Name\n10-K        SAMPLE CORP\n";

    fn index_date(raw_date: &str) -> IndexDate {
        IndexDate::new(raw_date)
            .expect("Given a hardcoded past date, validation should always succeed")
    }

    /// Returns a client that answers the index of 2026-10-27 with `INDEX_BODY`, and lists only
    /// that index for the quarter.
    fn sec_client() -> RoutingSecClient {
        RoutingSecClient::new()
            .route(INDEX_URL, 200, "application/octet-stream", INDEX_BODY)
            .route(
                QTR4_LISTING_URL,
                200,
                "application/json",
                br#"{"directory":{"item":[{"name":"form.20261027.idx"}]}}"#,
            )
    }

    fn download_without_output(raw_date: &str) -> DownloadDailyIndex<RoutingSecClient> {
        let date = index_date(raw_date);
        DownloadDailyIndex::new(
            DownloadDailyIndexInput::new(SecRequest::builder().daily_index().date(date).build()),
            DownloadDailyIndexContext::new(date, sec_client()),
        )
    }

    async fn store_daily_index(raw_date: &str) -> StoreDailyIndex<FakeDailyIndexRepository> {
        let mut state = download_without_output(raw_date);
        state.compute_output_data_async().await.expect(
            "Given a routed index, or a 404 and a listing with a later date, the download should always succeed",
        );
        StoreDailyIndex::try_from((state, SAMPLE_EXECUTION_ID, FakeDailyIndexRepository::new()))
            .expect("Given a source state with output, the transition should always succeed")
    }

    #[tokio::test]
    async fn should_move_the_response_when_the_day_has_an_index() {
        let raw_date = "2026-10-27";

        let expected_result = Some(INDEX_BODY);

        let state = store_daily_index(raw_date).await;
        let result = state
            .input_data()
            .response
            .as_deref()
            .map(SecResponseTrait::body);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_date_url_and_user_agent_without_response_when_the_day_has_no_filings() {
        let raw_date = "2026-10-25";

        let expected_result = &StoreDailyIndexInput::new(
            None,
            index_date("2026-10-25"),
            SUNDAY_INDEX_URL
                .parse::<Url>()
                .expect("Given a hardcoded valid URL, parsing should always succeed"),
            UserAgent::new("Fake Routing Client fake@example.com").expect(
                "Given a hardcoded SEC-format user agent, validation should always succeed",
            ),
        );

        let state = store_daily_index(raw_date).await;
        let result = state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_execution_id_and_repository_when_source_has_output() {
        let raw_date = "2026-10-27";

        let expected_result =
            &StoreDailyIndexContext::new(SAMPLE_EXECUTION_ID, FakeDailyIndexRepository::new());

        let state = store_daily_index(raw_date).await;
        let result = state.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = download_without_output("2026-10-27");

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(DOWNLOAD_DAILY_INDEX, STORE_DAILY_INDEX),
        ));

        let result = StoreDailyIndex::try_from((
            state,
            SAMPLE_EXECUTION_ID,
            FakeDailyIndexRepository::new(),
        ));

        assert_eq!(result, expected_result);
    }
}
