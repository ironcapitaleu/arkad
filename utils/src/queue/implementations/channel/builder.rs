//! Channel builder implementation providing compile-time type safety.
//!
//! This module contains the [`ChannelBuilder`] and associated marker types that enable
//! type-safe construction of [`Channel`](crate::queue::traits::Channel). The builder uses a consuming type state pattern to ensure
//! all required fields are set before construction and automatically determines the
//! correct [`ChannelType`](crate::queue::shared::ChannelType) at compile time.
//!
//! # Type Safety
//!
//! The builder prevents invalid states through compile-time type checking:
//! - Cannot call `build()` without setting all required fields
//! - Cannot set the same field twice
//! - Cannot set item type before specifying channel type (producer/consumer)
//! - Enforces appropriate trait bounds for both channel types:
//!   - Producer channels, as well as Consumer channels require items that implement the [`Item`] trait
//! - Automatically returns the correct [`ChannelType`](crate::queue::shared::ChannelType) based on marker
//!
//! # Examples
//!
//! ```compile_fail
//! use utils::queue::channel::{ChannelBuilder, QueueIdentifier};
//!
//! let sample_inner_channel = ...; // Replace with actual type that implements `InnerChannel`
//! let example_item: ExampleItemType = ...; // Replace with actual type that implements `Item`
//!
//! // Producer channel
//! let producer = ChannelBuilder::new()
//!     .producer()
//!     .item_type::<ExampleItemType>() // `ExampleItemType` implements the `Item` trait
//!     .queue_identifier(QueueIdentifier::BatchExtractor)
//!     .inner(sample_inner_channel)
//!     .build(); // Automatically returns a `ProducerChannel<I, String>`
//!
//! // Consumer channel
//! let consumer = ChannelBuilder::new()
//!     .consumer()
//!     .item_type::<ExampleItemType>() // `ExampleItemType` implements the `Item` trait
//!     .queue_identifier(QueueIdentifier::BatchExtractor)
//!     .inner(sample_inner_channel)
//!     .build(); // Automatically returns a `ConsumerChannel<I, Vec<u8>>`
//! ```

use crate::queue::traits::{InnerChannel, Item};

use crate::queue::implementations::channel::{ConsumerChannel, ProducerChannel};
use crate::queue::shared::queue_identifier::QueueIdentifier;

/// Marker type for tracking when no channel type has been set in the builder.
///
/// This marker prevents the builder from calling any channel-type-specific methods, such as `build()`
/// until either [`producer()`](ChannelBuilder::producer) or [`consumer()`](ChannelBuilder::consumer) is called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoChannelType;

/// Marker type for tracking when no queue identifier has been set in the builder.
///
/// This marker prevents the builder from calling `build()`
/// until [`queue_identifier()`](ChannelBuilder::queue_identifier) is called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoQueueIdentifier;

/// Marker type for tracking when no inner channel has been set in the builder.
///
/// This marker prevents the builder from calling `build()`
/// until [`inner()`](ChannelBuilder::inner) is called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoInner;

/// Marker type for tracking when no item type has been set in the builder.
///
/// This marker prevents the builder from calling `build()`
/// until [`item_type()`](ChannelBuilder::item_type) is called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoItemType;

/// Marker type that enables building a [`ProducerChannel`].
///
/// When this marker (along with the other required fields of the builder!) is set via [`ChannelBuilder::producer()`](ChannelBuilder::producer()),
/// the [`ChannelBuilder`] 's `fn build(self) -> ProducerChannel` method becomes available and returns a [`ProducerChannel`].
/// This corresponds to [`ChannelType::Producer`](crate::queue::shared::ChannelType::Producer) semantically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProducerChannelMarker;

/// Marker type that enables building a [`ConsumerChannel`].
///
/// When this marker (along with the other required fields of the builder!) is set via [`ChannelBuilder::consumer()`](ChannelBuilder::consumer()),
/// the [`ChannelBuilder`] 's `fn build(self) -> ConsumerChannel` method becomes available and returns a [`ConsumerChannel`].
/// This corresponds to [`ChannelType::Consumer`](crate::queue::shared::ChannelType::Consumer) semantically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerChannelMarker;

