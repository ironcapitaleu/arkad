use lapin::Connection as LapinConnection;

use async_trait::async_trait;

use crate::queue::error::connection_failed::ConnectionFailed;
use crate::queue::traits::InnerConnection;

pub mod channel;
pub mod connection;

#[async_trait]
impl InnerConnection for LapinConnection {
    async fn connect(&self, uri: &str) -> Result<Self, ConnectionFailed> {
        let lapin_connection_result =
            Self::connect(uri, lapin::ConnectionProperties::default()).await;

        lapin_connection_result.map_err(|lapin_error| ConnectionFailed::from((uri, lapin_error)))
    }
}
