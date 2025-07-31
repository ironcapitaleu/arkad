use std::fmt;

use async_trait::async_trait;

use crate::simplequeue::error::connection_failed::ConnectionFailed;

/// Trait for inner connection types that can establish connections.
#[async_trait]
pub trait InnerConnection: Send + Sync + fmt::Debug + Sized {
    /// Establishes a connection using the provided URI.
    ///
    /// # Arguments
    /// * `uri` - The connection URI string
    ///
    /// # Returns
    /// `Ok(Self)` if the connection is successful.
    /// `Err(ConnectionFailed)` if the connection fails.
    async fn connect(&self, uri: &str) -> Result<Self, ConnectionFailed>;
}
