//! # `SecClient` Integration Tests
//!
//! Validates that [`SecClient`] correctly executes requests against the live SEC EDGAR API and
//! returns properly parsed responses.
//!
//! ## External Dependencies
//!
//! - `data.sec.gov` — Live SEC EDGAR API (JSON company filings)
//! - `www.sec.gov` — Live Notes files, asked for with `HEAD` requests

use chrono::{Datelike, Months, NaiveDate, Utc};
use pretty_assertions::assert_eq;
use serde_json::Value;

use sec::shared::cik::Cik;
use sec::shared::content_type::ContentType;
use sec::shared::http_client::SecClient as SecClientTrait;
use sec::shared::http_client::implementations::reqwest_client::ReqwestClient;
use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::http_client::implementations::sec_client::error::ErrorReason;
use sec::shared::notes_period::NotesPeriod;
use sec::shared::request::implementations::sec_request::SecRequest;
use sec::shared::response::SecResponse as SecResponseTrait;
use sec::shared::response::implementations::sec_response::error::{
    ErrorReason as SecResponseErrorReason, InvalidSecResponse,
};
use sec::shared::status_code::StatusCode;
use sec::shared::user_agent::UserAgent;

const BERKSHIRE_FIXTURE: &str =
    include_str!("../src/lib/tests/fixtures/data/raw_input/CIK0001067983.json");

/// The age in months after which the SEC has merged the monthly Notes files of every quarter so
/// far. The slowest merge measured took 458 days, about 15 months.
const MERGED_AFTER_MONTHS: u32 = 16;

/// Returns the last quarter that ended before the quarter of `date`.
fn quarter_before(date: NaiveDate) -> NotesPeriod {
    let quarter = (date.month() - 1) / 3;
    let (year, quarter) = if quarter == 0 {
        (date.year() - 1, 4)
    } else {
        (date.year(), quarter)
    };
    NotesPeriod::new(&format!("{year}q{quarter}")).expect(
        "Given a year from 2009 on and a quarter from 1 to 4, parsing should always succeed",
    )
}

/// Creates an `SecClient` for testing, with connection pooling off. The tests do not reuse
/// connections.
fn sec_client() -> SecClient {
    let user_agent = UserAgent::new("ArkadTest admin@example.com")
        .expect("A hardcoded SEC-format string should always validate");
    let builder = reqwest::Client::builder().pool_max_idle_per_host(0);
    let http_client = ReqwestClient::from_builder(builder, user_agent)
        .expect("Given a validated user agent, building the HTTP client fails only if the TLS backend cannot initialize");
    SecClient::new(http_client)
}

