use super::Channel;

/// Trait for consumer channels that can receive items from a queue.
///
/// Implementors of this trait are responsible for providing the logic to consume
/// items of type [`Self::Item`] from the underlying queue or channel.
pub trait ConsumerChannel: Channel {
    /// The item type that can be consumed by this channel.
    type Item: Send + Sync + 'static;

    /// Consumes (receives) an item from the channel.
    ///
    /// # Errors
    ///
    /// Returns an `Err(String)` if the item could not be consumed, for example due to
    /// deserialization errors, connection issues, or queue-specific constraints.
    fn consume(&self) -> Result<Self::Item, String>; // TODO: Define a more specific error type
}
