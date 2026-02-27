use pretty_assertions::assert_eq;
use reqwest::Request;

use sec::shared::http_client::InnerClient;
use sec::shared::http_client::implementations::reqwest_client::ReqwestClient;

#[tokio::test]
async fn should_return_successful_response_for_valid_request() {
    let client = ReqwestClient::default();
    let url = "https://httpbin.org/get";
    let request = Request::new(reqwest::Method::GET, reqwest::Url::parse(url).unwrap());

    let expected_result = reqwest::StatusCode::OK;

    let result = client
        .execute_request(request)
        .await
        .expect(&format!("The URL `{url}` should always succeed"))
        .status();

    assert_eq!(result, expected_result);
}