#[tokio::test]
async fn should_return_sec_response_when_executing_valid_sec_request() {
    let client = sec_client();
    let cik = Cik::new("1067983").expect("A hardcoded CIK should always be valid");
    let request = SecRequest::builder().all_company_facts().cik(cik).build();

    let expected_result = None;

    let result = client.execute_sec_request(request).await.err();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_succeed_when_probing_a_published_notes_file() {
    let client = sec_client();
    let period = NotesPeriod::new("2025q3").expect("A hardcoded period should always be valid");
    let request = SecRequest::builder()
        .notes_file_probe()
        .period(period)
        .build();

    let expected_result = None;

    let result = client.execute_sec_request(request).await.err();

    assert_eq!(result, expected_result);
}

// Early warning for `MERGE_WAIT_MONTHS` (24) in `published_periods`: if the SEC stops merging, or
// merges later, this test fails months before a backfill run counts a missing quarter as merged.
#[tokio::test]
async fn should_find_the_quarterly_file_of_a_quarter_that_ended_16_months_ago() {
    let client = sec_client();
    let months_ago = Utc::now()
        .date_naive()
        .checked_sub_months(Months::new(MERGED_AFTER_MONTHS))
        .expect("Given today, a date 16 months earlier should always exist");
    let quarter = quarter_before(months_ago);
    let request = SecRequest::builder()
        .notes_file_probe()
        .period(quarter)
        .build();

    let expected_result = None;

    let result = client.execute_sec_request(request).await.err();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_report_not_found_when_probing_an_unpublished_month() {
    let client = sec_client();
    let period = NotesPeriod::new("2099_12").expect("A hardcoded period should always be valid");
    let request = SecRequest::builder()
        .notes_file_probe()
        .period(period)
        .build();

    let expected_result = Some(true);

    let result = client
        .execute_sec_request(request)
        .await
        .err()
        .map(|error| error.is_not_found());

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_json_content_type_when_sec_request_succeeds() {
    let client = sec_client();
    let cik = Cik::new("1067983").expect("A hardcoded CIK should always be valid");
    let request = SecRequest::builder().all_company_facts().cik(cik).build();

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
    let request = SecRequest::builder().all_company_facts().cik(cik).build();

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
    let request = SecRequest::builder().all_company_facts().cik(cik).build();
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

#[tokio::test]
async fn should_return_expected_cik_when_retrieving_berkshire_company_facts() {
    let client = sec_client();
    let cik = Cik::new("1067983").expect("A hardcoded CIK should always be valid");
    let request = SecRequest::builder().all_company_facts().cik(cik).build();
    let fixture: Value = serde_json::from_str(BERKSHIRE_FIXTURE)
        .expect("The Berkshire Hathaway fixture should always be valid JSON");

    let expected_result = fixture["cik"].clone();

    let result = client
        .execute_sec_request(request)
        .await
        .expect("A valid SEC request should return a valid SecResponse")
        .body()["cik"]
        .clone();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_expected_entity_name_when_retrieving_berkshire_company_facts() {
    let client = sec_client();
    let cik = Cik::new("1067983").expect("A hardcoded CIK should always be valid");
    let request = SecRequest::builder().all_company_facts().cik(cik).build();
    let fixture: Value = serde_json::from_str(BERKSHIRE_FIXTURE)
        .expect("The Berkshire Hathaway fixture should always be valid JSON");

    let expected_result = fixture["entityName"].clone();

    let result = client
        .execute_sec_request(request)
        .await
        .expect("A valid SEC request should return a valid SecResponse")
        .body()["entityName"]
        .clone();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_expected_top_level_keys_when_retrieving_berkshire_company_facts() {
    let client = sec_client();
    let cik = Cik::new("1067983").expect("A hardcoded CIK should always be valid");
    let request = SecRequest::builder().all_company_facts().cik(cik).build();
    let fixture: Value = serde_json::from_str(BERKSHIRE_FIXTURE)
        .expect("The Berkshire Hathaway fixture should always be valid JSON");
    let fixture_keys: Vec<&str> = fixture
        .as_object()
        .expect("The fixture should be a JSON object")
        .keys()
        .map(String::as_str)
        .collect();

    let expected_result = fixture_keys;

    let body = client
        .execute_sec_request(request)
        .await
        .expect("A valid SEC request should return a valid SecResponse");
    let result: Vec<&str> = body
        .body()
        .as_object()
        .expect("The response body should be a JSON object")
        .keys()
        .map(String::as_str)
        .collect();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_expected_facts_keys_when_retrieving_berkshire_company_facts() {
    let client = sec_client();
    let cik = Cik::new("1067983").expect("A hardcoded CIK should always be valid");
    let request = SecRequest::builder().all_company_facts().cik(cik).build();
    let fixture: Value = serde_json::from_str(BERKSHIRE_FIXTURE)
        .expect("The Berkshire Hathaway fixture should always be valid JSON");
    let fixture_keys: Vec<&str> = fixture["facts"]
        .as_object()
        .expect("The fixture 'facts' field should be a JSON object")
        .keys()
        .map(String::as_str)
        .collect();

    let expected_result = fixture_keys;

    let body = client
        .execute_sec_request(request)
        .await
        .expect("A valid SEC request should return a valid SecResponse");
    let result: Vec<&str> = body.body()["facts"]
        .as_object()
        .expect("The response 'facts' field should be a JSON object")
        .keys()
        .map(String::as_str)
        .collect();

    assert_eq!(result, expected_result);
}
