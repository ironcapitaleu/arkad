use std::env;

use dotenvy::dotenv;
use lapin::Channel;

use utils::queues::{create_queue, establish_connection};

/// Main entry point for testing RabbitMQ queue connectivity using environment variables.
///
/// Loads configuration from `.env`, establishes a connection to RabbitMQ,
/// and creates the specified queue. Prints the result to stdout.
///
/// # Errors
/// Returns an error if connection or queue creation fails.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

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
    let conn = establish_connection(&addr).await.expect(format!("Failed to connect to RabbitMQ at: {addr}").as_str());
    let channel: Channel = conn.create_channel().await?;
    create_queue(&channel, &queue_name).await.expect(format!("Failed to create queue: {queue_name}").as_str());

    // Assert: Print success message
    println!("Created queue with name: '{}'", queue_name);

    Ok(())
}