use crate::simplequeue::connector::Connector;

pub mod inner;

pub use inner::InnerConnection;

pub trait Connection {
    fn new<I: InnerConnection>(inner: I, connector: Connector) -> Self;
}
