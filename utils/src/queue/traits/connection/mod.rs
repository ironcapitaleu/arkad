use std::fmt::Debug;

use async_trait::async_trait;

use crate::queue::connector::Connector;
use crate::queue::traits::{InnerChannel, Item};

use crate::queue::implementations::channel::{ConsumerChannel, ProducerChannel, QueueIdentifier};

pub mod inner;

pub use inner::InnerConnection;

#[async_trait]
pub trait Connection: Send + Sync + 'static + Debug {
    type Inner: InnerConnection;

    fn new(inner: Self::Inner, connector: Connector) -> Self;

    fn inner(&self) -> &Self::Inner;
    fn connector(&self) -> &Connector;

    fn create_producer_channel<T: Item, IC: InnerChannel>(
        &self,
        queue_identifier: QueueIdentifier,
        inner_channel: IC,
    ) -> ProducerChannel<IC, T> {
        self.inner()
            .create_producer_channel(queue_identifier, inner_channel)
    }

    fn create_consumer_channel<T: Item, IC: InnerChannel>(
        &self,
        queue_identifier: QueueIdentifier,
        inner_channel: IC,
    ) -> ConsumerChannel<IC, T> {
        self.inner()
            .create_consumer_channel(queue_identifier, inner_channel)
    }
}
