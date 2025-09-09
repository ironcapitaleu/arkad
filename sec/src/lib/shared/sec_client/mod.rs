//! # SEC Client Module
//!
//! This module provides the [`SecClient`] type and related utilities for creating and managing
//! HTTP clients specifically configured for interacting with SEC (Securities and Exchange Commission)
//! services. It ensures that all requests are made with proper user agent identification as required
//! by SEC guidelines.
//!
//! ## Overview
//! The [`SecClient`] is a wrapper around [`reqwest::Client`] that enforces SEC-compliant user agent
//! formatting and provides unique identification for each client instance. It is designed to be used
//! throughout the SEC state machine library for making HTTP requests to SEC endpoints.
//!
//! ## Types
//! - [`SecClient`]: Main client wrapper with unique ID and configured HTTP client.
//! - [`SecClientError`], [`SecClientErrorReason`]: Error types for client creation failures.
//!
//! ## See Also
//! - [`super::user_agent`]: User agent validation and formatting utilities.
//! - [`reqwest::Client`]: Underlying HTTP client implementation.

pub mod sec_client_error;
pub use sec_client_error::{SecClientError, SecClientErrorReason};

use reqwest::{Client, ClientBuilder};
use uuid::Uuid;

use super::sec_request::SecRequest;
use super::sec_response::SecResponse;
use super::user_agent::UserAgent;

/// A wrapper around [`reqwest::Client`] configured for SEC-compliant HTTP requests.
///
/// `SecClient` ensures that all HTTP requests are made with a properly formatted user agent
/// string as required by SEC guidelines. Each client instance has a unique identifier for
/// tracking and debugging purposes.
///
/// # Examples
///
/// ```rust
/// use sec::shared::sec_client::SecClient;
///
/// // Create a new client with SEC-compliant user agent
/// let client = SecClient::new("Sample Corp contact@sample.com")?;
///
/// // Access the unique client ID
/// let id = client.id();
///
/// // Get the underlying HTTP client for making requests
/// let http_client = client.client();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # User Agent Requirements
/// The user agent string must follow the SEC format: "Company Name email@domain.com"
/// - Company name can contain letters, numbers, spaces, and common punctuation
/// - Must be followed by a space and a valid email address
/// - Email must have proper domain extension (minimum 2 characters)
#[derive(Debug, Clone)]
pub struct SecClient {
    /// Unique identifier for this client instance.
    pub id: String,
    /// The underlying reqwest HTTP client.
    pub inner: reqwest::Client,
}

impl SecClient {
    /// Creates a new `SecClient` with a unique ID and SEC-compliant user agent.
    ///
    /// This method validates the provided user agent string to ensure it meets SEC requirements,
    /// creates a configured HTTP client, and assigns a unique UUID to the client instance.
    ///
    /// # Arguments
    ///
    /// * `user_agent` - A string slice containing the user agent in SEC format:
    ///   "Company Name email@domain.com"
    ///
    /// # Returns
    ///
    /// Returns `Ok(SecClient)` if the user agent is valid and the client was created successfully,
    /// or `Err(SecClientError)` if validation fails or client creation fails.
    ///
    /// # Errors
    ///
    /// This method can return the following errors:
    /// - [`SecClientErrorReason::InvalidUserAgent`] - If the user agent string doesn't meet SEC format requirements
    /// - [`SecClientErrorReason::ReqwestClientCreationFailed`] - If the underlying reqwest client cannot be created
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sec::shared::sec_client::SecClient;
    ///
    /// // Valid user agent
    /// let client = SecClient::new("Sample Corp contact@sample.com")?;
    /// assert!(!client.id().is_empty());
    ///
    /// // Invalid user agent (missing email)
    /// let result = SecClient::new("Sample Corp");
    /// assert!(result.is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # SEC User Agent Format
    ///
    /// The SEC requires user agents to identify the requesting party with:
    /// - A descriptive company or application name
    /// - A valid contact email address
    /// - Proper spacing between name and email
    pub fn new(user_agent: &str) -> Result<Self, SecClientError> {
        let user_agent = UserAgent::new(user_agent);
        let user_agent = match user_agent {
            Ok(user_agent) => user_agent.inner().to_owned(),
            Err(e) => {
                return Err(SecClientError::new(
                    SecClientErrorReason::InvalidUserAgent,
                    e.user_agent,
                ));
            }
        };

        let user_agent_str = user_agent.clone();
        let client = ClientBuilder::new().user_agent(user_agent_str).build();

        let Ok(client) = client else {
            return Err(SecClientError {
                reason: SecClientErrorReason::ReqwestClientCreationFailed,
                user_agent,
            });
        };

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            inner: client,
        })
    }

    /// Future method to execute SEC requests using the internal client.
    pub async fn execute_request(&self, request: SecRequest) -> SecResponse {
        let resp = self.inner.execute(request.inner).await.unwrap();
        SecResponse::new(resp)
    }

    /// Returns the unique identifier for this client instance.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns a reference to the underlying HTTP client.
    #[must_use]
    pub const fn client(&self) -> &Client {
        &self.inner
    }
}

