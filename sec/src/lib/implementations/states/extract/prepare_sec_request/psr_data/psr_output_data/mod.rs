use std::{fmt, hash::Hash};

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

use reqwest::{Client, Method, Request, Url};

#[derive(Debug)]
pub struct PrepareSecRequestOutputData {
    pub client: Client,
    pub request: Request,
}

impl Clone for PrepareSecRequestOutputData {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            request: self
                .request
                .try_clone()
                .expect("Failed to clone RequestBuilder"),
        }
    }
}

impl PartialEq for PrepareSecRequestOutputData {
    fn eq(&self, other: &Self) -> bool {
        self.request.url() == other.request.url()
    }
}

impl PartialOrd for PrepareSecRequestOutputData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.request.url().cmp(other.request.url()))
    }
}

impl Ord for PrepareSecRequestOutputData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.request.url().cmp(other.request.url())
    }
}

impl Eq for PrepareSecRequestOutputData {}

impl Hash for PrepareSecRequestOutputData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.request.url().hash(state);
    }
}

impl PrepareSecRequestOutputData {
    /// Creates a new instance of `PrepareSecRequestOutputData` with the provided client and request.
    ///
    /// # Errors
    /// Returns a `StateError` if the output data cannot be created from the provided data.
    pub const fn new(client: Client, request: Request) -> Result<Self, StateError> {
        Ok(Self { client, request })
    }

    /// Returns a reference to the client.
    #[must_use]
    pub const fn client(&self) -> &Client {
        &self.client
    }

    /// Returns a reference to the request.
    #[must_use]
    pub const fn request(&self) -> &Request {
        &self.request
    }
}

impl StateData for PrepareSecRequestOutputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(client) = updates.client {
            self.client = client;
        }
        if let Some(request) = updates.request {
            self.request = request;
        }
        Ok(())
    }
}
impl SMStateData for PrepareSecRequestOutputData {
    type UpdateType = PrepareSecRequestOutputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl Default for PrepareSecRequestOutputData {
    fn default() -> Self {
        Self {
            client: Client::new(),
            request: Request::new(
                Method::GET,
                Url::parse("https://httpbin.org/get")
                    .expect("Hardcoded URL should always be valid."),
            ),
        }
    }
}

impl fmt::Display for PrepareSecRequestOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tURL: {}", self.request.url())
    }
}

#[derive(Debug)]
pub struct PrepareSecRequestOutputDataUpdater {
    pub client: Option<Client>,
    pub request: Option<Request>,
}

pub struct PrepareSecRequestOutputDataUpdaterBuilder {
    client: Option<Client>,
    request: Option<Request>,
}

impl PrepareSecRequestOutputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            client: None,
            request: None,
        }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn request(mut self, request: Request) -> Self {
        self.request = Some(request);
        self
    }

    #[must_use]
    pub fn build(self) -> PrepareSecRequestOutputDataUpdater {
        PrepareSecRequestOutputDataUpdater {
            client: self.client,
            request: self.request,
        }
    }
}

impl Default for PrepareSecRequestOutputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
