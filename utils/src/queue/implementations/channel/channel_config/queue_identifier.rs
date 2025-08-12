use crate::queue::shared::queue_identifiers::{
    BATCH_EXTRACTOR_QUEUE_NAME, BATCH_LOADER_QUEUE_NAME, BATCH_TRANSFORMER_QUEUE_NAME,
};

/// Identifies a queue and provides its constant name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QueueIdentifier {
    BatchExtractor,
    BatchTransformer,
    BatchLoader,
}

impl QueueIdentifier {
    /// Returns the queue name which is a string representation associated with the [`QueueIdentifier`].
    #[must_use]
    pub const fn queue_name(&self) -> &'static str {
        match self {
            Self::BatchExtractor => BATCH_EXTRACTOR_QUEUE_NAME,
            Self::BatchTransformer => BATCH_TRANSFORMER_QUEUE_NAME,
            Self::BatchLoader => BATCH_LOADER_QUEUE_NAME,
        }
    }
}
