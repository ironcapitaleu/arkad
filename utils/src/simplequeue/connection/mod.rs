use crate::simplequeue::traits::InnerConnection;

pub use super::connector::Connector;

#[derive(Debug)]
pub struct Connection<I: InnerConnection> {
    pub inner: I,
    pub connector: Connector,
}

impl<I: InnerConnection> Connection<I> {
    #[must_use]
    pub const fn new(inner: I, connector: Connector) -> Self {
        Self { inner, connector }
    }
}
