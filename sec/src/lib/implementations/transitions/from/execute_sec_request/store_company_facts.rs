//! # Transition: `ExecuteSecRequest` → `StoreCompanyFacts`
//!
//! Implements the [`TryFrom`] conversion producing a [`StoreCompanyFacts`] from an
//! [`ExecuteCompanyFactsRequest`], the `execution_id`, and the storage repository.
//!
//! It moves the response from the output of the source state, the requested URL from its
//! [`SecRequest`], the `User-Agent` from its [`SecClient`], and the CIK from its context. The
//! `execution_id` and the repository belong to the state machine run, not to the source state, so
//! the caller passes the ones from the state machine context.
//!
//! # Errors
//!
//! Returns [`TransitionError`] when the source state has no computed output
//! ([`MissingOutput`](crate::error::state_machine::transition::MissingOutput)), or when its
//! request names a different CIK from its context
//! ([`FailedContextConversion`](crate::error::state_machine::transition::FailedContextConversion)).

use std::sync::Arc;

use uuid::Uuid;

use super::request_check::ensure_request_matches_input;
use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::ExecuteCompanyFactsRequest;
use crate::implementations::states::extract::execute_sec_request::constants::STATE_NAME as EXECUTE_SEC_REQUEST;
use crate::implementations::states::extract::store_company_facts::constants::STATE_NAME as STORE_COMPANY_FACTS;
use crate::implementations::states::extract::store_company_facts::{
    StoreCompanyFacts, StoreCompanyFactsContext, StoreCompanyFactsInput,
};
use crate::shared::http_client::SecClient as SecClientTrait;
#[cfg(doc)]
use crate::shared::http_client::implementations::sec_client::SecClient;
#[cfg(doc)]
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::url::Url;

impl<R, C: SecClientTrait> TryFrom<(ExecuteCompanyFactsRequest<R, C>, Uuid, R)>
    for StoreCompanyFacts<R>
{
    type Error = TransitionError;

    /// Builds a [`StoreCompanyFacts`] from a computed [`ExecuteCompanyFactsRequest`], the
    /// `execution_id`, and the storage repository.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::MissingOutput`] if `state` has no computed output.
    ///
    /// Returns [`TransitionError::FailedContextConversion`] if the request of `state` names a
    /// different CIK from its context. The stored key follows the context CIK, so a mismatch stores
    /// a response under the wrong company.
    fn try_from(
        (state, execution_id, repository): (ExecuteCompanyFactsRequest<R, C>, Uuid, R),
    ) -> Result<Self, TransitionError> {
        let (input, output, context) = state.into_parts();
        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(EXECUTE_SEC_REQUEST, STORE_COMPANY_FACTS)
        })?;

        ensure_request_matches_input(
            &input.sec_request,
            &context.request_input,
            "CIK",
            STORE_COMPANY_FACTS,
        )?;

        let requested_url = Url::from(input.sec_request.inner.url().clone());
        let user_agent = input.sec_client.user_agent().clone();
        let new_input = StoreCompanyFactsInput::new(
            Arc::unwrap_or_clone(output_data.response),
            requested_url,
            user_agent,
        );
        let new_context =
            StoreCompanyFactsContext::new(context.request_input, execution_id, repository);

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
        ExecuteSecRequestContext, ExecuteSecRequestInput, ExecuteSecRequestOutput,
    };
    use crate::prelude::*;
    use crate::shared::cik::Cik;
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::http_client::implementations::reqwest_client::ReqwestClient;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::request::implementations::sec_request::{CompanyFacts, SecRequest};
    use crate::shared::response::implementations::sec_response::SecResponse;
    use crate::shared::status_code::StatusCode;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);

    const REQUESTED_URL: &str = "https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json";
    const REDIRECTED_URL: &str = "https://data.sec.gov/redirected/CIK0001067983.json";

    fn sample_cik() -> Cik {
        Cik::new("1067983")
            .expect("Given a hardcoded numeric CIK, its creation should always succeed")
    }

    fn other_user_agent() -> UserAgent {
        UserAgent::new("Other Company other@example.com")
            .expect("Given a hardcoded SEC-format string, validation should always succeed")
    }

    fn sec_client_with_other_user_agent() -> SecClient {
        let inner = ReqwestClient::new(other_user_agent())
            .expect("Given a validated user agent, building the HTTP client fails only if the TLS backend cannot initialize");
        SecClient::new(inner)
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

    fn sample_context() -> ExecuteSecRequestContext<CompanyFacts, FakeWriteRepository> {
        ExecuteSecRequestContext::new(sample_cik(), FakeWriteRepository::new())
    }

    fn execute_sec_request_without_output(
        cik: Cik,
    ) -> ExecuteCompanyFactsRequest<FakeWriteRepository> {
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let input = ExecuteSecRequestInput::new(sec_client_with_other_user_agent(), request);
        ExecuteCompanyFactsRequest::new(input, sample_context())
    }

    fn store_company_facts() -> StoreCompanyFacts<FakeWriteRepository> {
        let state = execute_sec_request_without_output(sample_cik())
            .with_output(ExecuteSecRequestOutput::new(sample_response()));
        StoreCompanyFacts::try_from((state, SAMPLE_EXECUTION_ID, rejecting_repository()))
            .expect("Given a source state with output, the transition should always succeed")
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
    fn should_take_user_agent_from_sec_client_when_source_has_output() {
        let state = store_company_facts();

        let expected_result = &other_user_agent();

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
    fn should_take_execution_id_from_run_when_source_has_output() {
        let state = store_company_facts();

        let expected_result = SAMPLE_EXECUTION_ID;

        let result = state.context_data().execution_id();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_repository_from_run_when_source_has_output() {
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
            transition::FailedContextConversion::new(
                EXECUTE_SEC_REQUEST,
                STORE_COMPANY_FACTS,
                "The request targets 'https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json', but the context names CIK '0001067983'",
            ),
        );

        let result =
            StoreCompanyFacts::try_from((state, SAMPLE_EXECUTION_ID, rejecting_repository()))
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

        let result =
            StoreCompanyFacts::try_from((state, SAMPLE_EXECUTION_ID, rejecting_repository()))
                .expect_err(
                    "Given a source state without output, the transition should always fail",
                );

        assert_eq!(result, expected_result);
    }
}
