use lapin::Connection as LapinConnection;

#[derive(Debug)]
pub struct Connection {
    pub inner: LapinConnection,
}

impl Connection {
    pub fn new(inner: LapinConnection) -> Self {
        Connection { inner }
    }
}
