use crate::simplequeue::error::ConnectionFailed;
use crate::simplequeue::traits::InnerConnection;

#[derive(Debug)]
pub struct InvalidFakeConnection;

impl InnerConnection for InvalidFakeConnection {
    async fn connect(&self, uri: &str) -> Result<Self, ConnectionFailed> {
        Err(ConnectionFailed::new(
            uri,
            format!("Failed to connect to URI: '{uri}'"),
        ))
    }
}
