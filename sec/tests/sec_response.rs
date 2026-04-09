use pretty_assertions::assert_eq;
use reqwest::{Method, Request};

use sec::shared::content_type::ContentType;
use sec::shared::http_client::InnerClient;
use sec::shared::http_client::implementations::reqwest_client::ReqwestClient;
use sec::shared::response::SecResponse as SecResponseTrait;
use sec::shared::response::implementations::sec_response::SecResponse;
use sec::shared::response::implementations::sec_response::error::ErrorReason;
use sec::shared::status_code::StatusCode;

#[tokio::test]
async fn should_create_sec_response_when_response_is_valid_json_with_success_status() {
    let client = ReqwestClient::default();
    let url = "https://httpbin.org/get"; // Returns 200 OK with application/json content type and valid JSON body
    let request_url = reqwest::Url::parse(url).expect("The hardcoded URL should always be valid");
    let request = Request::new(Method::GET, request_url);

    let expected_result = true;

    let response = client
        .execute_request(request)
        .await
        .expect("A request to httpbin.org should always succeed");
    let result = SecResponse::from_inner(response).await.is_ok();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_json_content_type_when_sec_response_is_created_successfully() {
    let client = ReqwestClient::default();
    let url = "https://httpbin.org/get";
    let request_url = reqwest::Url::parse(url).expect("The hardcoded URL should always be valid");
    let request = Request::new(Method::GET, request_url);

    let response = client
        .execute_request(request)
        .await
        .expect("A request to httpbin.org should always succeed");

    let expected_result = ContentType::Json;

    let result = SecResponse::from_inner(response)
        .await
        .expect("A valid JSON response with 200 status should create a SecResponse")
        .content_type();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_ok_status_code_when_sec_response_is_created_successfully() {
    let client = ReqwestClient::default();
    let url = "https://httpbin.org/get";
    let request_url = reqwest::Url::parse(url).expect("The hardcoded URL should always be valid");
    let request = Request::new(Method::GET, request_url);

    let response = client
        .execute_request(request)
        .await
        .expect("A request to httpbin.org should always succeed");

    let expected_result = StatusCode::Ok;

    let result = SecResponse::from_inner(response)
        .await
        .expect("A valid JSON response with 200 status should create a SecResponse")
        .status_code();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_fail_with_invalid_status_code_when_response_is_not_success() {
    let client = ReqwestClient::default();
    let url = "https://httpbin.org/status/404"; // Returns 404 Not Found
    let request_url = reqwest::Url::parse(url).expect("The hardcoded URL should always be valid");
    let request = Request::new(Method::GET, request_url);

    let response = client
        .execute_request(request)
        .await
        .expect("A request to httpbin.org should always succeed");

    let expected_result = ErrorReason::InvalidStatusCode {
        status_code: StatusCode::NotFound,
    };

    let result = SecResponse::from_inner(response)
        .await
        .expect_err("A 404 response should fail to create a SecResponse")
        .reason;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_fail_with_invalid_content_type_when_response_is_not_json() {
    let client = ReqwestClient::default();
    let url = "https://httpbin.org/html"; // Returns 200 OK with text/html content type
    let request_url = reqwest::Url::parse(url).expect("The hardcoded URL should always be valid");
    let request = Request::new(Method::GET, request_url);

    let response = client
        .execute_request(request)
        .await
        .expect("A request to httpbin.org should always succeed");

    let expected_result = ErrorReason::InvalidContentType {
        content_type: ContentType::Html,
    };

    let result = SecResponse::from_inner(response)
        .await
        .expect_err("An HTML response should fail to create a SecResponse")
        .reason;

    assert_eq!(result, expected_result);
}
