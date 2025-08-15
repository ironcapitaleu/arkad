/// The type of a channel in the queue system. Encodes the permissions associated with the channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChannelType {
    /// Allows messages to be sent to the channel. Equivalent to writing permissions to the associated channel.
    Producer,
    /// Allows messages to be received from the channel. Equivalent to reading permissions to the associated channel.
    Consumer,
}
