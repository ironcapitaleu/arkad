pub mod inner;

use crate::queue::traits::{Connection as ConnectionTrait, InnerConnection};

pub use super::connector::Connector;

#[derive(Debug)]
pub struct Connection<I: InnerConnection> {
    pub inner: I,
    pub connector: Connector,
}

impl<I: InnerConnection> ConnectionTrait for Connection<I> {
    type Inner = I;
    fn new(inner: I, connector: Connector) -> Self {
        Self { inner, connector }
    }

    fn inner(&self) -> &I {
        &self.inner
    }

    fn connector(&self) -> &Connector {
        &self.connector
    }
}
