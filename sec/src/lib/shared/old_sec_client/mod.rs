//! # SEC Client Module
//!
//! This module provides the [`SecClient`] trait and implementations for creating and managing
//! HTTP clients specifically configured for interacting with SEC (Securities and Exchange Commission)
//! services. It ensures that all requests are made with proper user agent identification as required
//! by SEC guidelines.
//!
//! ## Overview
//! The [`SecClient`] trait defines the interface for SEC-compliant HTTP clients. The default
//! implementation is [`DefaultSecClient`], which uses dependency injection via the [`HttpClient`]
//! trait, allowing for flexible HTTP client implementations. By default, it uses [`ReqwestHttpClient`]
//! which wraps [`reqwest::Client`], but custom implementations can be provided as needed.
//!
//! ## Types
//! - [`SecClient`]: Trait defining the interface for SEC-compliant HTTP clients.
//! - [`DefaultSecClient`]: Generic implementation of [`SecClient`] that works with any [`HttpClient`].
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

pub use implementations::{DefaultSecClient, ReqwestHttpClient};
pub use sec_client_error::{SecClientError, SecClientErrorReason};
pub use traits::{HttpClient, SecClient as SecClientTrait};

pub type SecClient = DefaultSecClient<ReqwestHttpClient>;

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
