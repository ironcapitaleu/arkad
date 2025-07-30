use std::fmt;

use lapin::Connection as LapinConnection;

use crate::simplequeue::error::connection_failed::ConnectionFailed;

/// Trait for inner connection types that can establish connections.
pub trait InnerConnection: Send + Sync + fmt::Debug {
    /// Establishes a connection using the provided URI.
    ///
    /// # Arguments
    /// * `uri` - The connection URI string
    ///
    /// # Returns
    /// `Ok(Self)` if the connection is successful.
    /// `Err(ConnectionFailed)` if the connection fails.
    async fn connect(&self, uri: &str) -> Result<Self, ConnectionFailed>
    where
        Self: Sized;
}

impl InnerConnection for LapinConnection {
    async fn connect(&self, uri: &str) -> Result<Self, ConnectionFailed> {
        let lapin_connection_result =
            Self::connect(uri, lapin::ConnectionProperties::default()).await;

        lapin_connection_result.map_err(|lapin_error| ConnectionFailed::from((uri, lapin_error)))
    }
}
