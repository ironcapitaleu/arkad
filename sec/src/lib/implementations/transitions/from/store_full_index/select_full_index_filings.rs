//! # Transition: `StoreFullIndex` → `SelectFullIndexFilings`
//!
//! Implements the [`TryFrom`] conversion producing a [`SelectFullIndexFilings`] from a
//! [`StoreFullIndex`] and the first and the last date filed to keep.
//!
//! It moves the stored index from the output of the source state into the input of the next
//! state, and the `execution_id` from its context. The repository drops here.
//!
//! # Errors
//!
//! Returns [`TransitionError`] (a [`MissingOutput`](crate::error::state_machine::transition::MissingOutput))
//! when the source state has no computed output.

use chrono::NaiveDate;

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::select_full_index_filings::constants::STATE_NAME as SELECT_FULL_INDEX_FILINGS;
use crate::implementations::states::extract::select_full_index_filings::{
    SelectFullIndexFilings, SelectFullIndexFilingsContext, SelectFullIndexFilingsInput,
};
use crate::implementations::states::extract::store_full_index::StoreFullIndex;
use crate::implementations::states::extract::store_full_index::constants::STATE_NAME as STORE_FULL_INDEX;

impl<R> TryFrom<(StoreFullIndex<R>, NaiveDate, NaiveDate)> for SelectFullIndexFilings {
    type Error = TransitionError;

    /// Builds a [`SelectFullIndexFilings`] from a computed [`StoreFullIndex`] and the first and
    /// the last date filed to keep.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    fn try_from(
        (state, first_filed_date, last_filed_date): (StoreFullIndex<R>, NaiveDate, NaiveDate),
    ) -> Result<Self, TransitionError> {
        let (_input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(STORE_FULL_INDEX, SELECT_FULL_INDEX_FILINGS)
        })?;

        Ok(Self::new(
            SelectFullIndexFilingsInput::new(output_data.index),
            SelectFullIndexFilingsContext::new(
                context.execution_id,
                first_filed_date,
                last_filed_date,
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use bytes::Bytes;
    use pretty_assertions::assert_eq;
    use storage::Sha256Digest;
    use storage::implementations::repository::filesystem::{DocumentKey, IngestionMode};
    use uuid::Uuid;

    use super::*;
    use crate::implementations::states::extract::store_full_index::{
        StoreFullIndexContext, StoreFullIndexInput,
    };
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::quarter::Quarter;
    use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
    use crate::shared::status_code::StatusCode;
    use crate::shared::stored_form_index::StoredFormIndex;
    use crate::shared::url::Url;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_write_repository::FakeFullIndexRepository;
    use crate::traits::state_machine::state::State;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);
    const INDEX_BODY: &[u8] = b"Form Type   Company Name\n---\n";

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, the date should always build")
    }

    fn store_full_index() -> StoreFullIndex<FakeFullIndexRepository> {
        let url: Url = "https://www.sec.gov/Archives/edgar/full-index/2026/QTR4/form.idx"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed");
        let headers = Headers::new(HashMap::from([(
            "content-type".to_string(),
            "application/octet-stream".to_string(),
        )]));
        let response = SecResponse::<ZipBody>::from_parts(
            url.clone(),
            headers,
            ContentType::OctetStream,
            StatusCode::Ok,
            INDEX_BODY.to_vec(),
        )
        .expect(
            "Given status 200 and an octet stream, building the response should always succeed",
        );
        let user_agent = UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        StoreFullIndex::new(
            StoreFullIndexInput::new(Arc::new(response), url, user_agent),
            StoreFullIndexContext::new(
                Quarter::of(date(2026, 10, 1)),
                SAMPLE_EXECUTION_ID,
                IngestionMode::Incremental,
                FakeFullIndexRepository::new(),
            ),
        )
    }

    #[tokio::test]
    async fn should_move_the_stored_index_the_execution_id_and_the_dates_when_source_has_output() {
        let mut state = store_full_index();
        state.compute_output_data_async().await.expect(
            "Given a fake repository that accepts every write, the store should always succeed",
        );
        let first_filed_date = date(2026, 9, 24);
        let last_filed_date = date(2026, 11, 12);

        let expected_result = Ok(SelectFullIndexFilings::new(
            SelectFullIndexFilingsInput::new(StoredFormIndex {
                key: DocumentKey::new("sec/full-index/2026/QTR4/form.idx")
                    .expect("Given a hardcoded valid relative path, the key should always build"),
                sha256: Sha256Digest::of(INDEX_BODY),
                text: Bytes::from_static(INDEX_BODY),
            }),
            SelectFullIndexFilingsContext::new(
                SAMPLE_EXECUTION_ID,
                date(2026, 9, 24),
                date(2026, 11, 12),
            ),
        ));

        let result = SelectFullIndexFilings::try_from((state, first_filed_date, last_filed_date));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = store_full_index();
        let first_filed_date = date(2026, 9, 24);
        let last_filed_date = date(2026, 11, 12);

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(STORE_FULL_INDEX, SELECT_FULL_INDEX_FILINGS),
        ));

        let result = SelectFullIndexFilings::try_from((state, first_filed_date, last_filed_date));

        assert_eq!(result, expected_result);
    }
}
