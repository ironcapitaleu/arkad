use crate::queue::error::ConnectionFailed;
use crate::queue::traits::InnerConnection;

use async_trait::async_trait;

#[derive(Debug)]
pub struct InvalidFakeConnection;

#[async_trait]
impl InnerConnection for InvalidFakeConnection {
    async fn connect(&self, uri: &str) -> Result<Self, ConnectionFailed> {
        Err(ConnectionFailed::new(
            uri,
            format!("Failed to connect to URI: '{uri}'"),
        ))
    }
}
