//! # Raw Document Metadata Context
//!
//! Provides [`RawDocumentMetadataContext`], the metadata fields that only a
//! [`RawDocument`](super::RawDocument) has.

use serde::{Deserialize, Serialize};

use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::{FileBody, SecResponse};
use crate::shared::status_code::StatusCode;
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;

/// The metadata fields that only a [`RawDocument`](super::RawDocument) has: the request that
/// fetched it.
///
/// The repository writes these fields as the nested `context` of the metadata file. On read, the
/// URL and the user agent pass the same checks as on a fetch.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RawDocumentMetadataContext {
    /// The URL the fetch sent. It equals [`final_url`](Self::final_url) when the SEC does not
    /// redirect.
    pub requested_url: Url,
    /// The URL the response came from, after any redirects. It equals
    /// [`requested_url`](Self::requested_url) when the SEC does not redirect.
    pub final_url: Url,
    /// The HTTP status of the final response, after any redirects.
    pub http_status: StatusCode,
    /// The `User-Agent` the fetch sent.
    pub user_agent: UserAgent,
}

impl RawDocumentMetadataContext {
    /// Creates a new [`RawDocumentMetadataContext`] from a response, and the URL and the
    /// `User-Agent` the fetch sent.
    ///
    /// The final URL comes from [`SecResponse::url`](SecResponseTrait::url).
    #[must_use]
    pub fn new(response: &SecResponse, requested_url: &Url, user_agent: &UserAgent) -> Self {
        Self {
            requested_url: requested_url.clone(),
            final_url: response.url().clone(),
            http_status: response.status_code(),
            user_agent: user_agent.clone(),
        }
    }

    /// Creates a new [`RawDocumentMetadataContext`] from a raw-bytes response, such as a Notes
    /// file, a daily index, or an XML file of a filing, and the URL and the `User-Agent` the fetch sent.
    ///
    /// The final URL comes from [`SecResponse::url`](SecResponseTrait::url).
    #[must_use]
    pub fn from_file<B: FileBody>(
        file: &SecResponse<B>,
        requested_url: &Url,
        user_agent: &UserAgent,
    ) -> Self {
        Self {
            requested_url: requested_url.clone(),
            final_url: file.url().clone(),
            http_status: file.status_code(),
            user_agent: user_agent.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::response::implementations::sec_response::ZipBody;
    use crate::shared::user_agent::{UserAgentError, UserAgentErrorReason};

    fn sample_user_agent() -> UserAgent {
        UserAgent::new("Sample Company contact@example.com")
            .expect("A hardcoded SEC-format string should always validate")
    }

    fn sample_requested_url() -> Url {
        "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
            .parse()
            .expect("A hardcoded valid URL string should always parse")
    }

    fn sample_final_url() -> Url {
        "https://data.sec.gov/api/xbrl/companyfacts/redirected/CIK0000320193.json"
            .parse()
            .expect("A hardcoded valid URL string should always parse")
    }

    fn sample_context() -> RawDocumentMetadataContext {
        RawDocumentMetadataContext {
            requested_url: sample_requested_url(),
            final_url: sample_final_url(),
            http_status: StatusCode::Ok,
            user_agent: sample_user_agent(),
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_raw_document_metadata_context() {
        implements_auto_traits::<RawDocumentMetadataContext>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_raw_document_metadata_context() {
        implements_value_traits::<RawDocumentMetadataContext>();
    }

    fn sample_redirected_response() -> SecResponse {
        SecResponse::from_parts(
            sample_final_url(),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            b"{}".to_vec(),
        )
        .expect("Given the hardcoded JSON body '{}', building the response should always succeed")
    }

    #[test]
    fn should_put_the_requested_url_in_requested_url_when_creating_a_context() {
        let response = sample_redirected_response();
        let requested_url = sample_requested_url();
        let user_agent = sample_user_agent();

        let expected_result = sample_requested_url();

        let result =
            RawDocumentMetadataContext::new(&response, &requested_url, &user_agent).requested_url;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_put_the_response_url_in_final_url_when_creating_a_context() {
        let response = sample_redirected_response();
        let requested_url = sample_requested_url();
        let user_agent = sample_user_agent();

        let expected_result = response.url().clone();

        let result =
            RawDocumentMetadataContext::new(&response, &requested_url, &user_agent).final_url;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_status_from_the_response_when_creating_a_context() {
        let response = sample_redirected_response();
        let requested_url = sample_requested_url();
        let user_agent = sample_user_agent();

        let expected_result = response.status_code();

        let result =
            RawDocumentMetadataContext::new(&response, &requested_url, &user_agent).http_status;

        assert_eq!(result, expected_result);
    }

    fn sample_redirected_notes_file() -> SecResponse<ZipBody> {
        SecResponse::<ZipBody>::from_parts(
            sample_notes_final_url(),
            Headers::new(HashMap::from([(
                "content-type".to_string(),
                "application/zip".to_string(),
            )])),
            ContentType::Zip,
            StatusCode::Ok,
            b"PK".to_vec(),
        )
        .expect("Given status 200 and a ZIP, building the response should always succeed")
    }

    fn sample_notes_requested_url() -> Url {
        "https://www.sec.gov/files/dera/data/financial-statement-and-notes-data-sets/2024q1_notes.zip"
            .parse()
            .expect("A hardcoded valid URL string should always parse")
    }

    fn sample_notes_final_url() -> Url {
        "https://www.sec.gov/files/dera/data/financial-statement-and-notes-data-sets/redirected/2024q1_notes.zip"
            .parse()
            .expect("A hardcoded valid URL string should always parse")
    }

    #[test]
    fn should_put_the_requested_url_in_requested_url_when_creating_a_context_from_a_file() {
        let file = sample_redirected_notes_file();
        let requested_url = sample_notes_requested_url();
        let user_agent = sample_user_agent();

        let expected_result = sample_notes_requested_url();

        let result =
            RawDocumentMetadataContext::from_file(&file, &requested_url, &user_agent).requested_url;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_put_the_file_url_in_final_url_when_creating_a_context_from_a_file() {
        let file = sample_redirected_notes_file();
        let requested_url = sample_notes_requested_url();
        let user_agent = sample_user_agent();

        let expected_result = sample_notes_final_url();

        let result =
            RawDocumentMetadataContext::from_file(&file, &requested_url, &user_agent).final_url;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_four_fields_as_plain_values_for_raw_document_metadata_context() {
        let context = sample_context();

        let expected_result = serde_json::json!({
            "requested_url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json",
            "final_url": "https://data.sec.gov/api/xbrl/companyfacts/redirected/CIK0000320193.json",
            "http_status": 200,
            "user_agent": "Sample Company contact@example.com",
        });

        let result = serde_json::to_value(context)
            .expect("Given a context of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_same_context_when_deserializing_a_serialized_context() {
        let json = serde_json::to_string(&sample_context())
            .expect("Given a context of domain types, serialization should always succeed");

        let expected_result = sample_context();

        let result: RawDocumentMetadataContext = serde_json::from_str(&json)
            .expect("Given JSON this type produced, deserialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_deserialize_when_the_user_agent_has_no_email() {
        let json = r#"{
            "requested_url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json",
            "final_url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json",
            "http_status": 200,
            "user_agent": "Sample Company"
        }"#;

        let expected_result =
            UserAgentError::new(UserAgentErrorReason::InvalidSecFormat, "Sample Company")
                .to_string();

        let result = serde_json::from_str::<RawDocumentMetadataContext>(json)
            .expect_err("Given a user agent without an email, deserialization should always fail")
            .to_string();

        assert!(result.starts_with(&expected_result));
    }
}
