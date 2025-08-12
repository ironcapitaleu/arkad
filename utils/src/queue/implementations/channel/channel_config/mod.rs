pub mod channel_type;
pub mod queue_identifier;

pub use channel_type::ChannelType;
pub use queue_identifier::QueueIdentifier;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelConfig {
    pub channel_type: ChannelType,
    pub queue_identifier: QueueIdentifier,
}
