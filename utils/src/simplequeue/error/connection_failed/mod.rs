use lapin::Error as LapinError;
use thiserror::Error;

/// Error representing a failed connection to a message broker.
///
/// This error type is used to encapsulate the details of a connection failure,
/// including the URI of the broker and the reason for the failure.
#[derive(Error, Debug, Clone)]
#[error(
    "[ConnectionFailed] Failure to establish a connection to message broker with URI: '{uri}'. Reason: '{reason}'"
)]
pub struct ConnectionFailed {
    /// The message broker URI at which failed to connect.
    pub uri: String,

    /// The reason describing the connection failure.
    pub reason: String,
}

impl ConnectionFailed {
    /// Creates a new `ConnectionFailed` error.
    ///
    /// # Arguments
    /// * `uri` - The URI of the message broker.
    /// * `reason` - The reason for the connection failure.
    ///
    /// # Returns
    /// A new `ConnectionFailed` error instance.
    #[must_use]
    pub fn new(uri: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            reason: reason.into(),
        }
    }

    /// Gets the message broker URI that failed to connect.
    ///
    /// # Returns
    /// A string slice containing the URI of the message broker.
    #[must_use]
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Gets the reason describing the connection failure.
    ///
    /// # Returns
    /// A string slice containing the failure reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl From<(&str, LapinError)> for ConnectionFailed {
    fn from((uri, source_error): (&str, LapinError)) -> Self {
        Self::new(uri, source_error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use lapin::Error as LapinError;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_create_connection_failed_when_new_is_called() {
        let uri = "amqp://localhost:5672";
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
        let source_error = LapinError::from(io_error);

        let expected_result = ConnectionFailed {
            uri: uri.to_string(),
            reason: source_error.to_string(),
        };

        let result = ConnectionFailed::new(uri, source_error.to_string());

        assert_eq!(result.uri, expected_result.uri);
        assert_eq!(result.reason, expected_result.reason);
    }

    #[test]
    fn should_format_display_as_expected_when_connection_fails() {
        let uri = "amqp://localhost:5672";
        let reason = "Connection refused by broker";

        let expected_result = format!(
            "[ConnectionFailed] Failure to establish a connection to message broker with URI: '{uri}'. Reason: '{reason}'"
        );

        let connection_failed = ConnectionFailed::new(uri, reason);
        let result = format!("{connection_failed}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_connection_failed_with_empty_strings_when_provided() {
        let uri = "";
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "empty test");
        let source_error = LapinError::from(io_error);

        let expected_result = ConnectionFailed {
            uri: uri.to_string(),
            reason: source_error.to_string(),
        };

        let result = ConnectionFailed::new(uri, source_error.to_string());

        assert_eq!(result.uri, expected_result.uri);
        assert_eq!(result.reason, expected_result.reason);
    }

    #[test]
    fn should_create_connection_failed_from_lapin_error_tuple() {
        let uri = "amqp://localhost:5672";
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
        let source_error = LapinError::from(io_error);

        let expected_uri = uri;
        let expected_reason = source_error.to_string();

        let result = ConnectionFailed::from((uri, source_error));

        assert_eq!(result.uri, expected_uri);
        assert_eq!(result.reason, expected_reason);
    }
}
