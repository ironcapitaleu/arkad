use std::{fmt, hash::Hash};

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::sec_client::SecClient;
use crate::shared::sec_request::SecRequest;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestOutputData {
    pub client: SecClient,
    pub request: SecRequest,
}

impl PrepareSecRequestOutputData {
    /// Creates a new instance of `PrepareSecRequestOutputData` with the provided client and request.
    ///
    /// # Errors
    /// Returns a `StateError` if the output data cannot be created from the provided data.
    pub const fn new(client: SecClient, request: SecRequest) -> Result<Self, StateError> {
        Ok(Self { client, request })
    }

    /// Returns a reference to the client.
    #[must_use]
    pub const fn client(&self) -> &SecClient {
        &self.client
    }

    /// Returns a reference to the request.
    #[must_use]
    pub const fn request(&self) -> &SecRequest {
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

impl fmt::Display for PrepareSecRequestOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tURL: {}", self.request.inner.url())
    }
}

#[derive(Debug)]
pub struct PrepareSecRequestOutputDataUpdater {
    pub client: Option<SecClient>,
    pub request: Option<SecRequest>,
}

pub struct PrepareSecRequestOutputDataUpdaterBuilder {
    client: Option<SecClient>,
    request: Option<SecRequest>,
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
    pub fn client(mut self, client: SecClient) -> Self {
        self.client = Some(client);
        self
    }
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn request(mut self, request: SecRequest) -> Self {
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
