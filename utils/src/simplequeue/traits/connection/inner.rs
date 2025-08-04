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

    async fn create_producer_channel<T: Item>(
        &self,
        queue_identifier: QueueIdentifier,
    ) -> Result<ProducerChannel<Self, T>, String>;

    async fn create_consumer_channel<T: Item>(
        &self,
        queue_identifier: QueueIdentifier,
    ) -> Result<ConsumerChannel<Self, T>, String>;
}
