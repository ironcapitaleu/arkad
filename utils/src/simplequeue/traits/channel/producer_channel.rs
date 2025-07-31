use super::Channel;

/// Trait for producer channels that can send items to a queue.
///
/// Implementors of this trait are responsible for providing the logic to produce
/// items of type [`Self::Item`] to the underlying queue or channel.
pub trait ProducerChannel: Channel {
    /// The item type that can be produced by this channel.
    type Item: Send + Sync + 'static;

    /// Produces (sends) an item to the channel.
    ///
    /// # Errors
    ///
    /// Returns an `Err(String)` if the item could not be produced, for example due to
    /// serialization errors, connection issues, or queue-specific constraints.
    fn produce(&self, item: Self::Item) -> Result<(), String>; // TODO: Define a more specific error type
}
