use std::fmt::Debug;

use crate::simplequeue::channel::{ChannelType, QueueIdentifier};
use crate::simplequeue::traits::{
    Channel, InnerChannel, ProducerChannel as ProducerChannelTrait, ProducerItem,
};

#[derive(Debug, Clone)]
pub struct ProducerChannel<I, T>
where
    I: InnerChannel,
    T: ProducerItem,
{
    inner: I,
    queue_identifier: QueueIdentifier,
    _marker: std::marker::PhantomData<T>,
}

impl<I, T> Channel for ProducerChannel<I, T>
where
    I: InnerChannel,
    T: ProducerItem,
{
    type Inner = I;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Producer
    }

    fn queue_identifier(&self) -> &QueueIdentifier {
        &self.queue_identifier
    }
}

impl<I, T> ProducerChannelTrait for ProducerChannel<I, T>
where
    I: InnerChannel,
    T: ProducerItem,
{
    type Item = T;

    fn produce(&self, _item: Self::Item) -> Result<(), String> {
        // TODO: Implement actual produce logic
        Err("Not implemented".to_string())
    }
}
impl<I, T> ProducerChannel<I, T>
where
    I: InnerChannel,
    T: ProducerItem,
{
    #[must_use]
    pub const fn new(inner: I, queue_identifier: QueueIdentifier) -> Self {
        Self {
            inner,
            queue_identifier,
            _marker: std::marker::PhantomData,
        }
    }
}
