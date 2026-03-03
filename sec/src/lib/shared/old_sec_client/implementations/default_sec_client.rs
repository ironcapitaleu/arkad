//! # Default SEC Client Implementation
//!
//! This module provides the default implementation of the [`SecClient`] trait using
//! a generic HTTP client. This implementation can work with any type that implements
//! the [`HttpClient`] trait.
//!
//! ## Types
//! - [`DefaultSecClient`]: Generic implementation of [`SecClient`] trait.
//!
//! ## See Also
//! - [`super::super::traits::SecClient`]: The trait this implementation fulfills.
//! - [`super::super::traits::HttpClient`]: The trait constraint for the inner HTTP client.

use async_trait::async_trait;
use reqwest::ClientBuilder;
use uuid::Uuid;

use super::super::sec_client_error::{SecClientError, SecClientErrorReason};
use super::super::traits::{HttpClient, SecClient};
use super::reqwest_http_client::ReqwestHttpClient;
use crate::shared::sec_request::SecRequest;
use crate::shared::sec_request::implementations::reqwest_request::ReqwestRequest;
use crate::shared::sec_request::sec_request_error::SecRequestError;
use crate::shared::sec_response::SecResponse;
use crate::shared::user_agent::UserAgent;

/// Default implementation of [`SecClient`] trait.
///
/// `DefaultSecClient` is a generic implementation that can work with any HTTP client
/// that implements the [`HttpClient`] trait. Each client instance has a unique identifier
/// for tracking and debugging purposes.
///
/// # Type Parameters
/// - `C`: The underlying HTTP client type, must implement [`HttpClient`].
///
/// # Examples
///
/// ```rust
/// use sec::shared::old_sec_client::DefaultSecClient;
/// use sec::shared::old_sec_client::traits::SecClient;
///
/// // Create a default client with reqwest
/// let client = DefaultSecClient::new("Sample Corp contact@sample.com")?;
///
/// // Access the unique client ID
/// let id = client.id();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct DefaultSecClient<C: HttpClient> {
    /// Unique identifier for this client instance.
    pub id: String,
    /// The underlying HTTP client implementation.
    pub inner: C,
}

impl DefaultSecClient<ReqwestHttpClient> {
    /// Creates a new `DefaultSecClient` with a unique ID and SEC-compliant user agent.
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
    /// Returns `Ok(DefaultSecClient)` if the user agent is valid and the client was created successfully,
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
    /// use sec::shared::old_sec_client::DefaultSecClient;
    /// use sec::shared::old_sec_client::traits::SecClient;
    ///
    /// // Valid user agent
    /// let client = DefaultSecClient::new("Sample Corp contact@sample.com")?;
    /// assert!(!client.id().is_empty());
    ///
    /// // Invalid user agent (missing email)
    /// let result = DefaultSecClient::new("Sample Corp");
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

impl<C: HttpClient> DefaultSecClient<C> {
    /// Creates a new `DefaultSecClient` with a custom HTTP client implementation.
    ///
    /// # Arguments
    ///
    /// * `http_client` - An implementation of the [`HttpClient`] trait.
    ///
    /// # Returns
    ///
    /// Returns a new `DefaultSecClient` instance with the provided HTTP client.
    #[must_use]
    pub fn with_http_client(http_client: C) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            inner: http_client,
        }
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

#[async_trait]
impl<C: HttpClient> SecClient for DefaultSecClient<C> {
    type Inner = C;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn id(&self) -> &str {
        &self.id
    }

    async fn execute_request(
        &self,
        request: SecRequest<ReqwestRequest>,
    ) -> Result<SecResponse, SecRequestError> {
        self.inner.execute_request(request).await
    }
}

/// Provides equality for `DefaultSecClient` instances based on their unique IDs.
impl<C: HttpClient> PartialEq for DefaultSecClient<C> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Provides partial ordering for `DefaultSecClient` instances based on their unique IDs.
impl<C: HttpClient> PartialOrd for DefaultSecClient<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Provides total ordering for `DefaultSecClient` instances based on their unique IDs.
impl<C: HttpClient> Ord for DefaultSecClient<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

/// Marks `DefaultSecClient` as having full equality semantics.
impl<C: HttpClient> Eq for DefaultSecClient<C> {}

/// Provides hashing for `DefaultSecClient` instances based on their unique ID.
impl<C: HttpClient> std::hash::Hash for DefaultSecClient<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Provides a default `DefaultSecClient` instance for testing and fallback scenarios.
///
/// The default client uses a generic user agent and a hardcoded ID. This should
/// primarily be used for testing purposes or as a fallback when a specific client
/// configuration is not required.
///
/// # Warning
///
/// The default implementation creates a client with ID "default" and a basic
/// reqwest client without user agent validation. For production use, always
/// create clients using [`DefaultSecClient::new`] with proper user agent strings.
impl Default for DefaultSecClient<ReqwestHttpClient> {
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

        let result = DefaultSecClient::new(user_agent);

        assert!(result.is_ok());
    }

    #[test]
    fn should_return_error_when_invalid_user_agent_is_provided() {
        let user_agent = "Invalid User Agent";
        let expected_result = SecClientErrorReason::InvalidUserAgent;

        let result = DefaultSecClient::new(user_agent)
            .expect_err("Given an invalid user agent, DefaultSecClient creation should fail")
            .reason;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_generate_unique_ids_when_multiple_clients_are_created() {
        let user_agent = "Test Company test@company.com";
        let client1 = DefaultSecClient::new(user_agent)
            .expect("Given a valid user agent, DefaultSecClient creation should succeed");
        let client2 = DefaultSecClient::new(user_agent)
            .expect("Given a valid user agent, DefaultSecClient creation should succeed");

        let result = client1.id();

        assert_ne!(result, client2.id());
    }

    #[test]
    fn should_return_client_id_when_id_method_is_called() {
        let user_agent = "Test Company test@company.com";
        let client = DefaultSecClient::new(user_agent)
            .expect("Given a valid user agent, DefaultSecClient creation should succeed");
        let expected_result = client.id.clone();

        let result = client.id().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_sec_client_when_custom_http_client_is_provided() {
        let reqwest_client = reqwest::Client::new();
        let http_client = ReqwestHttpClient::new(reqwest_client);

        let result = DefaultSecClient::with_http_client(http_client);

        assert!(!result.id().is_empty());
    }

    #[test]
    fn should_create_default_client_when_default_is_called() {
        let expected_result = "default";

        let result = DefaultSecClient::<ReqwestHttpClient>::default();
        let result = result.id();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_user_agent_has_invalid_email_format() {
        let user_agent = "Test Company invalid-email";
        let expected_result = SecClientErrorReason::InvalidUserAgent;

        let result = DefaultSecClient::new(user_agent)
            .expect_err("Given an invalid user agent, DefaultSecClient creation should fail")
            .reason;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_user_agent_is_empty() {
        let user_agent = "";
        let expected_result = SecClientErrorReason::InvalidUserAgent;

        let result = DefaultSecClient::new(user_agent)
            .expect_err("Given an empty user agent, DefaultSecClient creation should fail")
            .reason;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_client_when_email_has_plus_sign() {
        let user_agent = "Test Company admin+sec@company.com";

        let result = DefaultSecClient::new(user_agent);

        assert!(result.is_ok());
    }

    #[test]
    fn should_create_client_when_email_has_subdomain() {
        let user_agent = "Research Team research@api.university.edu";

        let result = DefaultSecClient::new(user_agent);

        assert!(result.is_ok());
    }
}
