use std::error::Error;

use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::{Channel, Connection, ConnectionProperties};
use tokio;


/// Establishes an AMQP connection to the given address.
///
/// # Arguments
/// * `addr` - AMQP URI string (e.g., "amqp://user:pass@host:port/vhost")
///
/// # Errors
/// Returns an error if the connection cannot be established.
///
/// # Example
/// ```
/// let conn = establish_connection("amqp://127.0.0.1:5672/%2f").await?;
/// ```
pub async fn establish_connection(addr: &str) -> Result<Connection, Box<dyn Error + Send + Sync>> {
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    Ok(conn)
}

/// Creates a queue with the given name on the provided channel.
///
/// # Arguments
/// * `channel` - A reference to a Lapin channel
/// * `queue_name` - The name of the queue to create
///
/// # Errors
/// Returns an error if the queue cannot be created.
///
/// # Example
/// ```
/// create_queue(&channel, "test-queue").await?;
/// ```
pub async fn create_queue(
    channel: &Channel,
    queue_name: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    
    #[tokio::test]
    async fn should_establish_connection_when_valid_addr_provided() {
        // Arrange
        let addr = "amqp://admin:admin123@localhost:5672/%2f";

        // Define
        let expected_result = true;

        // Act
        let result = establish_connection(addr).await;

        // Assert
        assert!(result.is_ok() == expected_result);
    }
}