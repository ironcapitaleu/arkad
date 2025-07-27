pub mod builder;
pub mod channel_config;
pub mod consumer_channel;
pub mod producer_channel;

pub use channel_config::{ChannelConfig, ChannelType, QueueIdentifier};
pub use consumer_channel::ConsumerChannel;
pub use producer_channel::ProducerChannel;

pub trait Channel {
    type Inner;

    fn inner(&self) -> &Self::Inner;
    fn channel_type(&self) -> ChannelType;
    fn queue_identifier(&self) -> &QueueIdentifier;
}
