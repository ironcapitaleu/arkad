use super::{Channel, Item};

/// Trait for consumer channels that can receive items from a queue.
///
/// Implementors of this trait are responsible for providing the logic to consume
/// items of type [`Self::Item`] from the underlying queue or channel.
/// Only the consume operation is exposed, even if the inner channel supports more.
pub trait ConsumerChannel: Channel {
    /// The type of item that can be consumed from this channel.
    ///
    /// Must be constructible from a `Vec<u8>` (for deserialization from queue data).
    type Item: Item;

    /// Consumes (receives) an item from the channel.
    ///
    /// # Errors
    ///
    /// Returns an `Err(String)` if the item could not be consumed, for example due to
    /// deserialization errors, connection issues, or queue-specific constraints.
    fn consume(&self) -> Result<Self::Item, String>;
}
