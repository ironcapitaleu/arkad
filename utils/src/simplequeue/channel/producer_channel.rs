use lapin::Channel as LapinChannel;

use super::{ChannelType, QueueIdentifier};
use crate::simplequeue::traits::{Channel, InnerChannel};

pub struct ProducerChannel<I: InnerChannel> {
    inner: I,
    queue_identifier: QueueIdentifier,
}

impl<I: InnerChannel> Channel for ProducerChannel<I> {
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

impl<I: InnerChannel> ProducerChannel<I> {
    #[must_use]
    pub const fn new(inner: I, queue_identifier: QueueIdentifier) -> Self {
        Self {
            inner,
            queue_identifier,
        }
    }
}
