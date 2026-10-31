//! # Transition: `StoreDailyIndex` → `SelectFilings`
//!
//! Implements the [`TryFrom`] conversion producing a [`SelectFilings`] from a
//! [`StoreDailyIndex`].
//!
//! It moves the stored index from the output of the source state into the input of the next
//! state, and the `execution_id` from its context. A day without filings has no stored index, so
//! the next state gets `None` and selects no filing. The repository drops here.
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output.

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::select_filings::constants::STATE_NAME as SELECT_FILINGS;
use crate::implementations::states::extract::select_filings::{
    SelectFilings, SelectFilingsContext, SelectFilingsInput,
};
use crate::implementations::states::extract::store_daily_index::StoreDailyIndex;
use crate::implementations::states::extract::store_daily_index::constants::STATE_NAME as STORE_DAILY_INDEX;

impl<R> TryFrom<StoreDailyIndex<R>> for SelectFilings {
    type Error = TransitionError;

    /// Builds a [`SelectFilings`] from a computed [`StoreDailyIndex`].
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    fn try_from(state: StoreDailyIndex<R>) -> Result<Self, TransitionError> {
        let (_input, output, context) = state.into_parts();
        let output_data = output
            .ok_or_else(|| transition::MissingOutput::new(STORE_DAILY_INDEX, SELECT_FILINGS))?;

        Ok(Self::new(
            SelectFilingsInput::new(output_data.index),
            SelectFilingsContext::new(context.execution_id),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use bytes::Bytes;
    use pretty_assertions::assert_eq;
    use storage::implementations::repository::filesystem::DocumentKey;
    use uuid::Uuid;

    use super::*;
    use crate::implementations::states::extract::store_daily_index::{
        StoreDailyIndexContext, StoreDailyIndexInput, StoredDailyIndex,
    };
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::index_date::IndexDate;
    use crate::shared::response::SecResponse as SecResponseTrait;
    use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_write_repository::FakeDailyIndexRepository;
    use crate::traits::state_machine::state::State;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);
    const INDEX_BODY: &[u8] = b"Form Type   Company Name\n10-K        SAMPLE CORP\n";

    fn sample_response() -> Arc<SecResponse<ZipBody>> {
        let url: Url = "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261027.idx"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed");
        let headers = Headers::new(HashMap::from([(
            "content-type".to_string(),
            "application/octet-stream".to_string(),
        )]));
        let response = SecResponse::<ZipBody>::from_parts(
            url,
            headers,
            ContentType::OctetStream,
            StatusCode::Ok,
            INDEX_BODY.to_vec(),
        )
        .expect(
            "Given status 200 and an octet stream, building the response should always succeed",
        );
        Arc::new(response)
    }

    /// Returns the state for 2026-10-27. A `response` of `None` marks a day without filings.
    fn store_daily_index(
        response: Option<Arc<SecResponse<ZipBody>>>,
    ) -> StoreDailyIndex<FakeDailyIndexRepository> {
        let requested_url = sample_response().url().clone();
        let index_date = IndexDate::new("2026-10-27")
            .expect("Given a hardcoded past date, validation should always succeed");
        let user_agent = UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        StoreDailyIndex::new(
            StoreDailyIndexInput::new(response, index_date, requested_url, user_agent),
            StoreDailyIndexContext::new(SAMPLE_EXECUTION_ID, FakeDailyIndexRepository::new()),
        )
    }

    #[tokio::test]
    async fn should_move_stored_index_and_execution_id_when_source_has_output() {
        let mut state = store_daily_index(Some(sample_response()));
        state.compute_output_data_async().await.expect(
            "Given a fake repository that accepts every write, the store should always succeed",
        );

        let expected_result = Ok(SelectFilings::new(
            SelectFilingsInput::new(Some(StoredDailyIndex {
                key: DocumentKey::new("sec/daily-index/form.20261027.idx")
                    .expect("Given a hardcoded valid relative path, the key should always build"),
                text: Bytes::from_static(INDEX_BODY),
            })),
            SelectFilingsContext::new(SAMPLE_EXECUTION_ID),
        ));

        let result = SelectFilings::try_from(state);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_move_no_index_when_the_day_has_no_filings() {
        let mut state = store_daily_index(None);
        state
            .compute_output_data_async()
            .await
            .expect("Given no response, the store writes nothing and should always succeed");

        let expected_result = Ok(SelectFilings::new(
            SelectFilingsInput::new(None),
            SelectFilingsContext::new(SAMPLE_EXECUTION_ID),
        ));

        let result = SelectFilings::try_from(state);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = store_daily_index(Some(sample_response()));

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(STORE_DAILY_INDEX, SELECT_FILINGS),
        ));

        let result = SelectFilings::try_from(state);

        assert_eq!(result, expected_result);
    }
}
