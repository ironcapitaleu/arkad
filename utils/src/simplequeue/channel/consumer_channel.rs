use super::{Channel, ChannelType, QueueIdentifier};

pub struct ConsumerChannel {
    inner: String, // Placeholder for actual inner structure
    queue_identifier: QueueIdentifier,
}

impl Channel for ConsumerChannel {
    type Inner = String; // Placeholder for actual inner type

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

impl ConsumerChannel {
    pub fn new(inner: String, queue_identifier: QueueIdentifier) -> Self {
        Self {
            inner,
            queue_identifier,
        }
    }
}
