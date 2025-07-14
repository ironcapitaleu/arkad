use thiserror::Error;

use super::State as StateError;
use crate::shared::sec_client::SecClientError;
use crate::traits::error::FromDomainError;

#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[ClientCreationFailed] Failure in State: `{state_name}`. Error: {sec_client_error}")]
pub struct ClientCreationFailed {
    /// The name of the state where the error occurred.
    pub state_name: String,
    pub sec_client_error: SecClientError,
}

impl ClientCreationFailed {
    /// Creates a new state-level [`ClientCreationFailed`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    ///
    /// # Returns
    /// A new [`ClientCreationFailed`] error instance.
    #[must_use]
    pub fn new(state_name: &(impl ToString + ?Sized), sec_client_error: SecClientError) -> Self {
        Self {
            state_name: state_name.to_string(),
            sec_client_error,
        }
    }
}

/// Converts a state-level `ClientCreationFailed` error into the state error enum variant.
impl From<ClientCreationFailed> for StateError {
    /// Converts an [`ClientCreationFailed`] into a [`StateError::ClientCreationFailed`] variant.
    ///
    /// # Arguments
    /// * `val` - The [`ClientCreationFailed`] error to convert.
    ///
    /// # Returns
    /// A [`StateError`] containing the provided [`ClientCreationFailed`] error.
    fn from(domain_error: ClientCreationFailed) -> Self {
        Self::ClientCreationFailed(domain_error)
    }
}

/// Implements conversion from a domain-level [`CikError`] to a state-level [`InvalidCikFormat`] error.
///
/// This allows enriching a [`CikError`] with state context for use in state machine error handling.
impl FromDomainError<SecClientError> for ClientCreationFailed {
    type DomainErr = SecClientError;

    /// Converts a domain-level [`SecClientError`] into a state-level [`ClientCreationFailed`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `err` - The domain-level [`SecClientError`] to wrap.
    ///
    /// # Returns
    /// An [`ClientCreatioNFailed`] error containing the provided context.
    fn from_domain_error(state_name: &(impl ToString + ?Sized), err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}
