use std::fmt::Debug;

use crate::simplequeue::channel::{ChannelType, QueueIdentifier};
use crate::simplequeue::traits::{
    Channel, ConsumerChannel as ConsumerChannelTrait, ConsumerItem, InnerChannel,
};

#[derive(Debug, Clone)]
pub struct ConsumerChannel<I, T>
where
    I: InnerChannel,
    T: ConsumerItem,
{
    inner: I,
    queue_identifier: QueueIdentifier,
    _marker: std::marker::PhantomData<T>,
}

impl<I, T> Channel for ConsumerChannel<I, T>
where
    I: InnerChannel,
    T: ConsumerItem,
{
    type Inner = I;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Consumer
    }

    fn queue_identifier(&self) -> &QueueIdentifier {
        &self.queue_identifier
    }
}

impl<I, T> ConsumerChannelTrait for ConsumerChannel<I, T>
where
    I: InnerChannel,
    T: ConsumerItem,
{
    type Item = T;

    fn consume(&self) -> Result<Self::Item, String> {
        // TODO: Implement actual consume logic
        Err("Not implemented".to_string())
    }
}

impl<I, T> ConsumerChannel<I, T>
where
    I: InnerChannel,
    T: ConsumerItem,
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
