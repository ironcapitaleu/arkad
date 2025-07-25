use lapin::Connection as LapinConnection;

pub mod connection_kind;
pub use connection_kind::ConnectionKind;

#[derive(Debug)]
pub struct Connection {
    pub inner: LapinConnection,
}

impl Connection {
    #[must_use]
    pub const fn new(inner: LapinConnection) -> Self {
        Self { inner }
    }
}
