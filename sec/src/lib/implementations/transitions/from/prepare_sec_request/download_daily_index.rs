//! # Transition: `PrepareSecRequest<DailyIndex>` → `DownloadDailyIndex`
//!
//! Implements the [`TryFrom`] conversion producing a [`DownloadDailyIndex`] from a
//! [`PrepareSecRequest`] of the [`DailyIndex`] kind and an SEC client.
//!
//! It moves the prepared request from the output of the source state into the input of the next
//! state. The context takes the date from the source input, which built the request, and the
//! client the caller passes. [`DownloadDailyIndex`] writes nothing, so the repository of the
//! source context drops here.
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::download_daily_index::constants::STATE_NAME as DOWNLOAD_DAILY_INDEX;
use crate::implementations::states::extract::download_daily_index::{
    DownloadDailyIndex, DownloadDailyIndexContext, DownloadDailyIndexInput,
};
use crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
use crate::implementations::states::extract::prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
use crate::shared::request::implementations::sec_request::DailyIndex;

impl<C> TryFrom<(PrepareSecRequest<DailyIndex>, C)> for DownloadDailyIndex<C> {
    type Error = TransitionError;

    /// Builds a [`DownloadDailyIndex`] from a computed [`PrepareSecRequest`] of the [`DailyIndex`]
    /// kind and the client that downloads the index.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    fn try_from(
        (state, sec_client): (PrepareSecRequest<DailyIndex>, C),
    ) -> Result<Self, TransitionError> {
        let (input, output, _context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(PREPARE_SEC_REQUEST, DOWNLOAD_DAILY_INDEX)
        })?;

        Ok(Self::new(
            DownloadDailyIndexInput::new(output_data.request),
            DownloadDailyIndexContext::new(input.request_input, sec_client),
        ))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::implementations::states::extract::prepare_sec_request::{
        PrepareSecRequestContext, PrepareSecRequestInput,
    };
    use crate::shared::index_date::IndexDate;
    use crate::shared::request::implementations::sec_request::SecRequest;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use crate::traits::state_machine::state::State;

    fn index_date() -> IndexDate {
        IndexDate::new("2026-10-27")
            .expect("Given a hardcoded past date, validation should always succeed")
    }

    fn prepare_daily_index_request() -> PrepareSecRequest<DailyIndex> {
        PrepareSecRequest::new(
            PrepareSecRequestInput::new(index_date()),
            PrepareSecRequestContext::new(index_date()),
        )
    }

    #[tokio::test]
    async fn should_move_request_date_and_client_when_source_has_output() {
        let mut state = prepare_daily_index_request();
        state
            .compute_output_data_async()
            .await
            .expect("Given a validated date, preparing the request should always succeed");
        let sec_client = RoutingSecClient::new();

        let expected_result = Ok(DownloadDailyIndex::new(
            DownloadDailyIndexInput::new(
                SecRequest::builder()
                    .daily_index()
                    .date(index_date())
                    .build(),
            ),
            DownloadDailyIndexContext::new(index_date(), RoutingSecClient::new()),
        ));

        let result = DownloadDailyIndex::try_from((state, sec_client));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = prepare_daily_index_request();
        let sec_client = RoutingSecClient::new();

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(PREPARE_SEC_REQUEST, DOWNLOAD_DAILY_INDEX),
        ));

        let result = DownloadDailyIndex::try_from((state, sec_client));

        assert_eq!(result, expected_result);
    }
}
