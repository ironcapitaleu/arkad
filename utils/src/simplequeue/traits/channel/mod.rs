use std::fmt::Debug;

use crate::simplequeue::channel::{ChannelType, QueueIdentifier};

pub mod consumer_channel;
pub mod inner;
pub mod producer_channel;

pub use consumer_channel::ConsumerChannel;
pub use inner::InnerChannel;
pub use producer_channel::ProducerChannel;

pub trait Channel: Send + Sync + 'static + Debug {
    type Inner: InnerChannel;

    fn inner(&self) -> &Self::Inner;
    fn channel_type(&self) -> ChannelType;
    fn queue_identifier(&self) -> &QueueIdentifier;
}
