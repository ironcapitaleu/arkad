use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::sec_client::SecClient;
use crate::shared::sec_request::SecRequest;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Input data for the `ExecuteSecRequest` fixture.
#[derive(Default)]
pub struct ExecuteSecRequestInputData {
    pub sec_client: SecClient,
    pub sec_request: SecRequest,
}

impl ExecuteSecRequestInputData {
    /// Creates a new instance of the input data for the execute SEC request.
    pub const fn new(sec_client: SecClient, sec_request: SecRequest) -> Self {
        Self {
            sec_client,
            sec_request,
        }
    }

    /// Returns a reference to the SEC client.
    #[must_use]
    pub const fn sec_client(&self) -> &SecClient {
        &self.sec_client
    }

    /// Returns a reference to the SEC request.
    #[must_use]
    pub const fn sec_request(&self) -> &SecRequest {
        &self.sec_request
    }
}

impl StateData for ExecuteSecRequestInputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(sec_client) = updates.sec_client {
            self.sec_client = sec_client;
        }
        if let Some(sec_request) = updates.sec_request {
            self.sec_request = sec_request;
        }
        Ok(())
    }
}

impl SMStateData for ExecuteSecRequestInputData {
    type UpdateType = ExecuteSecRequestInputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, _updates: Self::UpdateType) {}
}


impl fmt::Display for ExecuteSecRequestInputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SEC Client ID: {}\nSEC Request URL: {}",
            self.sec_client.id(),
            self.sec_request.inner.url()
        )
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for [`ExecuteSecRequestInputData`].
pub struct ExecuteSecRequestInputDataUpdater {
    pub sec_client: Option<SecClient>,
    pub sec_request: Option<SecRequest>,
}

/// Builder for [`ExecuteSecRequestInputDataUpdater`].
pub struct ExecuteSecRequestInputDataUpdaterBuilder {
    sec_client: Option<SecClient>,
    sec_request: Option<SecRequest>,
}
impl ExecuteSecRequestInputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sec_client: None,
            sec_request: None,
        }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn sec_client(mut self, sec_client: SecClient) -> Self {
        self.sec_client = Some(sec_client);
        self
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn sec_request(mut self, sec_request: SecRequest) -> Self {
        self.sec_request = Some(sec_request);
        self
    }

    #[must_use]
    pub fn build(self) -> ExecuteSecRequestInputDataUpdater {
        ExecuteSecRequestInputDataUpdater {
            sec_client: self.sec_client,
            sec_request: self.sec_request,
        }
    }
}

impl Default for ExecuteSecRequestInputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
