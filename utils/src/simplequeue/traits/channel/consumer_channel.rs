pub trait ConsumerChannel {
    type Item: Send + Sync + 'static;

    fn consume(&self) -> Result<Self::Item, String>; // TODO: Define a more specific error type
}
