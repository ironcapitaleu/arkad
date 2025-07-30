use crate::simplequeue::traits::InnerConnection;
use crate::simplequeue::error::ConnectionFailed;

#[derive(Debug)]
pub struct FakeConnection;

impl InnerConnection for FakeConnection {
    async fn connect(&self, uri: &str) -> Result<Self, ConnectionFailed> {
        println!("Fake connection established with URI: '{uri}'");
        Ok(FakeConnection)
    }
}