/// Builder for creating [`Channel`](crate::queue::traits::Channel) instances with compile-time type safety.
///
/// This builder uses a type-safe consuming builder pattern to ensure all required fields
/// are set before building and automatically determines the correct [`ChannelType`](crate::queue::shared::ChannelType) at compile time.
/// The item type can only be specified after the channel type (producer/consumer) has been set,
/// ensuring appropriate trait bounds are enforced.
///
/// # Examples
///
/// ```compile_fail
/// use utils::queue::channel::{ChannelBuilder, QueueIdentifier};
///
/// let sample_inner_channel = ...; // Replace with actual type that implements `InnerChannel`
/// let example_item = ...; // Replace with actual type that implements the `Item` trait
///
/// // Producer channel - automatically returns a `ProducerChannel` instance
/// let producer = ChannelBuilder::new()
///     .producer()
///     .item_type::<ExampleItemType>() // `ExampleItemType` implements the `Item` trait
///     .queue_identifier(QueueIdentifier::BatchExtractor)
///     .inner(sample_inner_channel)
///     .build();
///
/// // Consumer channel - automatically returns a `ConsumerChannel` instance
/// let consumer = ChannelBuilder::new()
///     .consumer()
///     .item_type::<ExampleItemType>()    // `ExampleItemType` implements the `Item` trait
///     .queue_identifier(QueueIdentifier::BatchExtractor)
///     .inner(sample_inner_channel)
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelBuilder<CT, QI, I, T> {
    channel_type: CT,
    queue_identifier: QI,
    inner: I,
    _item_type: std::marker::PhantomData<T>,
}

impl ChannelBuilder<NoChannelType, NoQueueIdentifier, NoInner, NoItemType> {
    /// Creates a new [`ChannelBuilder`] with no fields set.
    ///
    /// # Returns
    /// A new [`ChannelBuilder`] instance with all fields unset.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            channel_type: NoChannelType,
            queue_identifier: NoQueueIdentifier,
            inner: NoInner,
            _item_type: std::marker::PhantomData,
        }
    }
}

impl<QI, I, T> ChannelBuilder<NoChannelType, QI, I, T> {
    /// Configures the [`ChannelBuilder`] to produce a [`ProducerChannel`] when built.
    ///
    /// This method transitions the builder from having the [`NoChannelType`] marker set to being
    /// configured with [`ProducerChannelMarker`], enabling construction of a [`ProducerChannel`].
    /// Once called, the builder can only be used to create producer channels.
    ///
    /// # Returns
    /// A new [`ChannelBuilder`] instance configured for building a [`ProducerChannel`].
    ///
    /// # Examples
    /// ```compile_fail
    /// use utils::queue::channel::{ChannelBuilder, QueueIdentifier};
    ///
    /// let sample_inner_channel = ...; // Replace with actual type that implements `InnerChannel`
    ///
    /// let builder = ChannelBuilder::new()
    ///     .producer()
    ///     .queue_identifier(QueueIdentifier::BatchExtractor)
    ///     .inner(sample_inner_channel);
    ///
    /// let channel = builder.build(); // Returns a `ProducerChannel`
    /// ```
    #[must_use]
    pub fn producer(self) -> ChannelBuilder<ProducerChannelMarker, QI, I, T> {
        ChannelBuilder {
            channel_type: ProducerChannelMarker,
            queue_identifier: self.queue_identifier,
            inner: self.inner,
            _item_type: self._item_type,
        }
    }

