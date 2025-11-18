//! # Invalid SEC Response State Error
//!
//! This module defines the [`InvalidSecResponse`] error type, which represents SEC response validation errors
//! at the state level within the SEC state machine framework. It wraps domain-level [`ValidatedSecResponseError`]s with additional
//! state context, enabling precise error reporting and handling in state machine workflows.
//!
//! ## Purpose
//! - Enriches domain SEC response validation errors with state information for robust error propagation.
//! - Supports conversion from domain errors and integration into the [`State`](super::State) error enum.
//!
//! ## Types
//! - [`InvalidSecResponse`]: Struct representing a SEC response validation error with state context.
//!
//! ## Usage
//! Use [`InvalidSecResponse`] to wrap [`ValidatedSecResponseError`]s when a SEC response validation failure occurs within a state. This allows
//! downstream error handlers to access both the state context and the underlying domain error.
//!
//! ## Example
//! ```rust
//! use sec::error::state_machine::state::invalid_sec_response::InvalidSecResponse;
//! use sec::shared::validated_sec_response::{ValidatedSecResponseError, ValidatedSecResponseErrorReason};
//!
//! let validation_error = ValidatedSecResponseError::new(ValidatedSecResponseErrorReason::EmptyResponseBody);
//! let state_error = InvalidSecResponse::new("ValidateSecResponse", validation_error);
//! ```
use thiserror::Error;

use super::State as StateError;
use crate::shared::validated_sec_response::ValidatedSecResponseError;
use crate::traits::error::FromDomainError;

/// Error representing an invalid SEC response at the state level.
///
/// This error type is used to wrap domain-level [`ValidatedSecResponseError`]s with additional information about
/// the state in which the error occurred, making it suitable for use in state machine error handling.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[InvalidSecResponse] Failure in State: `{state_name}`. Invalid SEC Response: {validation_error}"
)]
pub struct InvalidSecResponse {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level SEC response validation error.
    #[source]
    pub validation_error: ValidatedSecResponseError,
}

impl InvalidSecResponse {
    /// Creates a new state-level [`InvalidSecResponse`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `validation_error` - The underlying domain-level validation error.
    ///
    /// # Returns
    /// A new [`InvalidSecResponse`] error instance.
    #[must_use]
    pub fn new(state_name: impl Into<String>, validation_error: ValidatedSecResponseError) -> Self {
        Self {
            state_name: state_name.into(),
            validation_error,
        }
    }
}

/// Converts a state-level `InvalidSecResponse` error into the state error enum variant.
impl From<InvalidSecResponse> for StateError {
    /// Converts an [`InvalidSecResponse`] into a [`StateError::InvalidSecResponse`] variant.
    ///
    /// # Arguments
    /// * `domain_error` - The [`InvalidSecResponse`] error to convert.
    ///
    /// # Returns
    /// A [`StateError`] containing the provided [`InvalidSecResponse`] error.
    fn from(domain_error: InvalidSecResponse) -> Self {
        Self::InvalidSecResponse(domain_error)
    }
}

/// Implements conversion from a domain-level [`ValidatedSecResponseError`] to a state-level [`InvalidSecResponse`] error.
///
/// This allows enriching a [`ValidatedSecResponseError`] with state context for use in state machine error handling.
impl FromDomainError<ValidatedSecResponseError> for InvalidSecResponse {
    type DomainErr = ValidatedSecResponseError;

    /// Converts a domain-level [`ValidatedSecResponseError`] into a state-level [`InvalidSecResponse`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `err` - The domain-level [`ValidatedSecResponseError`] to wrap.
    ///
    /// # Returns
    /// An [`InvalidSecResponse`] error containing the provided context.
    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name.into(), err)
    }
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use crate::shared::validated_sec_response::ValidatedSecResponseErrorReason;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_invalid_sec_response_when_new_is_called() {
        let state_name = "TestState";
        let reason = ValidatedSecResponseErrorReason::EmptyResponseBody;
        let validation_error = ValidatedSecResponseError::new(reason);

        let expected_result = InvalidSecResponse {
            state_name: state_name.to_string(),
            validation_error: validation_error.clone(),
        };

        let result = InvalidSecResponse::new(state_name, validation_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_from_domain_error_when_from_domain_error_is_called() {
        let validation_error =
            ValidatedSecResponseError::new(ValidatedSecResponseErrorReason::EmptyResponseBody);
        let state_name = "ValidateSecResponse";

        let expected_result = InvalidSecResponse {
            state_name: state_name.to_string(),
            validation_error: validation_error.clone(),
        };

        let result = InvalidSecResponse::from_domain_error(state_name, validation_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_into_is_called() {
        let validation_error = ValidatedSecResponseError::new(
            ValidatedSecResponseErrorReason::InvalidStatusCode(StatusCode::BAD_REQUEST),
        );
        let invalid_sec_response = InvalidSecResponse {
            state_name: "TestState".to_string(),
            validation_error,
        };

        let expected_result = StateError::InvalidSecResponse(invalid_sec_response.clone());

        let result: StateError = invalid_sec_response.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_validation_error_as_source_of_invalid_sec_response() {
        let state_name = "ValidateSecResponse";
        let reason = ValidatedSecResponseErrorReason::EmptyResponseBody;
        let validation_error = ValidatedSecResponseError::new(reason);
        let invalid_sec_response = InvalidSecResponse::new(state_name, validation_error.clone());

        let source = std::error::Error::source(&invalid_sec_response);

        assert!(source.is_some(), "Expected source error to be present");
        let source = source.unwrap();

        let validation_error_from_source = source.downcast_ref::<ValidatedSecResponseError>();
        assert!(
            validation_error_from_source.is_some(),
            "Source should be ValidatedSecResponseError type"
        );
        assert_eq!(validation_error_from_source.unwrap(), &validation_error);
    }

    #[test]
    fn should_print_error_and_source_for_logging_demo() {
        let state_name = "ValidateSecResponse";
        let reason = ValidatedSecResponseErrorReason::InvalidStatusCode(StatusCode::NOT_FOUND);
        let validation_error = ValidatedSecResponseError::new(reason);
        let invalid_sec_response = InvalidSecResponse::new(state_name, validation_error);

        let error_string = format!("{invalid_sec_response}");
        let source_string = match std::error::Error::source(&invalid_sec_response) {
            Some(source) => format!("{source}"),
            None => "No source error".to_string(),
        };

        println!("Top-level error: {error_string}");
        println!("Caused by: {source_string}");

        assert!(error_string.contains("Failure in State"));
        assert!(source_string.contains("Response validation failed"));
    }
}
