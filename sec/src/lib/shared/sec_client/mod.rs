//! # SEC Client Module
//!
//! This module provides the [`SecClient`] type and related utilities for creating and managing
//! HTTP clients specifically configured for interacting with SEC (Securities and Exchange Commission)
//! services. It ensures that all requests are made with proper user agent identification as required
//! by SEC guidelines.
//!
//! ## Overview
//! The [`SecClient`] uses dependency injection via the [`HttpClient`] trait, allowing for flexible
//! HTTP client implementations. By default, it uses [`ReqwestHttpClient`] which wraps [`reqwest::Client`],
//! but custom implementations can be provided as needed.
//!
//! ## Types
//! - [`SecClient`]: Main client wrapper with unique ID and configurable HTTP client.
//! - [`HttpClient`]: Trait defining the HTTP client interface for dependency injection.
//! - [`ReqwestHttpClient`]: Default implementation using reqwest.
//! - [`SecClientError`], [`SecClientErrorReason`]: Error types for client creation failures.
//!
//! ## Modules
//! - [`traits`]: HTTP client trait definitions.
//! - [`implementations`]: Concrete HTTP client implementations.
//!
//! ## See Also
//! - [`super::user_agent`]: User agent validation and formatting utilities.
//! - [`reqwest::Client`]: Underlying HTTP client implementation.

pub mod implementations;
pub mod sec_client_error;
pub mod traits;

pub use implementations::ReqwestHttpClient;
pub use sec_client_error::{SecClientError, SecClientErrorReason};
pub use traits::HttpClient;

use super::sec_request::SecRequest;
use super::sec_request::sec_request_error::SecRequestError;
use super::sec_response::SecResponse;
use super::user_agent::UserAgent;

use reqwest::ClientBuilder;
use uuid::Uuid;

/// A client for making SEC-compliant HTTP requests.
///
/// `SecClient` ensures that all HTTP requests follow SEC guidelines and uses the [`HttpClient`]
/// trait for dependency injection, enabling flexibility in HTTP client implementations. By default,
/// it uses [`ReqwestHttpClient`].
///
/// Each client instance has a unique identifier for tracking and debugging purposes.
///
/// # User Agent Requirements
/// The user agent string must follow the SEC format: "Company Name email@domain.com"
/// - Company name can contain letters, numbers, spaces, and common punctuation
/// - Must be followed by a space and a valid email address
/// - Email must have proper domain extension (minimum 2 characters)
#[derive(Debug, Clone)]
pub struct SecClient<C: HttpClient> {
    /// Unique identifier for this client instance.
    pub id: String,
    /// The underlying HTTP client implementation.
    pub inner: C,
}

impl SecClient<ReqwestHttpClient> {
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

        let http_client = ReqwestHttpClient::new(client);

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            inner: http_client,
        })
    }
}

impl<C: HttpClient> SecClient<C> {
    /// Creates a new `SecClient` with a custom HTTP client implementation.
    ///
    /// # Arguments
    ///
    /// * `http_client` - An implementation of the [`HttpClient`] trait.
    ///
    /// # Returns
    ///
    /// Returns a new `SecClient` instance with the provided HTTP client.
    #[must_use]
    pub fn with_http_client(http_client: C) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            inner: http_client,
        }
    }

    /// Executes the given `SecRequest` using the underlying HTTP client and returns a `SecResponse`.
    ///
    /// # Arguments
    /// * `request` - The `SecRequest` to be executed.
    ///
    /// # Returns
    /// Returns a `SecResponse` containing the response data from the executed request.
    ///
    /// # Errors
    /// This method will return a `SecRequestError` if:
    /// - The HTTP request fails (network issues, timeouts, etc.)
    /// - The response body cannot be read or parsed
    /// - Any other reqwest-related error occurs during execution
    pub async fn execute_request(
        &self,
        request: SecRequest,
    ) -> Result<SecResponse, SecRequestError> {
        self.inner.execute_request(request).await
    }

    /// Returns the unique identifier for this client instance.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns a reference to the underlying HTTP client implementation.
    ///
    /// This method provides access to the underlying [`HttpClient`] implementation.
    /// Use this when you need direct access to the HTTP client for advanced scenarios.
    #[must_use]
    pub const fn http_client(&self) -> &C {
        &self.inner
    }
}

/// Provides equality for `SecClient` instances based on their unique IDs.
impl<C: HttpClient> PartialEq for SecClient<C> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Provides partial ordering for `SecClient` instances based on their unique IDs.
impl<C: HttpClient> PartialOrd for SecClient<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Provides total ordering for `SecClient` instances based on their unique IDs.
impl<C: HttpClient> Ord for SecClient<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

/// Marks `SecClient` as having full equality semantics.
impl<C: HttpClient> Eq for SecClient<C> {}

/// Provides hashing for `SecClient` instances based on their unique ID.
impl<C: HttpClient> std::hash::Hash for SecClient<C> {
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
impl Default for SecClient<ReqwestHttpClient> {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            inner: ReqwestHttpClient::new(reqwest::Client::new()),
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
    }

    #[test]
    fn should_return_error_when_invalid_user_agent_is_provided() {
        let user_agent = "Invalid User Agent";
        let expected_result = SecClientErrorReason::InvalidUserAgent;

        let result = SecClient::new(user_agent)
            .expect_err("Given an invalid user agent, SecClient creation should fail")
            .reason;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_generate_unique_ids_when_multiple_clients_are_created() {
        let user_agent = "Test Company test@company.com";
        let client1 = SecClient::new(user_agent)
            .expect("Given a valid user agent, SecClient creation should succeed");
        let client2 = SecClient::new(user_agent)
            .expect("Given a valid user agent, SecClient creation should succeed");

        let result = client1.id();

        assert_ne!(result, client2.id());
    }

    #[test]
    fn should_return_client_id_when_id_method_is_called() {
        let user_agent = "Test Company test@company.com";
        let client = SecClient::new(user_agent)
            .expect("Given a valid user agent, SecClient creation should succeed");
        let expected_result = client.id.clone();

        let result = client.id().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_sec_client_when_custom_http_client_is_provided() {
        let reqwest_client = reqwest::Client::new();
        let http_client = ReqwestHttpClient::new(reqwest_client);

        let result = SecClient::with_http_client(http_client);

        assert!(!result.id().is_empty());
    }

    #[test]
    fn should_create_default_client_when_default_is_called() {
        let expected_result = "default";

        let result = SecClient::default();
        let result = result.id();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_user_agent_has_invalid_email_format() {
        let user_agent = "Test Company invalid-email";
        let expected_result = SecClientErrorReason::InvalidUserAgent;

        let result = SecClient::new(user_agent)
            .expect_err("Given an invalid user agent, SecClient creation should fail")
            .reason;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_user_agent_is_empty() {
        let user_agent = "";
        let expected_result = SecClientErrorReason::InvalidUserAgent;

        let result = SecClient::new(user_agent)
            .expect_err("Given an empty user agent, SecClient creation should fail")
            .reason;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_client_when_email_has_plus_sign() {
        let user_agent = "Test Company admin+sec@company.com";

        let result = SecClient::new(user_agent);

        assert!(result.is_ok());
    }

    #[test]
    fn should_create_client_when_email_has_subdomain() {
        let user_agent = "Research Team research@api.university.edu";

        let result = SecClient::new(user_agent);

        assert!(result.is_ok());
    }
}
