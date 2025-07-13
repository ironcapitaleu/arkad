use reqwest::{Client, ClientBuilder};
use uuid::Uuid;

use crate::error::State as StateError;

#[derive(Debug, Clone)]
pub struct SecClient {
    pub id: String,
    pub inner: reqwest::Client,
}

impl SecClient {
    pub fn new(user_agent: &String) -> Result<Self, StateError> {
        let client = ClientBuilder::new()
            .user_agent(user_agent)
            .build();
        
        let Ok(client) = client else {
            return Err(StateError::ClientCreationFailed(
                "Failed to create SecClient".to_string(),
            ));
        };

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            inner: client,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn client(&self) -> &Client {
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