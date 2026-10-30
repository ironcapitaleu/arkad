//! # Download Daily Index State
//!
//! Provides the [`DownloadDailyIndex`] state, which downloads the daily index of one date through
//! the [`SecClient`] of its context.
//!
//! The SEC publishes no index for a day without filings. It also has no index for a day that it has
//! not published yet. For both, it answers with 403 or 404. The state tells the two cases apart
//! with the listing of the quarter folder, `daily-index/YYYY/QTRn/index.json`:
//!
//! - If a listing has an index for a later date and does not list the date itself, the day had no
//!   filings. The output has no response. A listed date means the file exists, so the 403 or 404
//!   came from a refused request, for example at the SEC rate limit.
//! - Otherwise, the state fails with [`MissingDailyIndex`], and a later run retries the date.
//!
//! The state reads the listing of the quarter of the date. If that listing has no later date, it
//! also reads the listing of the next quarter, once that quarter has started. This covers a quarter
//! that ends on days without filings. The rule needs no list of SEC holidays.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`DownloadDailyIndexContext`] carried alongside the state.
//! - [`data`]: The [`DownloadDailyIndexInput`] and [`DownloadDailyIndexOutput`] data types.

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::error::state_machine::state::{FailedRequestExecution, MissingDailyIndex};
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::http_client::implementations::sec_client::error::{
    ErrorReason, FailedSecRequest,
};
use crate::shared::index_date::IndexDate;
use crate::shared::index_date::listing::listed_index_dates;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::error::ErrorReason as ResponseErrorReason;
use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
use crate::shared::status_code::StatusCode;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::DownloadDailyIndexContext;
pub use data::DownloadDailyIndexInput;
pub use data::DownloadDailyIndexOutput;

/// Downloads the daily index of one date and applies the rule for a missing index.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct DownloadDailyIndex<C = SecClient> {
    input: DownloadDailyIndexInput,
    context: DownloadDailyIndexContext<C>,
    output: Option<DownloadDailyIndexOutput>,
}

impl<C> DownloadDailyIndex<C> {
    /// Creates a new [`DownloadDailyIndex`] state with no output computed yet.
    #[must_use]
    pub const fn new(
        input: DownloadDailyIndexInput,
        context: DownloadDailyIndexContext<C>,
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
        DownloadDailyIndexInput,
        Option<DownloadDailyIndexOutput>,
        DownloadDailyIndexContext<C>,
    ) {
        (self.input, self.output, self.context)
    }
}

impl<C> DownloadDailyIndex<C>
where
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    async fn has_later_index(&self) -> Result<bool, StateError> {
        let date = self.context.index_date;
        if self.listing_has_index_after(date, date).await? {
            return Ok(true);
        }
        match date.first_day_of_next_quarter() {
            Some(next_quarter) => self.listing_has_index_after(next_quarter, date).await,
            None => Ok(false),
        }
    }

    async fn listing_has_index_after(
        &self,
        quarter: IndexDate,
        date: IndexDate,
    ) -> Result<bool, StateError> {
        let request = SecRequest::builder()
            .daily_index_listing()
            .date(quarter)
            .build();
        match self.context.sec_client.execute_sec_request(request).await {
            Ok(listing) => {
                let listed_dates = listed_index_dates(listing.body());
                let is_date_listed = listed_dates.contains(&date.date());
                Ok(!is_date_listed && listed_dates.iter().any(|listed| *listed > date.date()))
            }
            Err(error) if is_absent(&error) => Ok(false),
            Err(error) => Err(FailedRequestExecution::new(STATE_NAME, error).into()),
        }
    }
}

/// Returns `true` if the SEC answered 403 or 404. For a file that does not exist, the SEC answers
/// 403 with an S3 `AccessDenied` body, as observed on 2026-10-30.
const fn is_absent(error: &FailedSecRequest) -> bool {
    matches!(
        &error.reason,
        ErrorReason::InvalidResponse { source }
            if matches!(
                source.reason,
                ResponseErrorReason::InvalidStatusCode {
                    status_code: StatusCode::Forbidden | StatusCode::NotFound
                }
            )
    )
}

