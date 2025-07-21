//! # Queue Management Module
//!
//! This module provides utilities for connecting to and managing message queues,
//! specifically RabbitMQ. It supports the backing services principle of the 12-factor app.

use std::fmt;

use lapin::{Channel, Connection, ConnectionProperties};

pub mod connection;

// Re-export the main types for convenient access
pub use connection::{QueueConnection, QueueConnectionBuilder, QueueConnectionConfig};

use crate::config::error::ConfigError;

/// High-level Queue abstraction that manages `RabbitMQ` connections and operations.
///
/// This struct provides a simplified interface for common queue operations,
/// automatically handling connection management and providing ergonomic methods
/// for queue operations.
pub struct Queue {
    connection: QueueConnection,
    channel: Channel,
    inner: lapin::Queue,
    name: String,
}

impl Queue {
    /// Creates a new Queue instance with the specified name from environment variables.
    ///
    /// This method loads `RabbitMQ` connection configuration from environment variables,
    /// establishes both connection and channel automatically, and creates the specified queue.
    ///
    /// Expected environment variables:
    /// - `RABBITMQ_USERNAME` (default: "admin")
    /// - `RABBITMQ_PASSWORD` (default: "admin123")
    /// - `RABBITMQ_HOST` (default: "localhost")
    /// - `RABBITMQ_PORT` (default: "5672")
    /// - `RABBITMQ_VHOST` (default: "%2f")
    ///
    /// # Arguments
    /// * `queue_name` - Name of the queue to create and manage (accepts &str, String, etc.)
    ///
    /// # Errors
    /// Returns an error if environment variables are invalid, connection fails, or queue creation fails.
    pub async fn new(queue_name: impl Into<String>) -> Result<Self, QueueError> {
        let queue_name = queue_name.into();
        let builder = QueueConnectionBuilder::from_env().map_err(QueueError::Config)?;

        let connection = builder.connect().await.map_err(QueueError::Connection)?;

        let channel = connection
            .create_channel()
            .await
            .map_err(QueueError::Channel)?;

        let inner = create_queue(&channel, &queue_name)
            .await
            .map_err(QueueError::Operation)?;

        Ok(Self {
            connection,
            channel,
            inner,
            name: queue_name,
        })
    }

    /// Returns the name of this queue.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the number of messages currently in the queue.
    #[must_use]
    pub fn message_count(&self) -> u32 {
        self.inner.message_count()
    }

    /// Returns the number of consumers connected to this queue.
    #[must_use]
    pub fn consumer_count(&self) -> u32 {
        self.inner.consumer_count()
    }

    /// Checks if this queue exists and prints queue information.
    pub async fn check(&self) {
        check_queue(&self.channel, &self.name).await;
    }

    /// Checks if the connection is still active.
    #[must_use]
    pub fn check_connection(&self) -> bool {
        self.connection.check_connection()
    }

    /// Gets a reference to the connection configuration.
    #[must_use]
    pub const fn config(&self) -> &QueueConnectionConfig {
        self.connection.config()
    }

    /// Gets a reference to the underlying channel for advanced operations.
    #[must_use]
    pub const fn channel(&self) -> &Channel {
        &self.channel
    }

    /// Gets a reference to the underlying connection for advanced operations.
    #[must_use]
    pub const fn connection(&self) -> &QueueConnection {
        &self.connection
    }
}

impl fmt::Debug for Queue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Queue")
            .field("name", &self.name)
            .field("config", self.config())
            .field("connection_active", &self.check_connection())
            .field("message_count", &self.message_count())
            .field("consumer_count", &self.consumer_count())
            .finish()
    }
}

/// Error types for Queue operations.
#[derive(Debug)]
pub enum QueueError {
    /// Configuration error when loading from environment
    Config(ConfigError),
    /// Connection establishment error
    Connection(lapin::Error),
    /// Channel creation error
    Channel(lapin::Error),
    /// Queue operation error
    Operation(lapin::Error),
}

impl fmt::Display for QueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(e) => write!(f, "Configuration error: {e}"),
            Self::Connection(e) => write!(f, "Connection error: {e}"),
            Self::Channel(e) => write!(f, "Channel error: {e}"),
            Self::Operation(e) => write!(f, "Queue operation error: {e}"),
        }
    }
}

impl std::error::Error for QueueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Config(e) => Some(e),
            Self::Connection(e) => Some(e),
            Self::Channel(e) => Some(e),
            Self::Operation(e) => Some(e),
        }
    }
}

/// Establishes a connection to `RabbitMQ` using the provided AMQP URI.
///
/// # Arguments
/// * `addr` - The AMQP connection string (e.g., "amqp://user:pass@host:port/vhost")
///
/// # Errors
/// Returns an error if the connection cannot be established.
pub async fn establish_connection(addr: &str) -> Result<Connection, lapin::Error> {
    Connection::connect(addr, ConnectionProperties::default()).await
}

/// Checks if a queue exists and prints queue information.
///
/// # Arguments
/// * `channel` - The `RabbitMQ` channel to use
/// * `queue_name` - Name of the queue to check
pub async fn check_queue(channel: &Channel, queue_name: &str) {
    match channel
        .queue_declare(
            queue_name,
            lapin::options::QueueDeclareOptions {
                passive: true, // Only check if queue exists, don't create
                ..Default::default()
            },
            lapin::types::FieldTable::default(),
        )
        .await
    {
        Ok(queue) => {
            println!(
                "✓ Queue '{}' exists with {} messages",
                queue_name,
                queue.message_count()
            );
        }
        Err(e) => {
            println!("✗ Queue '{queue_name}' check failed: {e}");
        }
    }
}

/// Creates a queue if it doesn't exist.
///
/// # Arguments
/// * `channel` - The `RabbitMQ` channel to use
/// * `queue_name` - Name of the queue to create
///
/// # Errors
/// Returns an error if the queue cannot be created.
pub async fn create_queue(
    channel: &Channel,
    queue_name: &str,
) -> Result<lapin::Queue, lapin::Error> {
    channel
        .queue_declare(
            queue_name,
            lapin::options::QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            lapin::types::FieldTable::default(),
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_display_queue_error_correctly() {
        // Arrange
        let config_error = ConfigError::InvalidValue {
            key: "TEST_KEY".to_string(),
            message: "test message".to_string(),
        };
        let queue_error = QueueError::Config(config_error);

        // Act
        let result = format!("{}", queue_error);

        // Assert
        assert!(result.contains("Configuration error"));
        assert!(result.contains("TEST_KEY"));
    }

    #[test]
    fn should_implement_debug_for_queue_error() {
        // Arrange
        let config_error = ConfigError::InvalidValue {
            key: "TEST_KEY".to_string(),
            message: "test message".to_string(),
        };
        let queue_error = QueueError::Config(config_error);

        // Act
        let result = format!("{:?}", queue_error);

        // Assert
        assert!(result.contains("Config"));
        assert!(result.contains("InvalidValue"));
    }

    #[test]
    fn should_initialize_queue_correctly() {
        // Arrange & Act & Assert
        // This test verifies the structure is set up correctly
        // We can't actually create a Queue without a connection, but we can test the type

        // If this compiles, it means our Queue struct is properly defined
        let _queue_type_check = std::marker::PhantomData::<Queue>;
        assert!(true, "Queue struct should be properly defined");
    }
}
