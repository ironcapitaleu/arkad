use thiserror::Error;

/// Error types for Queue operations.
#[derive(Error, Debug)]
pub enum ErrorKind {
    /// Configuration error when loading from environment.
    #[error("Queue configuration error.")]
    Config,

    /// Connection establishment error.
    #[error("Queue connection error: {0}")]
    Connection(#[source] lapin::Error),

    /// Channel creation error.
    #[error("Queue channel creation error: {0}")]
    Channel(#[source] lapin::Error),

    /// Queue operation error.
    #[error("Queue operation error: {0}")]
    Operation(#[source] lapin::Error),
}