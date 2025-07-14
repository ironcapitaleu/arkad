//! # Client Creation Failed State Error
//!
//! This module defines the [`ClientCreationFailed`] error type, which represents client creation errors
//! at the state level within the SEC state machine framework. It wraps domain-level [`SecClientError`]s with additional
//! state context, enabling precise error reporting and handling in state machine workflows.
//!
//! ## Purpose
//! - Enriches domain client creation validation errors with state information for robust error propagation.
//! - Supports conversion from domain errors and integration into the [`State`](super::State) error enum.
//!
//! ## Types
//! - [`ClientCreationFailed`]: Struct representing a client creation error with state context.
//!
//! ## Usage
//! Use [`ClientCreationFailed`] to wrap [`SecClientError`]s when a client creation failure occurs within a state. This allows
//! downstream error handlers to access both the state context and the underlying domain error.
//!
//! ## Example
//! ```rust
//! use sec::error::state_machine::state::client_creation_failed::ClientCreationFailed;
//! use sec::shared::sec_client::{SecClientError, SecClientErrorReason};
//! let sec_client_error = SecClientError { reason: SecClientErrorReason::ReqwestClientCreationFailed, user_agent: "bad_agent".to_string() };
//! let state_error = ClientCreationFailed::new("PrepareSecRequest", sec_client_error);
//! ```
use thiserror::Error;

use super::State as StateError;
use crate::shared::sec_client::SecClientError;
use crate::traits::error::FromDomainError;

/// Error representing a client creation failure at the state level.
///
/// This error type is used to wrap domain-level [`SecClientError`]s with additional information about
/// the state in which the error occurred, making it suitable for use in state machine error handling.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[ClientCreationFailed] Failure in State: `{state_name}`. Error: {sec_client_error}")]
pub struct ClientCreationFailed {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level SEC client error.
    #[source]
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

/// Implements conversion from a domain-level [`SecClientError`] to a state-level [`ClientCreationFailed`] error.
///
/// This allows enriching a [`SecClientError`] with state context for use in state machine error handling.
impl FromDomainError<SecClientError> for ClientCreationFailed {
    type DomainErr = SecClientError;

    /// Converts a domain-level [`SecClientError`] into a state-level [`ClientCreationFailed`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `err` - The domain-level [`SecClientError`] to wrap.
    ///
    /// # Returns
    /// An [`ClientCreationFailed`] error containing the provided context.
    fn from_domain_error(state_name: &(impl ToString + ?Sized), err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}