/// Provides equality for `SecClient` instances based on their unique IDs.
impl PartialEq for SecClient {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Provides partial ordering for `SecClient` instances based on their unique IDs.
impl PartialOrd for SecClient {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Provides total ordering for `SecClient` instances based on their unique IDs.
impl Ord for SecClient {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

/// Marks `SecClient` as having full equality semantics.
impl Eq for SecClient {}

/// Provides hashing for `SecClient` instances based on their unique ID.
impl std::hash::Hash for SecClient {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Provides a default `SecClient` instance for testing and fallback scenarios.
///
/// The default client uses a generic user agent and a hardcoded ID. This should
/// primarily be used for testing purposes or as a fallback when a specific client
/// configuration is not required.
///
/// # Warning
///
/// The default implementation creates a client with ID "default" and a basic
/// reqwest client without user agent validation. For production use, always
/// create clients using [`SecClient::new`] with proper user agent strings.
impl Default for SecClient {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            inner: Client::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_sec_client_when_valid_user_agent_is_provided() {
        let user_agent = "Sample Corp contact@sample.com";

        let result = SecClient::new(user_agent);

        assert!(result.is_ok());
        let client = result.unwrap();
        assert!(!client.id().is_empty());
        assert!(uuid::Uuid::parse_str(client.id()).is_ok());
    }

    #[test]
    fn should_return_error_when_invalid_user_agent_is_provided() {
        let user_agent = "Invalid User Agent"; // Missing email
        let expected_error_reason = SecClientErrorReason::InvalidUserAgent;

        let result = SecClient::new(user_agent);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.reason, expected_error_reason);
    }

    #[test]
    fn should_generate_unique_ids_when_multiple_clients_are_created() {
        let user_agent = "Test Company test@company.com";

        let client1 = SecClient::new(user_agent).unwrap();
        let client2 = SecClient::new(user_agent).unwrap();

        assert_ne!(client1.id(), client2.id());
    }

    #[test]
    fn should_return_client_id_when_id_method_is_called() {
        let user_agent = "Test Company test@company.com";
        let client = SecClient::new(user_agent).unwrap();
        let expected_result = client.id.clone();

        let result = client.id();

        assert_eq!(result, &expected_result);
    }

    #[test]
    fn should_return_inner_client_when_client_method_is_called() {
        let user_agent = "Test Company test@company.com";
        let sec_client = SecClient::new(user_agent).unwrap();

        let result = sec_client.client();

        // We can't directly compare reqwest::Client instances, but we can verify
        // that we got a client back
        assert!(std::ptr::eq(result, &sec_client.inner));
    }

    #[test]
    fn should_create_default_client_when_default_is_called() {
        let expected_id = "default";

        let result = SecClient::default();

        assert_eq!(result.id(), expected_id);
    }

    #[test]
    fn should_return_error_when_user_agent_has_invalid_email_format() {
        let user_agent = "Test Company invalid-email";
        let expected_error_reason = SecClientErrorReason::InvalidUserAgent;

        let result = SecClient::new(user_agent);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.reason, expected_error_reason);
    }

    #[test]
    fn should_return_error_when_user_agent_is_empty() {
        let user_agent = "";
        let expected_error_reason = SecClientErrorReason::InvalidUserAgent;

        let result = SecClient::new(user_agent);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.reason, expected_error_reason);
    }

    #[test]
    fn should_create_client_when_email_has_plus_sign() {
        let user_agent = "Test Company admin+sec@company.com";

        let result = SecClient::new(user_agent);

        assert!(result.is_ok());
        let client = result.unwrap();
        assert!(!client.id().is_empty());
    }

    #[test]
    fn should_create_client_when_email_has_subdomain() {
        let user_agent = "Research Team research@api.university.edu";

        let result = SecClient::new(user_agent);

        assert!(result.is_ok());
        let client = result.unwrap();
        assert!(!client.id().is_empty());
    }
}
