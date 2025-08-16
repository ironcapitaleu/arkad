use std::fmt;

use crate::queue::shared::channel_type::ChannelType;
use crate::queue::shared::queue_identifier::QueueIdentifier;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelConfig {
    pub channel_type: ChannelType,
    pub queue_identifier: QueueIdentifier,
}

impl fmt::Display for ChannelConfig {
    /// Formats the [`ChannelConfig`] as: "<[`channel_type`](crate::queue::shared::ChannelType)>@<[`queue_identifier`](crate::queue::shared::QueueIdentifier)>"
    ///
    /// # Example
    /// ```
    /// use utils::queue::shared::{ChannelConfig, ChannelType, QueueIdentifier};
    ///
    /// let config = ChannelConfig {
    ///     channel_type: ChannelType::Producer,
    ///     queue_identifier: QueueIdentifier::BatchExtractor,
    /// };
    /// assert_eq!(format!("{config}"), "Producer@batch_extractor_queue");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.channel_type, self.queue_identifier)
    }
}

impl ChannelConfig {
    /// Returns the [`ChannelType`] for this configuration.
    ///
    /// # Example
    /// ```
    /// use utils::queue::shared::{ChannelConfig, ChannelType, QueueIdentifier};
    ///
    /// let config = ChannelConfig {
    ///     channel_type: ChannelType::Producer,
    ///     queue_identifier: QueueIdentifier::BatchExtractor,
    /// };
    /// assert_eq!(config.get_channel_type(), ChannelType::Producer);
    /// ```
    #[must_use]
    pub const fn get_channel_type(&self) -> ChannelType {
        self.channel_type
    }

    /// Returns the [`QueueIdentifier`] for this configuration.
    ///
    /// # Example
    /// ```
    /// use utils::queue::shared::{ChannelConfig, ChannelType, QueueIdentifier};
    ///
    /// let config = ChannelConfig {
    ///     channel_type: ChannelType::Producer,
    ///     queue_identifier: QueueIdentifier::BatchExtractor,
    /// };
    /// assert_eq!(config.get_queue_identifier(), QueueIdentifier::BatchExtractor);
    /// ```
    #[must_use]
    pub const fn get_queue_identifier(&self) -> QueueIdentifier {
        self.queue_identifier
    }
}
