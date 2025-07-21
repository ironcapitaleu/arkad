/// Example demonstrating the simplified Queue abstraction.
///
/// This example shows how the Queue completely abstracts away lapin dependencies
/// from the public API, providing a clean and simple interface.
///
/// # Required Environment Variables
///
/// The following environment variables must be set:
/// - `RABBITMQ_USERNAME`
/// - `RABBITMQ_PASSWORD`  
/// - `RABBITMQ_HOST`
/// - `RABBITMQ_PORT`
/// - `RABBITMQ_VHOST`
///
/// # Expected Output
///
/// Queue created successfully with name: test.simplified.queue
/// Queue configuration: `QueueConnectionConfig` { ... }
/// Connection is active: true
/// Messages in queue: 0
/// Consumers: 0
use utils::queue::Queue;

#[tokio::main]
async fn main() {
    println!("=== Simplified Queue Abstraction Example ===");

    // Demonstrate different ways to pass queue names
    let queue_name_str = "test.simplified.queue";
    let queue_name_string = String::from("test.simplified.queue");

    // Using &str
    match Queue::new(queue_name_str).await {
        Ok(queue) => {
            println!("✓ Queue created successfully with name: {}", queue.name());
            println!("✓ Queue configuration: {:?}", queue.config());
            println!("✓ Connection is active: {}", queue.check_connection());
            println!("✓ Messages in queue: {}", queue.message_count());
            println!("✓ Consumers: {}", queue.consumer_count());

            // Check the queue status
            queue.check().await;

            println!("✓ Queue abstraction working perfectly!");
        }
        Err(e) => {
            eprintln!("✗ Failed to create queue with &str: {e}");
            eprintln!("  (This is expected if RabbitMQ is not running)");
        }
    }

    // Using String (would work the same way)
    println!("\n--- Alternative: Using String type ---");
    match Queue::new(queue_name_string).await {
        Ok(queue) => {
            println!(
                "✓ Queue created successfully using String type: {}",
                queue.name()
            );
        }
        Err(e) => {
            eprintln!("✗ Failed to create queue with String: {e}");
            eprintln!("  (This is expected if RabbitMQ is not running)");
        }
    }

    // Using string literal directly
    println!("\n--- Alternative: Using string literal ---");
    match Queue::new("test.literal.queue").await {
        Ok(queue) => {
            println!(
                "✓ Queue created successfully using string literal: {}",
                queue.name()
            );
        }
        Err(e) => {
            eprintln!("✗ Failed to create queue with string literal: {e}");
            eprintln!("  (This is expected if RabbitMQ is not running)");
        }
    }
}
