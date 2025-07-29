use crate::simplequeue::channel::{ChannelType, QueueIdentifier};

pub mod inner;

pub use inner::InnerChannel;

pub trait Channel {
    type Inner: InnerChannel;

    fn inner(&self) -> &Self::Inner;
    fn channel_type(&self) -> ChannelType;
    fn queue_identifier(&self) -> &QueueIdentifier;
}
