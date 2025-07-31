use std::fmt::Debug;

use crate::simplequeue::channel::{ChannelType, QueueIdentifier};
use crate::simplequeue::traits::{Channel, InnerChannel, ProducerChannel as ProducerChannelTrait};

#[derive(Debug, Clone)]
pub struct ProducerChannel<I, T>
where
    I: InnerChannel + Debug + Send + Sync + 'static,
    T: Into<Vec<u8>> + Send + Sync + 'static,
{
    inner: I,
    queue_identifier: QueueIdentifier,
    _marker: std::marker::PhantomData<T>,
}

impl<I, T> Channel for ProducerChannel<I, T>
where
    I: InnerChannel + Debug + Send + Sync + 'static,
    T: Into<Vec<u8>> + Debug + Send + Sync + 'static,
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
    I: InnerChannel + Debug + Send + Sync + 'static,
    T: Into<Vec<u8>> + Debug + Send + Sync + 'static,
{
    type Item = T;

    fn produce(&self, _item: Self::Item) -> Result<(), String> {
        // TODO: Implement actual produce logic
        Err("Not implemented".to_string())
    }
}
impl<I, T> ProducerChannel<I, T>
where
    I: InnerChannel + Debug + Send + Sync + 'static,
    T: Into<Vec<u8>> + Send + Sync + 'static,
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
