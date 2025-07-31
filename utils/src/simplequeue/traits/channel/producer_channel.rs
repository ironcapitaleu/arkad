use super::Channel;

pub trait ProducerChannel: Channel {
    type Item: Send + Sync + 'static;

    fn produce(&self, item: Self::Item) -> Result<(), String>; // TODO: Define a more specific error type
}
