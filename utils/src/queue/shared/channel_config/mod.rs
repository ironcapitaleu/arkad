use crate::queue::shared::channel_type::ChannelType;
use crate::queue::shared::queue_identifier::QueueIdentifier;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelConfig {
    pub channel_type: ChannelType,
    pub queue_identifier: QueueIdentifier,
}