    /// Configures the [`ChannelBuilder`] to produce a [`ConsumerChannel`] when built.
    ///
    /// This method transitions the builder from having the [`NoChannelType`] marker set to being
    /// configured with [`ConsumerChannelMarker`], enabling construction of a [`ConsumerChannel`].
    /// Once called, the builder can only be used to create consumer channels.
    ///
    /// # Returns
    /// A new [`ChannelBuilder`] instance configured for building a [`ConsumerChannel`].
    ///
    /// # Examples
    /// ```compile_fail
    /// use utils::queue::channel::{ChannelBuilder, QueueIdentifier};
    ///
    /// let sample_inner_channel = ...; // Replace with actual type that implements `InnerChannel`
    ///
    /// let builder = ChannelBuilder::new()
    ///     .consumer()
    ///     .queue_identifier(QueueIdentifier::BatchExtractor)
    ///     .inner(sample_inner_channel);
    ///
    /// let channel = builder.build(); // Returns a `ConsumerChannel`
    /// ```
    #[must_use]
    pub fn consumer(self) -> ChannelBuilder<ConsumerChannelMarker, QI, I, T> {
        ChannelBuilder {
            channel_type: ConsumerChannelMarker,
            queue_identifier: self.queue_identifier,
            inner: self.inner,
            _item_type: self._item_type,
        }
    }
}

impl<CT, QI, I> ChannelBuilder<CT, QI, I, NoItemType> {
    /// Sets the item type that will be written by / consumed by this channel.
    ///
    /// # Type Parameters
    /// * `T` - The item type that implements the [`Item`] trait.
    ///
    /// # Returns
    /// A new [`ChannelBuilder`] instance with the item type configured.
    ///
    /// # Examples
    /// ```compile_fail
    /// let example_item: ExampleItemType = ...; // Replace with actual type that implements the `Item` trait
    ///
    /// let consumer = ChannelBuilder::new()
    ///     .consumer()
    ///     .item_type::<ExampleItemType>()  // `ExampleItemType` implements the `Item` trait
    ///     .queue_identifier(QueueIdentifier::BatchExtractor)
    ///     .inner(sample_inner_channel)
    ///     .build();
    /// ```
    #[must_use]
    pub fn item_type<T>(self) -> ChannelBuilder<CT, QI, I, T>
    where
        T: Item,
    {
        ChannelBuilder {
            channel_type: self.channel_type,
            queue_identifier: self.queue_identifier,
            inner: self.inner,
            _item_type: std::marker::PhantomData::<T>,
        }
    }
}

impl<CT, I, T> ChannelBuilder<CT, NoQueueIdentifier, I, T> {
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
    ) -> ChannelBuilder<CT, QueueIdentifier, I, T> {
        ChannelBuilder {
            channel_type: self.channel_type,
            queue_identifier,
            inner: self.inner,
            _item_type: self._item_type,
        }
    }
}

impl<CT, QI, T> ChannelBuilder<CT, QI, NoInner, T> {
    /// Sets the wrapped `inner` channel for the [`Channel`](crate::queue::traits::Channel).
    ///
    /// # Arguments
    /// * `inner` - The inner Lapin channel object
    ///
    /// # Returns
    /// A new [`ChannelBuilder`] instance with the `inner` field set.
    #[must_use]
    pub fn inner<I: InnerChannel>(self, inner: I) -> ChannelBuilder<CT, QI, I, T> {
        ChannelBuilder {
            channel_type: self.channel_type,
            queue_identifier: self.queue_identifier,
            inner,
            _item_type: self._item_type,
        }
    }
}

