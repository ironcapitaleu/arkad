use std::fmt::Debug;

use async_trait::async_trait;

use crate::simplequeue::connector::Connector;
use crate::simplequeue::traits::Item;

use crate::simplequeue::channel::QueueIdentifier;
use crate::simplequeue::channel::{ConsumerChannel, ProducerChannel};

pub mod inner;

pub use inner::InnerConnection;

#[async_trait]
pub trait Connection: Send + Sync + 'static + Debug + Clone {
    type Inner: InnerConnection;

    fn new<I: InnerConnection>(inner: I, connector: Connector) -> Self;

    fn inner(&self) -> &Self::Inner;
    fn connector(&self) -> &Connector;

    async fn create_producer_channel<T: Item>(
        &self,
        queue_identifier: QueueIdentifier,
    ) -> Result<ProducerChannel<Self::Inner, T>, String>;

    async fn create_consumer_channel<T: Item>(
        &self,
        queue_identifier: QueueIdentifier,
    ) -> Result<ConsumerChannel<Self::Inner, T>, String>;
}
