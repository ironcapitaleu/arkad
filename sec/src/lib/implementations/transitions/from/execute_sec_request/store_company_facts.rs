//! # Transition: `ExecuteSecRequest` → `StoreCompanyFacts`
//!
//! Implements the [`TryFrom`] conversion producing a [`StoreCompanyFacts`] from an
//! [`ExecuteSecRequest`].
//!
//! It moves the response from the output of the source state, the requested URL from its
//! [`SecRequest`], and the `User-Agent` from its context. The supporting `From` impl carries the
//! CIK, the `execution_id` and the storage repository across via context. The requested URL must
//! name the CIK of the context.
//!
//! # Errors
//!
//! Returns [`TransitionError`] when the source state has no computed output
//! ([`MissingOutput`](crate::error::state_machine::transition::MissingOutput)), or when its
//! request names another CIK than its context
//! ([`FailedContextConversion`](crate::error::state_machine::transition::FailedContextConversion)).

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::constants::STATE_NAME as EXECUTE_SEC_REQUEST;
use crate::implementations::states::extract::execute_sec_request::{
    ExecuteSecRequest, ExecuteSecRequestContext,
};
use crate::implementations::states::extract::store_company_facts::constants::STATE_NAME as STORE_COMPANY_FACTS;
use crate::implementations::states::extract::store_company_facts::{
    StoreCompanyFacts, StoreCompanyFactsContext, StoreCompanyFactsInput,
};
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::url::Url;

impl<R> From<ExecuteSecRequestContext<R>> for StoreCompanyFactsContext<R> {
    fn from(context: ExecuteSecRequestContext<R>) -> Self {
        Self::new(context.cik, context.execution_id, context.repository)
    }
}

impl<R> TryFrom<ExecuteSecRequest<R>> for StoreCompanyFacts<R> {
    type Error = TransitionError;

    /// Builds a [`StoreCompanyFacts`] from a computed [`ExecuteSecRequest`].
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    ///
    /// Returns [`TransitionError::FailedContextConversion`] if the request of `state` names
    /// another CIK than its context. The stored key follows the context CIK, so a mismatch would
    /// store a response under the wrong company.
    fn try_from(state: ExecuteSecRequest<R>) -> Result<Self, TransitionError> {
        let (input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(EXECUTE_SEC_REQUEST, STORE_COMPANY_FACTS)
        })?;

        let context_request = SecRequest::builder()
            .all_company_facts()
            .cik(context.cik.clone())
            .build();
        if input.sec_request != context_request {
            return Err(transition::FailedContextConversion::new(
                EXECUTE_SEC_REQUEST,
                STORE_COMPANY_FACTS,
            )
            .into());
        }

        let requested_url = Url::from(input.sec_request.inner.url().clone());
        let new_input = StoreCompanyFactsInput::new(
            output_data.response,
            requested_url,
            context.user_agent.clone(),
        );
        let new_context: StoreCompanyFactsContext<R> = context.into();

        Ok(Self::new(new_input, new_context))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;
    use storage::WriteError;

    use super::*;
    use crate::implementations::states::extract::execute_sec_request::{
        ExecuteSecRequestInput, ExecuteSecRequestOutput,
    };
    use crate::prelude::*;
    use crate::shared::cik::Cik;
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::response::implementations::sec_response::SecResponse;
    use crate::shared::status_code::StatusCode;
    use crate::tests::fixtures::sample_run::{SAMPLE_EXECUTION_ID, sample_user_agent};
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    const REQUESTED_URL: &str = "https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json";
    const REDIRECTED_URL: &str = "https://data.sec.gov/redirected/CIK0001067983.json";

    fn sample_cik() -> Cik {
        Cik::new("1067983")
            .expect("Given a hardcoded numeric CIK, its creation should always succeed")
    }

    fn rejecting_repository() -> FakeWriteRepository {
        FakeWriteRepository::rejecting(WriteError::conflicting_write("Sample conflict"))
    }

    fn sample_response() -> SecResponse {
        let url: Url = REDIRECTED_URL
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed");
        SecResponse::from_parts(
            url,
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            b"{}".to_vec(),
        )
        .expect("Given the hardcoded JSON body '{}', building the response should always succeed")
    }

    fn sample_context() -> ExecuteSecRequestContext<FakeWriteRepository> {
        ExecuteSecRequestContext::new(
            sample_cik(),
            SAMPLE_EXECUTION_ID,
            sample_user_agent(),
            rejecting_repository(),
        )
    }

    fn execute_sec_request_without_output(cik: Cik) -> ExecuteSecRequest<FakeWriteRepository> {
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let input = ExecuteSecRequestInput::new(SecClient::default(), request);
        ExecuteSecRequest::new(input, sample_context())
    }

    fn store_company_facts() -> StoreCompanyFacts<FakeWriteRepository> {
        let state = execute_sec_request_without_output(sample_cik())
            .with_output(ExecuteSecRequestOutput::new(sample_response()));
        StoreCompanyFacts::try_from(state)
            .expect("Given a source state with output, the transition should always succeed")
    }

    #[test]
    fn should_convert_context_when_valid_context() {
        let context = sample_context();

        let expected_result = StoreCompanyFactsContext::new(
            sample_cik(),
            SAMPLE_EXECUTION_ID,
            rejecting_repository(),
        );

        let result = StoreCompanyFactsContext::from(context);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_move_response_when_source_has_output() {
        let state = store_company_facts();

        let expected_result = &sample_response();

        let result = state.input_data().response();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_requested_url_from_sec_request_when_response_url_differs() {
        let state = store_company_facts();

        let expected_result = REQUESTED_URL;

        let result = state.input_data().requested_url().as_str();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_user_agent_from_context_when_source_has_output() {
        let state = store_company_facts();

        let expected_result = &sample_user_agent();

        let result = state.input_data().user_agent();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_move_cik_when_source_has_output() {
        let state = store_company_facts();

        let expected_result = &sample_cik();

        let result = state.context_data().cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_move_execution_id_when_source_has_output() {
        let state = store_company_facts();

        let expected_result = SAMPLE_EXECUTION_ID;

        let result = state.context_data().execution_id();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_move_repository_when_source_has_output() {
        let state = store_company_facts();

        let expected_result = &rejecting_repository();

        let result = state.context_data().repository();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_context_conversion_when_request_names_another_cik() {
        let other_cik = Cik::new("320193")
            .expect("Given a hardcoded numeric CIK, its creation should always succeed");
        let state = execute_sec_request_without_output(other_cik)
            .with_output(ExecuteSecRequestOutput::new(sample_response()));

        let expected_result = TransitionError::FailedContextConversion(
            transition::FailedContextConversion::new(EXECUTE_SEC_REQUEST, STORE_COMPANY_FACTS),
        );

        let result = StoreCompanyFacts::try_from(state)
            .expect_err("Given a request for another CIK, the transition should always fail");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_source_has_no_output() {
        let state = execute_sec_request_without_output(sample_cik());

        let expected_result = TransitionError::MissingOutput(transition::MissingOutput::new(
            EXECUTE_SEC_REQUEST,
            STORE_COMPANY_FACTS,
        ));

        let result = StoreCompanyFacts::try_from(state)
            .expect_err("Given a source state without output, the transition should always fail");

        assert_eq!(result, expected_result);
    }
}
