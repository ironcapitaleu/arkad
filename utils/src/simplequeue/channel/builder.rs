use super::{ConsumerChannel, ProducerChannel, QueueIdentifier};

/// Marker types for tracking which fields have been set
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoChannelType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoQueueIdentifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoInner;

/// Marker type that enables building a [`ProducerChannel`].
///
/// When this marker (along with the other required fields of the builder!) is set via [`ChannelBuilder::producer()`](ChannelBuilder::producer()),
/// the [`ChannelBuilder`] 's `fn build(self) -> ProducerChannel` method becomes available and returns a [`ProducerChannel`].
/// This corresponds to [`ChannelType::Producer`](super::ChannelType::Producer) semantically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProducerChannelMarker;

/// Marker type that enables building a [`ConsumerChannel`].
///
/// When this marker (along with the other required fields of the builder!) is set via [`ChannelBuilder::consumer()`](ChannelBuilder::consumer()),
/// the [`ChannelBuilder`] 's `fn build(self) -> ConsumerChannel` method becomes available and returns a [`ConsumerChannel`].
/// This corresponds to [`ChannelType::Consumer`](super::ChannelType::Consumer) semantically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerChannelMarker;

/// Builder for creating [`Channel`](super::Channel) instances with compile-time type safety.
///
/// This builder uses a type-safe consuming builder pattern to ensure all required fields
/// are set before building and automatically determines the correct [`ChannelType`](super::ChannelType) at compile time.
///
/// # Examples
///
/// ```
/// use utils::simplequeue::channel::{ChannelBuilder, QueueIdentifier};
///
/// // Producer channel - automatically returns a `ProducerChannel` instance
/// let producer = ChannelBuilder::new()
///     .producer()
///     .queue_identifier(QueueIdentifier::BatchExtractor)
///     .inner("connection_string".to_string())
///     .build();
///
/// // Consumer channel - automatically returns a `ConsumerChannel` instance
/// let consumer = ChannelBuilder::new()
///     .consumer()
///     .queue_identifier(QueueIdentifier::BatchExtractor)
///     .inner("connection_string".to_string())
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelBuilder<CT, QI, I> {
    channel_type: CT,
    queue_identifier: QI,
    inner: I,
}

impl ChannelBuilder<NoChannelType, NoQueueIdentifier, NoInner> {
    /// Creates a new channel builder with no fields set.
    ///
    /// # Returns
    /// A new `ChannelBuilder` instance with all fields unset.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            channel_type: NoChannelType,
            queue_identifier: NoQueueIdentifier,
            inner: NoInner,
        }
    }
}

impl<QI, I> ChannelBuilder<NoChannelType, QI, I> {
    /// Sets the channel type to Producer.
    ///
    /// # Returns
    /// A new [`ChannelBuilder`] instance configured for building a [`ProducerChannel`].
    #[must_use]
    pub fn producer(self) -> ChannelBuilder<ProducerChannelMarker, QI, I> {
        ChannelBuilder {
            channel_type: ProducerChannelMarker,
            queue_identifier: self.queue_identifier,
            inner: self.inner,
        }
    }

    /// Sets the marker [`ConsumerChannelMarker`] for [`ChannelType`](super::ChannelType) as [`ChannelType::Consumer`](super::ChannelType::Consumer).
    ///
    /// # Returns
    /// A new [`ChannelBuilder`] instance configured for building a [`ConsumerChannel`].
    #[must_use]
    pub fn consumer(self) -> ChannelBuilder<ConsumerChannelMarker, QI, I> {
        ChannelBuilder {
            channel_type: ConsumerChannelMarker,
            queue_identifier: self.queue_identifier,
            inner: self.inner,
        }
    }
}

impl<CT, I> ChannelBuilder<CT, NoQueueIdentifier, I> {
    /// Sets the `queue_identifier` for the channel.
    ///
    /// # Arguments
    /// * `queue_identifier` - The queue identifier for the channel
    ///
    /// # Returns
    /// A new [`ChannelBuilder`] instance with the `queue_identifier` field set.
    #[must_use]
    pub fn queue_identifier(
        self,
        queue_identifier: QueueIdentifier,
    ) -> ChannelBuilder<CT, QueueIdentifier, I> {
        ChannelBuilder {
            channel_type: self.channel_type,
            queue_identifier,
            inner: self.inner,
        }
    }
}

