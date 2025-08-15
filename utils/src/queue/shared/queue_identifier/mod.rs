use std::fmt;

pub mod constants;

pub use constants::{
    BATCH_EXTRACTOR_QUEUE_NAME, BATCH_LOADER_QUEUE_NAME, BATCH_TRANSFORMER_QUEUE_NAME,
};

/// Identifies a queue and provides its constant name.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl fmt::Display for QueueIdentifier {
    /// Formats the [`QueueIdentifier`] as its queue name string.
    ///
    /// This implementation allows calling `to_string()` on [`QueueIdentifier`] instances
    /// to get their string representation. It is based on the [`queue_name`](QueueIdentifier::queue_name) method.
    ///
    /// # Examples
    /// ```
    /// use utils::queue::shared::queue_identifier::QueueIdentifier;
    ///
    /// let identifier = QueueIdentifier::BatchExtractor;
    /// println!("The name of the queue is: {}", identifier.to_string());
    /// assert_eq!(identifier.to_string(), identifier.queue_name());
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.queue_name())
    }
}