impl<I: InnerChannel, T> ChannelBuilder<ProducerChannelMarker, QueueIdentifier, I, T>
where
    T: Item,
{
    /// Builds a [`ProducerChannel`] from the fully configured [`ChannelBuilder`] state.
    ///
    /// This method consumes the [`ChannelBuilder`] and creates a new [`ProducerChannel`] instance
    /// using the configured inner channel and queue identifier. This method is only
    /// available when:
    /// - [`ChannelType`](crate::queue::shared::ChannelType) is set to [`ChannelType::Producer`](crate::queue::shared::ChannelType::Producer) via [`producer()`](ChannelBuilder::producer)
    /// - Queue identifier is set via [`queue_identifier()`](ChannelBuilder::queue_identifier)
    /// - Inner channel is set via [`inner()`](ChannelBuilder::inner)
    /// - Item type is set via [`item_type()`](ChannelBuilder::item_type)
    ///
    /// # Returns
    /// A fully configured [`ProducerChannel`] instance ready for use.
    ///
    /// # Examples
    /// ```compile_fail
    /// use utils::queue::channel::{ChannelBuilder, QueueIdentifier};
    ///
    /// let sample_inner_channel = ...; // Replace with actual type that implements `InnerChannel`
    /// let example_item: ExampleItemType = ...; // Replace with actual type that implements `Item`
    ///
    /// let producer = ChannelBuilder::new()
    ///     .producer()
    ///     .item_type::<ExampleItemType>()
    ///     .queue_identifier(QueueIdentifier::BatchExtractor)
    ///     .inner(sample_inner_channel)
    ///     .build(); // This method is now available
    /// ```
    #[must_use]
    pub fn build(self) -> ProducerChannel<I, T> {
        ProducerChannel::new(self.inner, self.queue_identifier)
    }
}

impl<I: InnerChannel, T> ChannelBuilder<ConsumerChannelMarker, QueueIdentifier, I, T>
where
    T: Item,
{
    /// Builds a [`ConsumerChannel`] from the fully configured [`ChannelBuilder`] state.
    ///
    /// This method consumes the builder and creates a new [`ConsumerChannel`] instance
    /// using the configured inner channel and queue identifier. This method is only
    /// available when:
    /// - [`ChannelType`](crate::queue::shared::ChannelType) is set to [`ChannelType::Consumer`](crate::queue::shared::ChannelType::Consumer) via [`consumer()`](ChannelBuilder::consumer)
    /// - Queue identifier is set via [`queue_identifier()`](ChannelBuilder::queue_identifier)
    /// - Inner channel is set via [`inner()`](ChannelBuilder::inner)
    /// - Item type is set via [`item_type()`](ChannelBuilder::item_type)
    ///
    /// # Returns
    /// A fully configured [`ConsumerChannel`] instance ready for use.
    ///
    /// # Examples
    /// ```compile_fail
    /// use utils::queue::channel::{ChannelBuilder, QueueIdentifier};
    ///
    /// let sample_inner_channel = ...; // Replace with actual type that implements `InnerChannel`
    /// let example_item: ExampleItemType = ...; // Replace with actual type that implements `Item`
    ///
    /// let consumer = ChannelBuilder::new()
    ///     .consumer()
    ///     .item_type::<ExampleItemType>()
    ///     .queue_identifier(QueueIdentifier::BatchExtractor)
    ///     .inner(sample_inner_channel)
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> ConsumerChannel<I, T> {
        ConsumerChannel::new(self.inner, self.queue_identifier)
    }
}

