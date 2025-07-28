/// The role of a channel in the queue system.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChannelType {
    Producer,
    Consumer,
}