// Deviation: the output holds a concrete `SecResponse<ZipBody>`, and `FailedRequestExecution`
// wraps a `FailedSecRequest`. So the impls that compute the output pin the response and the error
// of `C`. Other impls bind `C: SecClient` only.
#[async_trait]
impl<C> State for DownloadDailyIndex<C>
where
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    /// Downloads the daily index of the input request and stores it as output.
    ///
    /// # Errors
    ///
    /// - Returns [`StateError::InvalidInput`] if the input request is not the request for the
    ///   date of the context.
    /// - Returns [`StateError::MissingDailyIndex`] if the SEC answers 403 or 404 and no listing
    ///   shows that the day had no filings.
    /// - Returns [`StateError::FailedRequestExecution`] if a request fails at the network level,
    ///   or if a response fails a check with a status other than 403 or 404.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let request = self.input.request.clone();
        // The listing is read for the date of the context, so both must name the same date.
        if request
            != SecRequest::builder()
                .daily_index()
                .date(self.context.index_date)
                .build()
        {
            return Err(StateError::InvalidInput);
        }
        let output = match self.context.sec_client.execute_sec_request(request).await {
            Ok(response) => DownloadDailyIndexOutput::new(response),
            Err(error) if is_absent(&error) => {
                if !self.has_later_index().await? {
                    return Err(MissingDailyIndex::new(STATE_NAME, self.context.index_date).into());
                }
                DownloadDailyIndexOutput::without_filings()
            }
            Err(error) => return Err(FailedRequestExecution::new(STATE_NAME, error).into()),
        };
        self.output = Some(output);
        Ok(())
    }
}

impl<C> SMState for DownloadDailyIndex<C>
where
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type InputData = DownloadDailyIndexInput;
    type OutputData = DownloadDailyIndexOutput;
    type Context = DownloadDailyIndexContext<C>;

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

