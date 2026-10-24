//! # Raw Document Metadata Context
//!
//! Provides [`RawDocumentMetadataContext`], the metadata fields that only a
//! [`RawDocument`](super::RawDocument) has.

use serde::{Deserialize, Serialize};

use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::shared::status_code::StatusCode;
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;

/// The metadata fields that only a [`RawDocument`](super::RawDocument) has: the request that
/// fetched it.
///
/// The repository writes these fields as the nested `context` of the metadata file. Each field is
/// a domain type, so a metadata file read back passes the same checks as a fetch.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RawDocumentMetadataContext {
    /// The URL the fetch requested.
    pub url: Url,
    /// The HTTP status the fetch returned.
    pub http_status: StatusCode,
    /// The `User-Agent` the fetch sent.
    pub user_agent: UserAgent,
}

impl RawDocumentMetadataContext {
    /// Creates a new [`RawDocumentMetadataContext`] from a response and the `User-Agent` the fetch
    /// sent.
    #[must_use]
    pub fn new(response: &SecResponse, user_agent: &UserAgent) -> Self {
        Self {
            url: response.url().clone(),
            http_status: response.status_code(),
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
    use crate::shared::user_agent::{UserAgentError, UserAgentErrorReason};

    fn sample_url() -> Url {
        "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
            .parse()
            .expect("A hardcoded valid URL string should always parse")
    }

    fn sample_user_agent() -> UserAgent {
        UserAgent::new("Sample Company contact@example.com")
            .expect("A hardcoded SEC-format string should always validate")
    }

    fn sample_context() -> RawDocumentMetadataContext {
        RawDocumentMetadataContext {
            url: sample_url(),
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

    #[test]
    fn should_take_the_url_and_the_status_from_the_response_when_creating_a_context() {
        let response = SecResponse::from_parts(
            sample_url(),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            serde_json::json!({}),
        );
        let user_agent = sample_user_agent();

        let expected_result = sample_context();

        let result = RawDocumentMetadataContext::new(&response, &user_agent);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_three_fields_as_plain_values_for_raw_document_metadata_context() {
        let context = sample_context();

        let expected_result = serde_json::json!({
            "url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json",
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
            "url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json",
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