impl Default for ChannelBuilder<NoChannelType, NoQueueIdentifier, NoInner, NoItemType> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::queue::implementations::channel::ChannelBuilder;
    use crate::queue::shared::{ChannelType, QueueIdentifier};
    use crate::queue::traits::Channel;
    use crate::tests::common::queue::{sample_channel::FakeChannel, sample_item::FakeItem};

    #[test]
    fn should_build_producer_channel_when_producer_type_is_specified() {
        let fake_channel = FakeChannel;

        let _result: ProducerChannel<FakeChannel, FakeItem> = ChannelBuilder::new()
            .producer()
            .item_type::<FakeItem>()
            .queue_identifier(QueueIdentifier::BatchExtractor)
            .inner(fake_channel)
            .build();
    }

    #[test]
    fn should_build_consumer_channel_when_consumer_type_is_specified() {
        let fake_channel = FakeChannel;

        let _result: ConsumerChannel<FakeChannel, FakeItem> = ChannelBuilder::new()
            .consumer()
            .item_type::<FakeItem>()
            .queue_identifier(QueueIdentifier::BatchTransformer)
            .inner(fake_channel)
            .build();
    }

    #[test]
    fn should_allow_setting_fields_in_any_order_when_using_channel_builder() {
        let fake_channel = FakeChannel;

        let _result: ProducerChannel<FakeChannel, FakeItem> = ChannelBuilder::new()
            .producer()
            .item_type::<FakeItem>()
            .inner(fake_channel)
            .queue_identifier(QueueIdentifier::BatchLoader)
            .build();
    }

    #[test]
    fn should_create_default_builder_when_new_is_used_with_no_fields_set() {
        let expected_result = ChannelBuilder::default();

        let result = ChannelBuilder::new();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_producer_channel_type_when_producer_marker_is_set_for_builder() {
        let fake_channel = FakeChannel;

        let expected_result = ChannelType::Producer;

        let channel: ProducerChannel<FakeChannel, FakeItem> = ChannelBuilder::new()
            .producer()
            .item_type::<FakeItem>()
            .queue_identifier(QueueIdentifier::BatchExtractor)
            .inner(fake_channel)
            .build();
        let result = channel.channel_type();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_consumer_channel_type_when_consumer_marker_is_set_for_builder() {
        let fake_channel = FakeChannel;

        let expected_result = ChannelType::Consumer;

        let channel: ConsumerChannel<FakeChannel, FakeItem> = ChannelBuilder::new()
            .consumer()
            .item_type::<FakeItem>()
            .queue_identifier(QueueIdentifier::BatchExtractor)
            .inner(fake_channel)
            .build();
        let result = channel.channel_type();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_preserve_queue_identifier_when_building_consumer_channel() {
        let fake_channel = FakeChannel;

        let queue_identifier = QueueIdentifier::BatchLoader;

        let expected_result = &queue_identifier.clone();

        let channel: ConsumerChannel<FakeChannel, FakeItem> = ChannelBuilder::new()
            .consumer()
            .item_type::<FakeItem>()
            .queue_identifier(queue_identifier.clone())
            .inner(fake_channel)
            .build();
        let result = channel.queue_identifier();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_preserve_queue_identifier_when_building_producer_channel() {
        let fake_channel = FakeChannel;

        let queue_identifier = QueueIdentifier::BatchLoader;

        let expected_result = &queue_identifier.clone();

        let channel: ProducerChannel<FakeChannel, FakeItem> = ChannelBuilder::new()
            .producer()
            .item_type::<FakeItem>()
            .queue_identifier(queue_identifier.clone())
            .inner(fake_channel)
            .build();
        let result = channel.queue_identifier();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_allow_setting_fields_in_different_order_when_building_consumer_channel() {
        let fake_channel = FakeChannel;

        let expected_result = ChannelType::Consumer;

        let channel: ConsumerChannel<FakeChannel, FakeItem> = ChannelBuilder::new()
            .queue_identifier(QueueIdentifier::BatchLoader)
            .consumer()
            .item_type::<FakeItem>()
            .inner(fake_channel)
            .build();
        let result = channel.channel_type();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_allow_setting_fields_in_different_order_when_building_producer_channel() {
        let fake_channel = FakeChannel;

        let expected_result = ChannelType::Producer;

        let channel = ChannelBuilder::new()
            .queue_identifier(QueueIdentifier::BatchLoader)
            .producer()
            .item_type::<FakeItem>()
            .inner(fake_channel)
            .build();
        let result = channel.channel_type();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_allow_setting_item_type_when_channel_type_is_not_set_yet() {
        let fake_channel = FakeChannel;

        let expected_result = ChannelType::Producer;

        let channel = ChannelBuilder::new()
            .item_type::<FakeItem>()
            .producer()
            .inner(fake_channel)
            .queue_identifier(QueueIdentifier::BatchLoader)
            .build();
        let result = channel.channel_type();

        assert_eq!(result, expected_result);
    }
}
