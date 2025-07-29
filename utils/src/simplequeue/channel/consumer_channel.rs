use lapin::Channel as LapinChannel;

use super::{ChannelType, QueueIdentifier};
use crate::simplequeue::traits::{Channel, InnerChannel};

pub struct ConsumerChannel<I: InnerChannel> {
    inner: I,
    queue_identifier: QueueIdentifier,
}

impl<I: InnerChannel> Channel for ConsumerChannel<I> {
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

impl<I: InnerChannel> ConsumerChannel<I> {
    #[must_use]
    pub const fn new(inner: I, queue_identifier: QueueIdentifier) -> Self {
        Self {
            inner,
            queue_identifier,
        }
    }
}
