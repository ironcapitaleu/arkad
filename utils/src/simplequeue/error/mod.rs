pub mod connection_failed;

pub use connection_failed::ConnectionFailed;

#[non_exhaustive]
#[derive(Debug, Clone)]
/// The error enum for all queue errors.
///
/// This enum encapsulates all possible errors that can occur
/// during queue operations, including connection issues, configuration errors,
/// and operation failures.
pub enum ErrorKind {
    /// Invalid connector configuration
    InvalidConfiguration { reason: String },

    /// Failed to establish connection to the message broker.
    ConnectionFailed(ConnectionFailed),
}
