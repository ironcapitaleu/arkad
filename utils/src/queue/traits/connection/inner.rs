use std::fmt;

use async_trait::async_trait;

use crate::queue::error::connection_failed::ConnectionFailed;
use crate::queue::implementations::channel::{ChannelBuilder, ConsumerChannel, ProducerChannel};
use crate::queue::shared::queue_identifiers::QueueIdentifier;
use crate::queue::traits::{InnerChannel, Item};

/// Trait for inner connection types that can establish connections.
#[async_trait]
pub trait InnerConnection: Send + Sync + 'static + fmt::Debug + Sized {
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
