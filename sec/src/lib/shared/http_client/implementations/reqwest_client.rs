//! # Reqwest Inner Client
//!
//! Implements [`InnerClient`] for [`reqwest::Client`], binding the request, response, and error
//! types to their `reqwest` equivalents.
//!
//! Also provides the [`ReqwestClient`], a [`reqwest::Client`] together with the [`UserAgent`] it
//! sends.

use async_trait::async_trait;
use reqwest::{Client, Error as ReqwestError, Request, Response};

use super::super::traits::InnerClient;
use crate::shared::user_agent::UserAgent;

#[async_trait]
impl InnerClient for Client {
    /// The [`reqwest::Request`] type.
    type Request = Request;
    /// The [`reqwest::Response`] type.
    type Response = Response;
    /// The [`reqwest::Error`] type.
    type Error = ReqwestError;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        self.execute(request).await
    }
}

/// A [`reqwest::Client`] that sends a validated [`UserAgent`] with every request.
///
/// A [`reqwest::Client`] cannot return the `User-Agent` it sends. This type keeps the
/// [`UserAgent`] next to the client, so [`ReqwestClient::user_agent`] returns the value the
/// requests carry.
#[derive(Debug, Clone)]
pub struct ReqwestClient {
    client: Client,
    user_agent: UserAgent,
}

impl ReqwestClient {
    /// Builds a [`reqwest::Client`] that sends `user_agent` as its `User-Agent` header.
    ///
    /// # Errors
    ///
    /// Returns [`reqwest::Error`] if `reqwest` cannot build the client, for example when the TLS
    /// backend fails to initialize.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::http_client::implementations::reqwest_client::ReqwestClient;
    /// use sec::shared::user_agent::UserAgent;
    ///
    /// let user_agent = UserAgent::new("Sample Company contact@example.com")
    ///     .expect("A hardcoded SEC-format string should always validate");
    /// let client = ReqwestClient::new(user_agent.clone())
    ///     .expect("Given a validated user agent, the client creation should always succeed");
    ///
    /// let expected_result = &user_agent;
    ///
    /// let result = client.user_agent();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn new(user_agent: UserAgent) -> Result<Self, ReqwestError> {
        let client = Client::builder().user_agent(user_agent.inner()).build()?;
        Ok(Self { client, user_agent })
    }

    /// Returns the [`UserAgent`] that every request carries.
    #[must_use]
    pub const fn user_agent(&self) -> &UserAgent {
        &self.user_agent
    }
}

#[async_trait]
impl InnerClient for ReqwestClient {
    /// The [`reqwest::Request`] type.
    type Request = Request;
    /// The [`reqwest::Response`] type.
    type Response = Response;
    /// The [`reqwest::Error`] type.
    type Error = ReqwestError;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        self.client.execute(request).await
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_the_user_agent_it_was_built_from_when_created_with_new() {
        let user_agent = UserAgent::new("Sample Company contact@example.com")
            .expect("A hardcoded SEC-format string should always validate");

        let expected_result = user_agent.clone();

        let result = ReqwestClient::new(user_agent)
            .expect("Given a validated user agent, the client creation should always succeed")
            .user_agent()
            .clone();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_reqwest_client() {
        implements_auto_traits::<ReqwestClient>();
    }

    const fn implements_inner_client<T: InnerClient>() {}
    #[test]
    const fn should_implement_inner_client_for_reqwest_client() {
        implements_inner_client::<ReqwestClient>();
    }
}
