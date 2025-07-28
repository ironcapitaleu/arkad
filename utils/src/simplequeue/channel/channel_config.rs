use crate::simplequeue::constants::queue_identifiers::{
    BATCH_EXTRACTOR_QUEUE_NAME, BATCH_LOADER_QUEUE_NAME, BATCH_TRANSFORMER_QUEUE_NAME,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelConfig {
    pub channel_type: ChannelType,
    pub queue_identifier: QueueIdentifier,
}

/// The role of a channel in the queue system.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChannelType {
    Producer,
    Consumer,
}

/// Identifies a queue and provides its constant name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QueueIdentifier {
    BatchExtractor,
    BatchTransformer,
    BatchLoader,
}

impl QueueIdentifier {
    /// Returns the queue name which is a string representation associated with the [`QueueIdentifier`].
    pub fn queue_name(&self) -> &'static str {
        match self {
            QueueIdentifier::BatchExtractor => BATCH_EXTRACTOR_QUEUE_NAME,
            QueueIdentifier::BatchTransformer => BATCH_TRANSFORMER_QUEUE_NAME,
            QueueIdentifier::BatchLoader => BATCH_LOADER_QUEUE_NAME,
        }
    }
}
