pub mod sec_client_error;
pub use sec_client_error::{SecClientError, SecClientErrorReason};

use super::user_agent::UserAgent;

use reqwest::{Client, ClientBuilder};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SecClient {
    pub id: String,
    pub inner: reqwest::Client,
}

impl SecClient {
    /// Creates a new `SecClient` with a unique ID and a user agent.
    ///
    /// # Errors
    /// Returns an `SecClientError` if the user agent string is invalid or the client cannot be created.
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

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub const fn client(&self) -> &Client {
        &self.inner
    }
}

impl PartialEq for SecClient {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for SecClient {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for SecClient {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl Eq for SecClient {}

impl std::hash::Hash for SecClient {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Default for SecClient {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            inner: Client::new(),
        }
    }
}
