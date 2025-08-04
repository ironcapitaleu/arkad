use std::fmt;

use async_trait::async_trait;

use crate::simplequeue::channel::{
    ChannelBuilder, ConsumerChannel, ProducerChannel, QueueIdentifier,
};
use crate::simplequeue::error::connection_failed::ConnectionFailed;
use crate::simplequeue::traits::{InnerChannel, Item};

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

    fn create_producer_channel<T: Item, IC: InnerChannel>(
        &self,
        queue_identifier: QueueIdentifier,
        inner_channel: IC,
    ) -> ProducerChannel<IC, T> {
        ChannelBuilder::new()
            .producer()
            .item_type::<T>()
            .inner(inner_channel)
            .queue_identifier(queue_identifier)
            .build()
    }

    fn create_consumer_channel<T: Item, IC: InnerChannel>(
        &self,
        queue_identifier: QueueIdentifier,
        inner_channel: IC,
    ) -> ConsumerChannel<IC, T> {
        ChannelBuilder::new()
            .consumer()
            .item_type::<T>()
            .inner(inner_channel)
            .queue_identifier(queue_identifier)
            .build()
    }
}
