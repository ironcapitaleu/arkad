use super::{Channel, Item};

/// Trait for producer channels that can send items to a queue.
///
/// Implementors of this trait are responsible for providing the logic to produce
/// items of type [`Self::Item`] to the underlying queue or channel.
/// Only the produce operation is exposed, even if the inner channel supports more.
pub trait ProducerChannel: Channel {
    /// The type of item that can be produced by this channel.
    ///
    /// Must be convertible into a `Vec<u8>` (for serialization or transmission).
    type Item: ProducerItem;

    /// Produces (sends) an item to the channel.
    ///
    /// # Errors
    ///
    /// Returns an `Err(String)` if the item could not be produced, for example due to
    /// serialization errors, connection issues, or queue-specific constraints.
    fn produce(&self, item: Self::Item) -> Result<(), String>;
}

/// This trait is used to enforce that items produced can be sent through the a ProducerChannel channel
/// can be converted into a Vec<u8> for serialization and transmission.

pub trait ProducerItem: Item + Into<Vec<u8>> {}
