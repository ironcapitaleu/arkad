use crate::simplequeue::connector::Connector;

pub mod inner;

pub use inner::InnerConnection;

pub trait Connection {
    fn new<I: InnerConnection>(inner: I, connector: Connector) -> Self;
}

pub trait Channel {
    type Inner: InnerConnection;

    fn inner(&self) -> &Self::Inner;
    fn connector(&self) -> &Connector;
}
