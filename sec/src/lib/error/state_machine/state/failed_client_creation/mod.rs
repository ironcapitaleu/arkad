//! # Failed Client Creation State Error
//!
//! This module defines the [`FailedClientCreation`] error type, which represents client creation errors
//! at the state level within the SEC state machine framework. It wraps domain-level [`SecClientError`]s with additional
//! state context, enabling precise error reporting and handling in state machine workflows.
//!
//! ## Purpose
//! - Enriches domain client creation validation errors with state information for robust error propagation.
//! - Supports conversion from domain errors and integration into the [`State`](super::State) error enum.
//!
//! ## Types
//! - [`FailedClientCreation`]: Struct representing a client creation error with state context.
//!
//! ## Usage
//! Use [`FailedClientCreation`] to wrap [`SecClientError`]s when a client creation failure occurs within a state. This allows
//! downstream error handlers to access both the state context and the underlying domain error.
//!
//! ## Example
//! ```rust
//! use sec::error::state_machine::state::failed_client_creation::FailedClientCreation;
//! use sec::shared::sec_client::{SecClientError, SecClientErrorReason};
//! let sec_client_error = SecClientError { reason: SecClientErrorReason::ReqwestClientCreationFailed, user_agent: "bad_agent".to_string() };
//! let state_error = FailedClientCreation::new("PrepareSecRequest", sec_client_error);
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
#[error("[FailedClientCreation] Failure in State: '{state_name}'. Error: '{sec_client_error}'")]
pub struct FailedClientCreation {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level SEC client error.
    #[source]
    pub sec_client_error: SecClientError,
}

impl FailedClientCreation {
    /// Creates a new state-level [`FailedClientCreation`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the [`State`](crate::traits::state_machine::state::State) where the error occurred.
    ///
    /// # Returns
    /// A new [`FailedClientCreation`] error instance.
    #[must_use]
    pub fn new(state_name: impl Into<String>, sec_client_error: SecClientError) -> Self {
        Self {
            state_name: state_name.into(),
            sec_client_error,
        }
    }
}

/// Converts a state-level [`FailedClientCreation`] error into the [`StateError`] enum variant.
impl From<FailedClientCreation> for StateError {
    /// Converts an [`FailedClientCreation`] into a [`StateError::FailedClientCreation`] variant.
    ///
    /// # Arguments
    /// * `val` - The [`FailedClientCreation`] error to convert.
    ///
    /// # Returns
    /// A [`StateError`] containing the provided [`FailedClientCreation`] error.
    fn from(domain_error: FailedClientCreation) -> Self {
        Self::FailedClientCreation(domain_error)
    }
}

/// Implements conversion from a domain-level [`SecClientError`] to a state-level [`FailedClientCreation`] error.
///
/// This allows enriching a [`SecClientError`] with state context for use in state machine error handling.
impl FromDomainError<SecClientError> for FailedClientCreation {
    type DomainErr = SecClientError;

    /// Converts a domain-level [`SecClientError`] into a state-level [`FailedClientCreation`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `err` - The domain-level [`SecClientError`] to wrap.
    ///
    /// # Returns
    /// An [`FailedClientCreation`] error containing the provided context.
    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name.into(), err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::sec_client::SecClientErrorReason;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_failed_client_creation_when_new_is_called() {
        let state_name = "TestState";
        let reason = SecClientErrorReason::ReqwestClientCreationFailed;
        let user_agent = "test_agent";
        let sec_client_error = SecClientError {
            reason,
            user_agent: user_agent.to_string(),
        };

        let expected_result = FailedClientCreation {
            state_name: state_name.to_string(),
            sec_client_error: sec_client_error.clone(),
        };

        let result = FailedClientCreation::new(state_name, sec_client_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_from_domain_error_when_from_domain_error_is_called() {
        let sec_client_error = SecClientError {
            reason: SecClientErrorReason::ReqwestClientCreationFailed,
            user_agent: "test_agent".to_string(),
        };
        let state_name = "ClientCreationState";

        let expected_result = FailedClientCreation {
            state_name: state_name.to_string(),
            sec_client_error: sec_client_error.clone(),
        };

        let result = FailedClientCreation::from_domain_error(state_name, sec_client_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_into_is_called() {
        let sec_client_error = SecClientError {
            reason: SecClientErrorReason::ReqwestClientCreationFailed,
            user_agent: "test_agent".to_string(),
        };
        let failed_client_creation = FailedClientCreation {
            state_name: "TestState".to_string(),
            sec_client_error,
        };

        let expected_result = StateError::FailedClientCreation(failed_client_creation.clone());

        let result: StateError = failed_client_creation.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_sec_client_error_as_source_of_failed_client_creation() {
        let state_name = "SomeState";
        let reason = SecClientErrorReason::ReqwestClientCreationFailed;
        let user_agent = "test_agent";
        let sec_client_error = SecClientError {
            reason,
            user_agent: user_agent.to_string(),
        };
        let failed_client_creation =
            FailedClientCreation::new(state_name, sec_client_error.clone());

        let expected_result = Some(&sec_client_error);

        let result = std::error::Error::source(&failed_client_creation)
            .and_then(|source| source.downcast_ref::<SecClientError>());

        assert_eq!(result, expected_result);
    }
}
