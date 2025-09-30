use reqwest::Error as ReqwestError;
use thiserror::Error;

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
    pub fn new(reason: SecRequestErrorReason) -> Self {
        Self { reason }
    }
}

/// Enum representing the reason for a request failure.
///
/// This enum is marked as non-exhaustive to allow for future extension.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecRequestErrorReason {
    NetworkError,
    HttpError(String),
    TimeoutError,
    Other(String),
}

impl Into<SecRequestError> for ReqwestError {
    fn into(self) -> SecRequestError {
        if self.is_timeout() {
            SecRequestError::new(SecRequestErrorReason::TimeoutError)
        } else if self.is_connect() {
            SecRequestError::new(SecRequestErrorReason::NetworkError)
        } else if self.is_status() {
            if let Some(status) = self.status() {
                SecRequestError::new(SecRequestErrorReason::HttpError(status.to_string()))
            } else {
                SecRequestError::new(SecRequestErrorReason::Other(self.to_string()))
            }
        } else {
            SecRequestError::new(SecRequestErrorReason::Other(self.to_string()))
        }
    }
}

impl std::fmt::Display for SecRequestErrorReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError => write!(f, "The HTTP request failed due to a network error."),
            Self::HttpError(status_code) => write!(
                f,
                "The HTTP request failed due to an HTTP error: {status_code}."
            ),
            Self::TimeoutError => write!(f, "The HTTP request timed out."),
            Self::Other(message) => write!(
                f,
                "The HTTP request failed for an unknown reason: {message}."
            ),
        }
    }
}
