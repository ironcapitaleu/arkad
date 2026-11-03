//! # Reqwest Inner Client
//!
//! Implements [`InnerClient`] for [`reqwest::Client`], binding the request, response, and error
//! types to their `reqwest` equivalents.
//!
//! Also provides the [`ReqwestClient`], a [`reqwest::Client`] together with the [`UserAgent`] it
//! sends, and the timeouts that [`ReqwestClient::new`] sets.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, ClientBuilder, Error as ReqwestError, Request, Response};

use super::super::traits::InnerClient;
use crate::shared::user_agent::UserAgent;

/// The longest wait to open a connection, before the request fails.
///
/// A connection to the SEC opens in well under one second. 10 s leaves room for a slow network,
/// and a host that does not answer still fails long before the job timeout. The value is a
/// proposal and is not measured.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// The longest wait for the next bytes of a response, before the request fails.
///
/// The wait starts with the request and starts again after each read of the body. So it limits
/// the wait between two reads, not the whole download. A total timeout does not fit, because it
/// must allow for the slowest download. The largest Notes file is 0.85 GB, and it takes about
/// 80 s with 2 files in flight. 60 s lets the SEC pause under load, and a stalled connection
/// still fails long before the job timeout. The value is a proposal and is not measured.
// Deviation: `Duration::from_mins` is not stable in Rust 1.90, the CI version, so the code writes
// the value in seconds. Rust 1.90 does not know the lint, hence `unknown_lints`.
#[allow(unknown_lints, clippy::duration_suboptimal_units)]
pub const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// The timeouts that [`ReqwestClient::new`] sets.
const TIMEOUTS: Timeouts = Timeouts {
    connect: CONNECT_TIMEOUT,
    read: READ_TIMEOUT,
};

/// The connect timeout and the read timeout of a client.
///
/// Named fields keep the two values of the same type from being swapped.
struct Timeouts {
    /// The longest wait to open a connection.
    connect: Duration,
    /// The longest wait for the next bytes of a response.
    read: Duration,
}

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
    /// The client fails a request that cannot connect within [`CONNECT_TIMEOUT`], or that waits
    /// longer than [`READ_TIMEOUT`] for the next bytes of the response.
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
    ///     .expect("Given a validated user agent, building the HTTP client fails only if the TLS backend cannot initialize");
    ///
    /// let expected_result = &user_agent;
    ///
    /// let result = client.user_agent();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn new(user_agent: UserAgent) -> Result<Self, ReqwestError> {
        Self::from_builder(timed_builder(&TIMEOUTS), user_agent)
    }

    /// Builds the client from `builder` and sets `user_agent` as its `User-Agent` header.
    ///
    /// The builder carries any other client settings, such as timeouts or pool size. This method
    /// sets the header, so the header and [`ReqwestClient::user_agent`] stay the same value.
    ///
    /// # Errors
    ///
    /// Returns [`reqwest::Error`] if `reqwest` cannot build the client, for example when the TLS
    /// backend fails to initialize.
    pub fn from_builder(
        builder: ClientBuilder,
        user_agent: UserAgent,
    ) -> Result<Self, ReqwestError> {
        let client = builder.user_agent(user_agent.inner()).build()?;
        Ok(Self { client, user_agent })
    }

    /// Returns the [`UserAgent`] that every request carries.
    #[must_use]
    pub const fn user_agent(&self) -> &UserAgent {
        &self.user_agent
    }
}

/// Returns a [`ClientBuilder`] with both `timeouts` set.
fn timed_builder(timeouts: &Timeouts) -> ClientBuilder {
    Client::builder()
        .connect_timeout(timeouts.connect)
        .read_timeout(timeouts.read)
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
    use tokio::time;

    use super::*;
    use crate::tests::stalled_server::stalled_server;

    #[tokio::test]
    async fn should_return_a_timeout_error_when_the_server_sends_no_response() {
        let address = stalled_server(b"").await;
        let timeouts = Timeouts {
            connect: CONNECT_TIMEOUT,
            read: Duration::from_millis(500),
        };
        let guard = Duration::from_secs(5);
        let builder = timed_builder(&timeouts).no_proxy();
        let user_agent = UserAgent::new("Sample Company contact@example.com")
            .expect("A hardcoded SEC-format string should always validate");
        let client = ReqwestClient::from_builder(builder, user_agent)
            .expect("Given a validated user agent, building the HTTP client fails only if the TLS backend cannot initialize");
        let request = client
            .client
            .get(format!("http://{address}/"))
            .build()
            .expect("A URL built from a bound socket address always parses");

        let expected_result = true;

        let result = time::timeout(guard, client.execute_request(request))
            .await
            .is_ok_and(|response| response.is_err_and(|error| error.is_timeout()));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_user_agent_it_was_built_from_when_created_with_new() {
        let user_agent = UserAgent::new("Sample Company contact@example.com")
            .expect("A hardcoded SEC-format string should always validate");

        let expected_result = user_agent.clone();

        let result = ReqwestClient::new(user_agent)
            .expect("Given a validated user agent, building the HTTP client fails only if the TLS backend cannot initialize")
            .user_agent()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_user_agent_it_was_built_from_when_created_from_a_builder() {
        let builder = Client::builder();
        let user_agent = UserAgent::new("Sample Company contact@example.com")
            .expect("A hardcoded SEC-format string should always validate");

        let expected_result = user_agent.clone();

        let result = ReqwestClient::from_builder(builder, user_agent)
            .expect("Given a validated user agent, building the HTTP client fails only if the TLS backend cannot initialize")
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