impl<C: SecClientTrait> fmt::Display for DownloadDailyIndex<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            STATE_NAME,
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

    use super::*;
    use crate::shared::response::implementations::sec_response::error::InvalidSecResponse;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;

    const INDEX_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2024/QTR1/form.20240330.idx";
    const QTR1_LISTING_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2024/QTR1/index.json";
    const QTR2_LISTING_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2024/QTR2/index.json";
    const INDEX_BODY: &[u8] = b"Form Type   Company Name\n10-K        SAMPLE CORP\n";
    const OCTET_STREAM: &str = "application/octet-stream";
    const JSON: &str = "application/json";

    fn index_date() -> IndexDate {
        IndexDate::new("2024-03-30")
            .expect("Given a hardcoded past date, validation should always succeed")
    }

    fn listing(names: &[&str]) -> Vec<u8> {
        let items: Vec<serde_json::Value> = names
            .iter()
            .map(|name| serde_json::json!({ "name": name }))
            .collect();
        serde_json::json!({ "directory": { "item": items } })
            .to_string()
            .into_bytes()
    }

    fn test_state(sec_client: RoutingSecClient) -> DownloadDailyIndex<RoutingSecClient> {
        let request = SecRequest::builder()
            .daily_index()
            .date(index_date())
            .build();
        DownloadDailyIndex::new(
            DownloadDailyIndexInput::new(request),
            DownloadDailyIndexContext::new(index_date(), sec_client),
        )
    }

    async fn computed_output(
        sec_client: RoutingSecClient,
    ) -> Result<Option<DownloadDailyIndexOutput>, StateError> {
        let mut state = test_state(sec_client);
        state.compute_output_data_async().await?;
        Ok(state.output_data().cloned())
    }

    #[test]
    fn should_return_state_name_for_download_daily_index() {
        let state = test_state(RoutingSecClient::new());

        let expected_result = "Download Daily Index";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let state = test_state(RoutingSecClient::new());

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_index_text_when_the_sec_answers_200() {
        let sec_client = RoutingSecClient::new().route(INDEX_URL, 200, OCTET_STREAM, INDEX_BODY);

        let expected_result = Some(INDEX_BODY.to_vec());

        let mut state = test_state(sec_client);
        state
            .compute_output_data_async()
            .await
            .expect("Given status 200 and an octet stream, the download should always succeed");
        let result = state
            .output_data()
            .and_then(|output| output.response.as_ref())
            .map(|response| response.body().to_vec());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_no_response_when_the_index_is_404_and_the_listing_has_a_later_date() {
        let sec_client = RoutingSecClient::new().route(
            QTR1_LISTING_URL,
            200,
            JSON,
            &listing(&["form.20240328.idx", "form.20240401.idx"]),
        );

        let expected_result = Ok(Some(DownloadDailyIndexOutput::without_filings()));

        let result = computed_output(sec_client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_no_response_when_the_index_is_403_and_the_listing_has_a_later_date() {
        let sec_client = RoutingSecClient::new()
            .route(
                INDEX_URL,
                403,
                "application/xml",
                b"<Error>AccessDenied</Error>",
            )
            .route(
                QTR1_LISTING_URL,
                200,
                JSON,
                &listing(&["form.20240401.idx"]),
            );

        let expected_result = Ok(Some(DownloadDailyIndexOutput::without_filings()));

        let result = computed_output(sec_client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_no_response_when_only_the_next_quarter_listing_has_a_later_date() {
        let sec_client = RoutingSecClient::new()
            .route(
                QTR1_LISTING_URL,
                200,
                JSON,
                &listing(&["form.20240328.idx"]),
            )
            .route(
                QTR2_LISTING_URL,
                200,
                JSON,
                &listing(&["form.20240401.idx"]),
            );

        let expected_result = Ok(Some(DownloadDailyIndexOutput::without_filings()));

        let result = computed_output(sec_client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_daily_index_when_the_index_is_404_and_no_listing_has_a_later_date()
     {
        let sec_client = RoutingSecClient::new()
            .route(
                QTR1_LISTING_URL,
                200,
                JSON,
                &listing(&["form.20240328.idx", "form.20240330.idx"]),
            )
            .route(QTR2_LISTING_URL, 404, JSON, b"");

        let expected_result = Err(MissingDailyIndex::new(STATE_NAME, index_date()).into());

        let result = computed_output(sec_client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_daily_index_when_the_index_is_403_and_the_listing_names_the_date()
     {
        let sec_client = RoutingSecClient::new()
            .route(
                INDEX_URL,
                403,
                "text/html",
                b"Request Rate Threshold Exceeded",
            )
            .route(
                QTR1_LISTING_URL,
                200,
                JSON,
                &listing(&["form.20240330.idx", "form.20240401.idx"]),
            );

        let expected_result = Err(MissingDailyIndex::new(STATE_NAME, index_date()).into());

        let result = computed_output(sec_client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_daily_index_when_the_index_and_both_listings_answer_403() {
        let sec_client = RoutingSecClient::new()
            .route(INDEX_URL, 403, "text/html", b"")
            .route(QTR1_LISTING_URL, 403, "text/html", b"")
            .route(QTR2_LISTING_URL, 403, "text/html", b"");

        let expected_result = Err(MissingDailyIndex::new(STATE_NAME, index_date()).into());

        let result = computed_output(sec_client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_no_response_when_the_quarter_listing_is_absent_and_the_next_has_a_later_date()
     {
        let sec_client = RoutingSecClient::new().route(
            QTR2_LISTING_URL,
            200,
            JSON,
            &listing(&["form.20240401.idx"]),
        );

        let expected_result = Ok(Some(DownloadDailyIndexOutput::without_filings()));

        let result = computed_output(sec_client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_input_when_the_request_and_the_context_name_different_dates() {
        let other_date = IndexDate::new("2024-04-02")
            .expect("Given a hardcoded past date, validation should always succeed");
        let request = SecRequest::builder().daily_index().date(other_date).build();
        let mut state = DownloadDailyIndex::new(
            DownloadDailyIndexInput::new(request),
            DownloadDailyIndexContext::new(index_date(), RoutingSecClient::new()),
        );

        let expected_result = Err(StateError::InvalidInput);

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_request_execution_when_the_listing_answers_500() {
        let sec_client = RoutingSecClient::new().route(QTR1_LISTING_URL, 500, JSON, b"");

        let expected_result = Err(FailedRequestExecution::new(
            STATE_NAME,
            FailedSecRequest::from(InvalidSecResponse::new(
                ResponseErrorReason::InvalidStatusCode {
                    status_code: StatusCode::InternalServerError,
                },
            )),
        )
        .into());

        let result = computed_output(sec_client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_request_execution_when_the_index_answers_500() {
        let sec_client = RoutingSecClient::new()
            .route(INDEX_URL, 500, OCTET_STREAM, b"")
            .route(
                QTR1_LISTING_URL,
                200,
                JSON,
                &listing(&["form.20240401.idx"]),
            );

        let expected_result = Err(FailedRequestExecution::new(
            STATE_NAME,
            FailedSecRequest::from(InvalidSecResponse::new(
                ResponseErrorReason::InvalidStatusCode {
                    status_code: StatusCode::InternalServerError,
                },
            )),
        )
        .into());

        let result = computed_output(sec_client).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let sec_client = RoutingSecClient::new().route(INDEX_URL, 200, OCTET_STREAM, INDEX_BODY);
        let mut state = test_state(sec_client);

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
    const fn should_implement_value_traits_for_download_daily_index() {
        implements_value_traits::<DownloadDailyIndex>();
    }
}
