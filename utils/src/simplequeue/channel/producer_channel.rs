use lapin::Channel as LapinChannel;

use super::{Channel, ChannelType, QueueIdentifier};

pub struct ProducerChannel {
    inner: LapinChannel,
    queue_identifier: QueueIdentifier,
}

impl Channel for ProducerChannel {
    type Inner = LapinChannel;

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

impl ProducerChannel {
    #[must_use]
    pub const fn new(inner: LapinChannel, queue_identifier: QueueIdentifier) -> Self {
        Self {
            inner,
            queue_identifier,
        }
    }
}