impl<CT, QI> ChannelBuilder<CT, QI, NoInner> {
    /// Sets the wrapped `inner` connection for the [`Channel`](super::Channel).
    ///
    /// # Arguments
    /// * `inner` - The inner connection string or object
    ///
    /// # Returns
    /// A new [`ChannelBuilder`] instance with the `inner` field set.
    #[must_use]
    pub fn inner(self, inner: impl Into<String>) -> ChannelBuilder<CT, QI, String> {
        ChannelBuilder {
            channel_type: self.channel_type,
            queue_identifier: self.queue_identifier,
            inner: inner.into(),
        }
    }
}

impl ChannelBuilder<ProducerChannelMarker, QueueIdentifier, String> {
    /// Builds a [`ProducerChannel`].
    ///
    /// This method is only available when all required fields have been set
    /// and the marker for [`ChannelType`](super::ChannelType) is set to [`ChannelType::Producer`](super::ChannelType::Producer).
    ///
    /// # Returns
    /// A fully configured [`ProducerChannel`] instance.
    #[must_use]
    pub fn build(self) -> ProducerChannel {
        ProducerChannel::new(self.inner, self.queue_identifier)
    }
}

impl ChannelBuilder<ConsumerChannelMarker, QueueIdentifier, String> {
    /// Builds a [`ConsumerChannel`].
    ///
    /// This method is only available when all required fields have been set
    /// and the marker for [`ChannelType`](super::ChannelType) is set to [`ChannelType::Consumer`](super::ChannelType::Consumer).
    ///
    /// # Returns
    /// A fully configured [`ConsumerChannel`] instance.
    #[must_use]
    pub fn build(self) -> ConsumerChannel {
        ConsumerChannel::new(self.inner, self.queue_identifier)
    }
}

impl Default for ChannelBuilder<NoChannelType, NoQueueIdentifier, NoInner> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::simplequeue::channel::{Channel, ChannelBuilder, ChannelType, QueueIdentifier};

    #[test]
    fn should_build_producer_channel_when_producer_type_is_specified() {
        let expected_queue_identifier = QueueIdentifier::BatchExtractor;
        let expected_inner = "test_connection".to_string();

        let result = ChannelBuilder::new()
            .producer()
            .queue_identifier(QueueIdentifier::BatchExtractor)
            .inner(expected_inner.clone())
            .build();

        assert_eq!(result.inner(), &expected_inner);
        assert_eq!(result.queue_identifier(), &expected_queue_identifier);
    }

    #[test]
    fn should_build_consumer_channel_when_consumer_type_is_specified() {
        let expected_queue_identifier = QueueIdentifier::BatchTransformer;
        let expected_inner = "test_connection".to_string();

        let result = ChannelBuilder::new()
            .consumer()
            .queue_identifier(QueueIdentifier::BatchTransformer)
            .inner(expected_inner.clone())
            .build();

        assert_eq!(result.inner(), &expected_inner);
        assert_eq!(result.queue_identifier(), &expected_queue_identifier);
    }

    #[test]
    fn should_allow_setting_fields_in_any_order_when_using_channel_builder() {
        let expected_queue_identifier = QueueIdentifier::BatchLoader;
        let expected_inner = "test_connection".to_string();

        let result = ChannelBuilder::new()
            .inner(expected_inner.clone())
            .queue_identifier(QueueIdentifier::BatchLoader)
            .producer()
            .build();

        assert_eq!(result.inner(), &expected_inner);
        assert_eq!(result.queue_identifier(), &expected_queue_identifier);
    }

    #[test]
    fn should_create_default_builder_when_new_is_used_with_no_fields_set() {
        let expected_result = ChannelBuilder::default();

        let result = ChannelBuilder::new();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_producer_channel_when_producer_marker_is_set() {
        let expected_channel_type = ChannelType::Producer;

        let result: ProducerChannel = ChannelBuilder::new()
            .producer()
            .queue_identifier(QueueIdentifier::BatchExtractor)
            .inner("test".to_string())
            .build();

        assert_eq!(result.channel_type(), expected_channel_type);
    }

    #[test]
    fn should_return_consumer_channel_when_consumer_marker_is_set() {
        let expected_channel_type = ChannelType::Consumer;

        let result: ConsumerChannel = ChannelBuilder::new()
            .consumer()
            .queue_identifier(QueueIdentifier::BatchExtractor)
            .inner("test".to_string())
            .build();

        assert_eq!(result.channel_type(), expected_channel_type);
    }
}
