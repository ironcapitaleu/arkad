use lapin::Connection as LapinConnection;

pub use super::connector::Connector;

#[derive(Debug)]
pub struct Connection {
    pub inner: LapinConnection,
    pub connector: Connector,
}

impl Connection {
    #[must_use]
    pub const fn new(inner: LapinConnection, connector: Connector) -> Self {
        Self { inner, connector }
    }
}
