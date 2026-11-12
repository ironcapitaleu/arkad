//! # gzip Body Integration Tests
//!
//! Downloads the same file from a local server, once as a gzip body and once uncompressed, and
//! checks that both give the same bytes and the same SHA-256. The store writes these bytes, and a
//! rerun skips a stored file by its size and SHA-256, so a store written without gzip stays valid.
//!
//! ## External Dependencies
//!
//! None. The server runs on the loopback interface.

use pretty_assertions::assert_eq;
use reqwest::{Client, ClientBuilder, Method, Request, Url};

use sec::shared::http_client::InnerClient;
use sec::shared::http_client::implementations::reqwest_client::ReqwestClient;
use sec::shared::response::SecResponse as SecResponseTrait;
use sec::shared::response::implementations::sec_response::{SecResponse, ZipBody};
use sec::shared::user_agent::UserAgent;
use sec::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
use sec::tests::fixtures::sample_server::gzip_server;

/// The content type of a Notes file.
const ZIP_CONTENT_TYPE: &str = "application/zip";

/// The SHA-256 of [`SAMPLE_NOTES_ZIP`].
const SAMPLE_NOTES_ZIP_SHA256: &str =
    "c1109f90f92dfdd7e398132e45586b893814810ceb7b858090c846e031c0a1b9";

/// The bytes, the SHA-256, and the `Content-Length` header of a downloaded file.
type Download = (Vec<u8>, String, Option<String>);

/// Downloads [`SAMPLE_NOTES_ZIP`] from a [`gzip_server`] with a [`ReqwestClient`] built from
/// `builder`.
async fn download(builder: ClientBuilder) -> Download {
    let address = gzip_server(ZIP_CONTENT_TYPE, SAMPLE_NOTES_ZIP).await;
    let user_agent = UserAgent::new("Sample Company contact@example.com")
        .expect("A hardcoded SEC-format string should always validate");
    let client = ReqwestClient::from_builder(builder.no_proxy(), user_agent)
        .expect("Given a validated user agent, building the HTTP client fails only if the TLS backend cannot initialize");
    let url = Url::parse(&format!("http://{address}/"))
        .expect("A URL built from a bound socket address always parses");
    let inner = client
        .execute_request(Request::new(Method::GET, url))
        .await
        .expect("The local server always answers with a full response");
    let response = SecResponse::<ZipBody>::from_inner(inner).await.expect(
        "Given status 200, a ZIP content type, and a whole body, building the response should always succeed",
    );
    (
        response.body_bytes().to_vec(),
        response.body_digest().to_string(),
        response.headers().get("content-length").map(str::to_string),
    )
}

#[tokio::test]
async fn should_keep_the_bytes_and_drop_the_content_length_when_the_client_asks_for_gzip() {
    // `ReqwestClient::new` starts from this builder, so it asks for gzip too.
    let builder = Client::builder();

    // `reqwest` removes `Content-Length` only when it decodes a gzip body.
    let expected_result: Download = (
        SAMPLE_NOTES_ZIP.to_vec(),
        SAMPLE_NOTES_ZIP_SHA256.to_string(),
        None,
    );

    let result = download(builder).await;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_keep_the_bytes_and_check_the_content_length_when_the_client_does_not_ask_for_gzip()
{
    let builder = Client::builder().no_gzip();

    let expected_result: Download = (
        SAMPLE_NOTES_ZIP.to_vec(),
        SAMPLE_NOTES_ZIP_SHA256.to_string(),
        Some("150".to_string()),
    );

    let result = download(builder).await;

    assert_eq!(result, expected_result);
}
