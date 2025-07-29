//! # LapinChannel Test Fixture
//!
//! This module provides a reusable test fixture for creating a `LapinChannel` instance.
//! It is designed to be used in tests that require a RabbitMQ connection.
//!
//! ## Usage
//! Import the `create_test_lapin_channel` function and use it in your tests:
//! ```rust
//! use utils::tests::common::queue::channel::fixture::create_test_lapin_channel;
//!
//! #[tokio::test]
//! async fn test_example() {
//!     let channel = create_test_lapin_channel().await.expect("Should create test channel");
//!     // Use the channel in your test
//! }
//! ```

use lapin::{Channel as LapinChannel, Connection, ConnectionProperties};

/// Creates a test `LapinChannel` for integration testing.
///
/// This function establishes a connection to RabbitMQ and creates a channel.
/// It requires RabbitMQ to be running locally.
///
/// # Errors
/// Returns an error if the connection or channel creation fails.
pub async fn create_test_lapin_channel() -> Result<LapinChannel, lapin::Error> {
    let connection = Connection::connect(
        "amqp://admin:admin123@localhost:5672",
        ConnectionProperties::default(),
    )
    .await?;

    connection.create_channel().await
}
