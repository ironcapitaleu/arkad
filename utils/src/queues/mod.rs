use std::error::Error;

use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::{Channel, Connection, ConnectionProperties};

/// Establishes an AMQP connection to the given address.
///
/// # Arguments
/// * `addr` - AMQP URI string (e.g., "amqp://user:pass@host:port/vhost")
///
/// # Errors
/// Returns an error if the connection cannot be established.
///
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

/// Attempts to passively connect to a queue and prints the result.
///
/// # Arguments
/// * `channel` - A reference to a Lapin channel
/// * `queue_name` - The name of the queue to check
///
/// Prints a success message if the queue exists, otherwise prints the error.
pub async fn check_queue(channel: &Channel, queue_name: &str) {
    let result = channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                passive: true,
                ..QueueDeclareOptions::default()
            },
            FieldTable::default(),
        )
        .await;

    match result {
        Ok(_) => println!("Successfully connected to queue '{}'", queue_name),
        Err(e) => println!("Failed to connect to queue '{}': {}", queue_name, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn should_establish_connection_when_valid_addr_provided() {
        // Arrange
        let addr = "amqp://admin:admin123@localhost:5672/%2f";

        // Define
        let expected_result = true;

        // Act
        let result = establish_connection(addr).await.is_ok();

        // Assert
        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_print_success_or_failure_when_check_queue_called() {
        // Arrange
        let addr = "amqp://admin:admin123@localhost:5672/%2f";
        let queue_name = "extraction.results";

        // Act
        let conn = establish_connection(addr)
            .await
            .expect("Connection should succeed");
        let channel = conn
            .create_channel()
            .await
            .expect("Channel creation should succeed");

        // This test only checks that the function runs without panicking.
        // Output is printed to stdout.
        check_queue(&channel, queue_name).await;

        // Assert
        // No assertion needed; if the function panics, the test will fail.
    }
}
