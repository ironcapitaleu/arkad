use crate::simplequeue::error::ConnectionFailed;
use crate::simplequeue::traits::InnerConnection;

#[derive(Debug)]
pub struct ValidFakeConnection;

impl InnerConnection for ValidFakeConnection {
    async fn connect(&self, uri: &str) -> Result<Self, ConnectionFailed> {
        println!("Fake connection established with URI: '{uri}'");
        Ok(ValidFakeConnection)
    }
}
