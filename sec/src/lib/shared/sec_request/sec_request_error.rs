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
    TimeoutError(String),
    Other(String),
}

impl From<ReqwestError> for SecRequestError {
    fn from(e: ReqwestError) -> Self {
        match () {
            () if e.is_timeout() => Self::new(SecRequestErrorReason::TimeoutError(e.to_string())),
            () if e.is_connect() => Self::new(SecRequestErrorReason::NetworkError(e.to_string())),
            () if e.is_status() => e.status().map_or_else(
                || Self::new(SecRequestErrorReason::Other(e.to_string())),
                |status| Self::new(SecRequestErrorReason::HttpError(status.to_string())),
            ),
            () => Self::new(SecRequestErrorReason::Other(e.to_string())),
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
            Self::TimeoutError(message) => write!(f, "The HTTP request timed out: {message}."),
            Self::Other(message) => write!(
                f,
                "The HTTP request failed for an unknown reason: {message}."
            ),
        }
    }
}
