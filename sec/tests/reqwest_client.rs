use pretty_assertions::assert_eq;
use reqwest::Request;

use sec::shared::http_client::InnerClient;

#[tokio::test]
async fn should_return_ok_status_code_when_request_is_valid() {
    let client = reqwest::Client::new();
    let url = "https://httpbin.org/get"; // Returns canned response with 200 OK status code
    let request_url = reqwest::Url::parse(url)
        .expect(&format!("The harcoded URL `{url}` should always be valid"));
    let request_method = reqwest::Method::GET;
    let request = Request::new(request_method, request_url);

    let expected_result = reqwest::StatusCode::OK;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!(
            "A request to the URL `{url}` should always succeed"
        ))
        .status();

    drop(client);
    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_created_status_code_when_resource_is_created() {
    let client = reqwest::Client::new();
    let url = "https://httpbin.org/status/201"; // Returns canned response with 201 Created status code
    let request_url = reqwest::Url::parse(url)
        .expect(&format!("The harcoded URL `{url}` should always be valid"));
    let request_method = reqwest::Method::POST;
    let request = Request::new(request_method, request_url);

    let expected_result = reqwest::StatusCode::CREATED;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!(
            "A request to the URL `{url}` should always succeed"
        ))
        .status();

    drop(client);
    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_bad_request_status_code_when_request_is_invalid() {
    let client = reqwest::Client::new();
    let url = "https://httpbin.org/status/400"; // Returns canned response with 400 Bad Request status code
    let request_url = reqwest::Url::parse(url)
        .expect(&format!("The harcoded URL `{url}` should always be valid"));
    let request_method = reqwest::Method::GET;
    let request = Request::new(request_method, request_url);

    let expected_result = reqwest::StatusCode::BAD_REQUEST;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!(
            "A request to the URL `{url}` should always succeed"
        ))
        .status();

    drop(client);
    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_unauthorized_status_code_when_not_authorized() {
    let client = reqwest::Client::new();
    let url = "https://httpbin.org/status/401"; // Returns canned response with 401 Unauthorized status code
    let request_url = reqwest::Url::parse(url)
        .expect(&format!("The harcoded URL `{url}` should always be valid"));
    let request_method = reqwest::Method::GET;
    let request = Request::new(request_method, request_url);

    let expected_result = reqwest::StatusCode::UNAUTHORIZED;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!(
            "A request to the URL `{url}` should always succeed"
        ))
        .status();

    drop(client);
    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_forbidden_status_code_when_resource_is_forbidden() {
    let client = reqwest::Client::new();
    let url = "https://httpbin.org/status/403"; // Returns canned response with 403 Forbidden status code
    let request_url = reqwest::Url::parse(url)
        .expect(&format!("The harcoded URL `{url}` should always be valid"));
    let request_method = reqwest::Method::GET;
    let request = Request::new(request_method, request_url);

    let expected_result = reqwest::StatusCode::FORBIDDEN;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!(
            "A request to the URL `{url}` should always succeed"
        ))
        .status();

    drop(client);
    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_not_found_status_code_when_resource_is_not_found() {
    let client = reqwest::Client::new();
    let url = "https://httpbin.org/status/404"; // Returns canned response with 404 Not Found status code
    let request_url = reqwest::Url::parse(url)
        .expect(&format!("The harcoded URL `{url}` should always be valid"));
    let request_method = reqwest::Method::GET;
    let request = Request::new(request_method, request_url);

    let expected_result = reqwest::StatusCode::NOT_FOUND;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!(
            "A request to the URL `{url}` should always succeed"
        ))
        .status();

    drop(client);
    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_too_many_requests_status_code_when_rate_limited() {
    let client = reqwest::Client::new();
    let url = "https://httpbin.org/status/429"; // Returns canned response with 429 Too Many Requests status code
    let request_url = reqwest::Url::parse(url)
        .expect(&format!("The harcoded URL `{url}` should always be valid"));
    let request_method = reqwest::Method::GET;
    let request = Request::new(request_method, request_url);

    let expected_result = reqwest::StatusCode::TOO_MANY_REQUESTS;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!(
            "A request to the URL `{url}` should always succeed"
        ))
        .status();

    drop(client);
    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_internal_server_error_status_code_when_server_error_occurs() {
    let client = reqwest::Client::new();
    let url = "https://httpbin.org/status/500"; // Returns canned response with 500 Internal Server Error status code
    let request_url = reqwest::Url::parse(url)
        .expect(&format!("The harcoded URL `{url}` should always be valid"));
    let request_method = reqwest::Method::GET;
    let request = Request::new(request_method, request_url);

    let expected_result = reqwest::StatusCode::INTERNAL_SERVER_ERROR;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!(
            "A request to the URL `{url}` should always succeed"
        ))
        .status();

    drop(client);
    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_service_unavailable_status_code_when_service_is_unavailable() {
    let client = reqwest::Client::new();
    let url = "https://httpbin.org/status/503"; // Returns canned response with 503 Service Unavailable status code
    let request_url = reqwest::Url::parse(url)
        .expect(&format!("The harcoded URL `{url}` should always be valid"));
    let request_method = reqwest::Method::GET;
    let request = Request::new(request_method, request_url);

    let expected_result = reqwest::StatusCode::SERVICE_UNAVAILABLE;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!(
            "A request to the URL `{url}` should always succeed"
        ))
        .status();

    drop(client);
    assert_eq!(result, expected_result);
}
