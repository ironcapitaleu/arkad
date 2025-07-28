use thiserror::Error;

pub mod connection_failed;

pub use connection_failed::ConnectionFailed;

/// The error enum for all queue errors.
///
/// This enum encapsulates all possible errors that can occur
/// during queue operations, including connection issues, configuration errors,
/// and operation failures.
///
/// The enum is marked as `#[non_exhaustive]` to allow for future error types
/// to be added without breaking existing code.
///
/// # Examples
///
/// ```
/// use utils::simplequeue::error::{ErrorKind, ConnectionFailed};
///
/// let connection_error = ConnectionFailed::new("amqp://localhost:5672", "Connection refused");
/// let error = ErrorKind::ConnectionFailed(connection_error);
///
/// match error {
///     ErrorKind::ConnectionFailed(ref err) => {
///         eprintln!("Connection failed to {}: {}", err.uri(), err.reason());
///     },
///     _ => {
///         eprintln!("An unknown error occurred");
///     }
/// }
/// ```
#[derive(Error, Debug, Clone)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Failed to establish connection to the message broker.
    ///
    /// This error occurs when the queue system cannot connect to the underlying
    /// message broker due to network issues, authentication failures, or
    /// broker unavailability.
    #[error(transparent)]
    ConnectionFailed(#[from] ConnectionFailed),
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_create_connection_failed_error_when_from_connection_failed_is_used() {
        let expected_uri = "amqp://localhost:5672";
        let expected_reason = "Connection refused";

        let connection_error = ConnectionFailed::new(expected_uri, expected_reason);
        let result = ErrorKind::from(connection_error);

        match result {
            ErrorKind::ConnectionFailed(err) => {
                assert_eq!(err.uri(), expected_uri);
                assert_eq!(err.reason(), expected_reason);
            }
        }
    }

    #[test]
    fn should_display_connection_failed_error_when_formatted() {
        let connection_error = ConnectionFailed::new("amqp://localhost:5672", "Connection refused");
        let high_level_error = ErrorKind::ConnectionFailed(connection_error);

        let expected_result = "[ConnectionFailed] Failure to establish a connection to message broker with URI: 'amqp://localhost:5672'. Reason: 'Connection refused'";

        let result = format!("{high_level_error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_cloneable_when_clone_is_called() {
        let connection_error = ConnectionFailed::new("amqp://localhost:5672", "Connection refused");
        let high_level_error = ErrorKind::ConnectionFailed(connection_error);

        let result = high_level_error.clone();

        assert_eq!(format!("{high_level_error}"), format!("{result}"));
    }
}
