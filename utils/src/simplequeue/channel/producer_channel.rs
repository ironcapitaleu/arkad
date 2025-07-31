use std::fmt::Debug;

use super::{ChannelType, QueueIdentifier};
use crate::simplequeue::traits::{Channel, InnerChannel};

#[derive(Debug, Clone)]
pub struct ProducerChannel<I>
where
    I: InnerChannel + Debug + Send + Sync + 'static,
{
    inner: I,
    queue_identifier: QueueIdentifier,
}

impl<I> Channel for ProducerChannel<I>
where
    I: InnerChannel + Debug + Send + Sync + 'static,
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

impl<I> ProducerChannel<I>
where
    I: InnerChannel + Debug + Send + Sync + 'static,
{
    #[must_use]
    pub const fn new(inner: I, queue_identifier: QueueIdentifier) -> Self {
        Self {
            inner,
            queue_identifier,
        }
    }
}
