use std::fmt::Debug;
use std::hash::Hash;

use crate::simplequeue::channel::{ChannelType, QueueIdentifier};

pub mod consumer_channel;
pub mod inner;
pub mod producer_channel;

pub use consumer_channel::ConsumerChannel;
pub use consumer_channel::ConsumerItem;
pub use inner::InnerChannel;
pub use producer_channel::ProducerChannel;
pub use producer_channel::ProducerItem;
pub trait Channel: Send + Sync + 'static + Debug {
    type Inner: InnerChannel;

    fn inner(&self) -> &Self::Inner;
    fn channel_type(&self) -> ChannelType;
    fn queue_identifier(&self) -> &QueueIdentifier;
}

/// Supertrait for items that are sent or received through channels.
/// This trait is used to enforce that items are thread-safe, so that they can be sent and received across threads.
pub trait Item:
    Send + Sync + 'static + Debug + Clone + Hash + Eq + PartialEq + Ord + PartialOrd
{
}
