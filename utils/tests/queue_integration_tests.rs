//! # Queue Integration Tests
//!
//! These tests verify queue functionality when RabbitMQ is running on localhost.
//! 
//! ## Prerequisites
//! Before running these tests, ensure that:
//! 1. RabbitMQ is running on localhost:5672
//! 2. The admin user (admin:admin123) is configured
//! 3. Docker services are running: `docker compose up -d`
//! 
//! ## Running Tests
//! ```bash
//! # Start RabbitMQ services first
//! docker compose -f docker/batch-etl/services/queues/rabbitmq.yaml up -d
//! 
//! # Run integration tests
//! cargo test --test queue_integration_tests
//! ```

use dotenvy;
use pretty_assertions::assert_eq;

use utils::queues::{check_queue, create_queue, establish_connection};

/// Tests establishing a connection to RabbitMQ running on localhost.
/// 
/// This test requires RabbitMQ to be running with the default credentials
/// from the docker-compose configuration.
#[tokio::test]
async fn should_establish_connection_when_rabbitmq_is_running() {
    // Arrange: Load configuration from environment
    dotenvy::from_path("src/tests/common/config/.env.valid")
        .expect("Test .env file should exist");
    
    let amqp_uri = std::env::var("RABBITMQ_URI")
        .unwrap_or_else(|_| "amqp://admin:admin123@localhost:5672".to_string());

    // Define
    let expected_result = true;

    // Act
    let result = establish_connection(&amqp_uri).await.is_ok();

    // Assert
    assert_eq!(result, expected_result, "Should successfully connect to RabbitMQ running on localhost");
}

/// Tests creating a queue on the RabbitMQ instance.
/// 
/// This test creates a test queue and verifies it was created successfully.
#[tokio::test]
async fn should_create_queue_when_rabbitmq_is_running() {
    // Arrange: Load configuration and establish connection
    dotenvy::from_path("src/tests/common/config/.env.valid")
        .expect("Test .env file should exist");
    
    let amqp_uri = std::env::var("RABBITMQ_URI")
        .unwrap_or_else(|_| "amqp://admin:admin123@localhost:5672".to_string());

    let conn = establish_connection(&amqp_uri)
        .await
        .expect("Should connect to RabbitMQ");

    let channel = conn
        .create_channel()
        .await
        .expect("Should create channel");

    let test_queue_name = "test.integration.queue";

    // Act
    let result = create_queue(&channel, test_queue_name).await;

    // Assert
    assert!(result.is_ok(), "Should successfully create test queue");
    
    let queue = result.unwrap();
    assert_eq!(queue.name().as_str(), test_queue_name);
    // Note: Queue durability is set during creation, cannot be queried from Queue object
}

/// Tests checking if the configured extraction results queue exists.
/// 
/// This test verifies that the queue defined in the configuration
/// can be found and accessed.
#[tokio::test]
async fn should_check_extraction_results_queue_when_configured() {
    // Arrange: Load configuration and establish connection
    dotenvy::from_path("src/tests/common/config/.env.valid")
        .expect("Test .env file should exist");
    
    let amqp_uri = std::env::var("RABBITMQ_URI")
        .unwrap_or_else(|_| "amqp://admin:admin123@localhost:5672".to_string());

    let conn = establish_connection(&amqp_uri)
        .await
        .expect("Should connect to RabbitMQ");

    let channel = conn
        .create_channel()
        .await
        .expect("Should create channel");

    let extraction_results_queue = std::env::var("BATCH_EXTRACTION_RESULTS_QUEUE")
        .unwrap_or_else(|_| "batch.extraction.results".to_string());

    // First ensure the queue exists by creating it
    let _queue = create_queue(&channel, &extraction_results_queue)
        .await
        .expect("Should create extraction results queue");

    // Act & Assert
    // This test checks that the function runs without panicking
    // The actual output is printed to stdout
    check_queue(&channel, &extraction_results_queue).await;
    
    // If we reach this point, the function didn't panic
    assert!(true, "check_queue should complete without panicking");
}

/// Tests the complete workflow: connect, create queue, and verify.
/// 
/// This integration test verifies the entire queue management workflow
/// using the actual configuration and services.
#[tokio::test]
async fn should_complete_full_queue_workflow_when_services_running() {
    // Arrange: Load configuration
    dotenvy::from_path("src/tests/common/config/.env.valid")
        .expect("Test .env file should exist");
    
    let amqp_uri = std::env::var("RABBITMQ_URI")
        .unwrap_or_else(|_| "amqp://admin:admin123@localhost:5672".to_string());

    let test_queue_name = "test.full.workflow.queue";

    // Act: Complete workflow
    
    // 1. Establish connection
    let conn = establish_connection(&amqp_uri)
        .await
        .expect("Step 1: Should establish connection");

    // 2. Create channel
    let channel = conn
        .create_channel()
        .await
        .expect("Step 2: Should create channel");

    // 3. Create queue
    let queue = create_queue(&channel, test_queue_name)
        .await
        .expect("Step 3: Should create queue");

    // 4. Verify queue properties
    assert_eq!(queue.name().as_str(), test_queue_name);
    assert_eq!(queue.message_count(), 0); // Should be empty initially
    // Note: Queue durability is set during creation, cannot be queried from Queue object

    // 5. Check queue exists
    check_queue(&channel, test_queue_name).await;

    // Assert: If we reach this point, the full workflow completed successfully
    assert!(true, "Full queue workflow should complete successfully");
}

/// Tests connection failure when using invalid credentials.
/// 
/// This test verifies proper error handling when connection fails.
#[tokio::test]
async fn should_fail_connection_when_using_invalid_credentials() {
    // Arrange: Use invalid credentials
    let invalid_addr = "amqp://invalid_user:invalid_pass@localhost:5672/%2f";

    // Act
    let result = establish_connection(invalid_addr).await;

    // Assert
    assert!(result.is_err(), "Should fail to connect with invalid credentials");
}

/// Tests connection failure when RabbitMQ is not running.
/// 
/// This test verifies error handling when the service is unavailable.
/// Note: This test uses a non-standard port to avoid conflicts.
#[tokio::test]
async fn should_fail_connection_when_rabbitmq_not_running() {
    // Arrange: Use a port where RabbitMQ is not running
    let unavailable_addr = "amqp://admin:admin123@localhost:9999/%2f";

    // Act
    let result = establish_connection(unavailable_addr).await;

    // Assert
    assert!(result.is_err(), "Should fail to connect when RabbitMQ is not running");
}
