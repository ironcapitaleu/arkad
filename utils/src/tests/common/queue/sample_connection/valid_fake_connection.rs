use async_trait::async_trait;

use crate::queue::error::ConnectionFailed;
use crate::queue::traits::InnerConnection;

#[derive(Debug)]
pub struct ValidFakeConnection;

#[async_trait]
impl InnerConnection for ValidFakeConnection {
    async fn connect(&self, uri: &str) -> Result<Self, ConnectionFailed> {
        println!("Fake connection established with URI: '{uri}'");
        Ok(ValidFakeConnection)
    }
}
