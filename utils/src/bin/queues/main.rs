use std::env;

use lapin::{BasicProperties, Channel, options::BasicPublishOptions};

use utils::queues::establish_connection;

/// Main entry point for testing RabbitMQ queue connectivity using environment variables.
///
/// Loads configuration from `.env`, establishes a connection to RabbitMQ,
/// and creates the specified queue. Prints the result to stdout.
///
/// # Errors
/// Returns an error if connection or queue creation fails.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting RabbitMQ queue connectivity test...");

    // Arrange: Read connection parameters from environment
    let user = env::var("RABBITMQ_DEFAULT_USER")?;
    let pass = env::var("RABBITMQ_DEFAULT_PASS")?;
    let host = env::var("RABBITMQ_HOST")?;
    let port = env::var("RABBITMQ_PORT")?;
    let vhost = env::var("RABBITMQ_VHOST")?;
    let queue_name = env::var("EXTRACTION_RESULTS_QUEUE")?;

    // Construct AMQP URI
    let addr = format!(
        "amqp://{}:{}@{}:{}/{}",
        user,
        pass,
        host,
        port,
        if vhost == "/" { "%2f" } else { &vhost }
    );

    // Act: Establish connection and create queue
    let conn = establish_connection(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to connect to RabbitMQ at: {addr}"));

    let channel: Channel = conn.create_channel().await?;

    // Publish a simple message to batch.extraction.results
    let payload = b"Hello, batch!";

    channel
        .basic_publish(
            "",
            queue_name.as_str(),
            BasicPublishOptions::default(),
            payload,
            BasicProperties::default(),
        )
        .await?
        .await?; // Wait for confirmation

    println!("Published message to queue: 'batch.extraction.results'");

    Ok(())
}
