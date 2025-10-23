use reqwest::Error as ReqwestError;
use thiserror::Error;

use crate::shared::sec_response::{SecResponseError, SecResponseErrorReason};

/// Error details for SEC request failures.
///
/// This struct provides both the reason for the failure and the user agent string that was provided.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[SecRequestError] Request failed: Reason: '{reason}'.")]
pub struct SecRequestError {
    /// The reason why the request couldn't be processed.
    pub reason: SecRequestErrorReason,
}

impl SecRequestError {
    /// Creates a new `SecRequestError`.
    #[must_use]
    pub const fn new(reason: SecRequestErrorReason) -> Self {
        Self { reason }
    }
}

/// Enum representing the reason for a request failure.
///
/// This enum is marked as non-exhaustive to allow for future extension.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecRequestErrorReason {
    NetworkError(String),
    HttpError(String),
    Timeout(String),
    Other(String),
}

impl From<ReqwestError> for SecRequestError {
    fn from(e: ReqwestError) -> Self {
        if e.is_timeout() {
            Self::new(SecRequestErrorReason::Timeout(e.to_string()))
        } else if e.is_connect() {
            Self::new(SecRequestErrorReason::NetworkError(e.to_string()))
        } else if e.is_status() {
            e.status().map_or_else(
                || Self::new(SecRequestErrorReason::Other(e.to_string())),
                |status| Self::new(SecRequestErrorReason::HttpError(status.to_string())),
            )
        } else {
            Self::new(SecRequestErrorReason::Other(e.to_string()))
        }
    }
}

impl From<SecResponseError> for SecRequestError {
    fn from(e: SecResponseError) -> Self {
        match e.reason {
            SecResponseErrorReason::InvalidUtf8InHeader(header) => {
                Self::new(SecRequestErrorReason::Other(format!(
                    "Response processing failed: Header '{header}' contains invalid UTF-8 data"
                )))
            }
            SecResponseErrorReason::NetworkError(message) => {
                Self::new(SecRequestErrorReason::NetworkError(message))
            }
            SecResponseErrorReason::Other(message) => {
                Self::new(SecRequestErrorReason::Other(message))
            }
        }
    }
}

impl std::fmt::Display for SecRequestErrorReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(message) => write!(
                f,
                "The HTTP request failed due to a network error: {message}."
            ),
            Self::HttpError(status_code) => write!(
                f,
                "The HTTP request failed due to an HTTP error: {status_code}."
            ),
            Self::Timeout(message) => write!(f, "The HTTP request timed out: {message}."),
            Self::Other(message) => write!(
                f,
                "The HTTP request failed for an unknown reason: {message}."
            ),
        }
    }
}
