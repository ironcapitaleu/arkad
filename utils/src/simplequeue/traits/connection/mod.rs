use std::fmt::Debug;

use async_trait::async_trait;

use crate::simplequeue::connector::Connector;
use crate::simplequeue::traits::{InnerChannel, Item};

use crate::simplequeue::channel::QueueIdentifier;
use crate::simplequeue::channel::{ConsumerChannel, ProducerChannel};

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
