use pretty_assertions::assert_eq;

use sec::shared::cik::Cik;
use sec::shared::content_type::ContentType;
use sec::shared::http_client::SecClient as SecClientTrait;
use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::http_client::implementations::sec_client::error::ErrorReason;
use sec::shared::request::SecRequest as SecRequestTrait;
use sec::shared::request::implementations::sec_request::{SecRequest, SecRequestType};
use sec::shared::response::SecResponse as SecResponseTrait;
use sec::shared::response::implementations::sec_response::error::{
    ErrorReason as SecResponseErrorReason, InvalidSecResponse,
};
use sec::shared::status_code::StatusCode;

/// Creates an `SecClient` with a proper User-Agent header required by the SEC API.
fn sec_client() -> SecClient {
    let http_client = reqwest::Client::builder()
        .user_agent("ArkadTest admin@example.com")
        .build()
        .expect("A hardcoded user agent should always produce a valid HTTP client");
    SecClient::new(http_client)
}

#[tokio::test]
async fn should_return_sec_response_when_executing_valid_sec_request() {
    let client = sec_client();
    let cik = Cik::new("1067983").expect("A hardcoded CIK should always be valid");
    let request = SecRequest::new(SecRequestType::new_fetch_all_company_facts(cik));

    let expected_result = true;

    let result = client.execute_sec_request(request).await.is_ok();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_json_content_type_when_sec_request_succeeds() {
    let client = sec_client();
    let cik = Cik::new("1067983").expect("A hardcoded CIK should always be valid");
    let request = SecRequest::new(SecRequestType::new_fetch_all_company_facts(cik));

    let expected_result = ContentType::Json;

    let result = client
        .execute_sec_request(request)
        .await
        .expect("A valid SEC request should return a valid SecResponse")
        .content_type();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_non_empty_json_body_when_sec_request_succeeds() {
    let client = sec_client();
    let cik = Cik::new("1067983").expect("A hardcoded CIK should always be valid");
    let request = SecRequest::new(SecRequestType::new_fetch_all_company_facts(cik));

    let expected_result = true;

    let result = client
        .execute_sec_request(request)
        .await
        .expect("A valid SEC request should return a valid SecResponse")
        .body()
        .is_object();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_fail_with_invalid_response_when_cik_does_not_exist() {
    let client = sec_client();
    let cik = Cik::new("0000000000").expect("A hardcoded CIK should always be valid");
    let request = SecRequest::new(SecRequestType::new_fetch_all_company_facts(cik));
    let response_error = InvalidSecResponse::new(SecResponseErrorReason::InvalidStatusCode {
        status_code: StatusCode::NotFound,
    });

    let expected_result = ErrorReason::InvalidResponse {
        source: response_error,
    };

    let result = client
        .execute_sec_request(request)
        .await
        .expect_err("A request for a non-existent CIK should always fail")
        .reason;

    assert_eq!(result, expected_result);
}
