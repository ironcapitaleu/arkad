//! # Queue Example
//!
//! This example demonstrates how to use the high-level Queue abstraction
//! to simplify RabbitMQ operations.

use utils::queue::Queue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Queue Abstraction Example");
    println!("==============================");

    // Example 1: Creating a Queue from environment variables
    println!("\n1️⃣ Creating Queue from environment variables:");
    match Queue::new().await {
        Ok(queue) => {
            println!("   ✅ Queue created successfully!");
            println!("   Configuration: {:?}", queue.config());
            
            // Check connection
            let is_connected = queue.check_connection();
            println!("   Connection active: {}", is_connected);

            // Example: Create a test queue
            println!("\n   Creating test queue...");
            match queue.create_queue("test.example.queue").await {
                Ok(q) => {
                    println!("   ✅ Queue '{}' created with {} messages", q.name(), q.message_count());
                }
                Err(e) => {
                    println!("   ❌ Queue creation failed: {}", e);
                }
            }

            // Example: Check if queue exists
            println!("\n   Checking queue...");
            queue.check_queue("test.example.queue").await;
        }
        Err(e) => {
            println!("   ❌ Queue creation failed (expected without RabbitMQ): {}", e);
        }
    }

    // Example 2: Creating a Queue with custom configuration
    println!("\n2️⃣ Creating Queue with custom configuration:");
    match Queue::with_config("admin", "admin123", "localhost", 5672, "%2f").await {
        Ok(queue) => {
            println!("   ✅ Custom Queue created successfully!");
            println!("   Configuration: {:?}", queue.config());
        }
        Err(e) => {
            println!("   ❌ Custom Queue creation failed (expected without RabbitMQ): {}", e);
        }
    }

    println!("\n✨ Example completed!");
    println!("\n💡 To test with real RabbitMQ:");
    println!("   docker compose -f docker/batch-etl/services/queues/rabbitmq.yaml up -d");
    
    Ok(())
}
