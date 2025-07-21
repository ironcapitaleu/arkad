use lapin::Connection as LapinConnection;

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
