//! # Queue Connection Module
//!
//! This module provides a builder pattern for creating `RabbitMQ` connections
//! with ergonomic configuration support.

use std::fmt;

use lapin::{Channel, Connection, ConnectionProperties};

use crate::config::error::ConfigError;

/// Configuration for a `RabbitMQ` queue connection.
///
/// This struct holds all the necessary parameters to establish a connection
/// to a `RabbitMQ` instance with sensible defaults (admin:admin123@localhost:5672/%2f).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueueConnectionConfig {
    /// The username for authentication
    pub username: String,
    /// The password for authentication
    pub password: String,
    /// The hostname or IP address of the `RabbitMQ` server
    pub host: String,
    /// The port number for the `RabbitMQ` server
    pub port: u16,
    /// The virtual host to connect to
    pub vhost: String,
}

impl QueueConnectionConfig {
    /// Creates a new queue connection configuration.
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        vhost: impl Into<String>,
    ) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            host: host.into(),
            port,
            vhost: vhost.into(),
        }
    }

    /// Builds the AMQP URI from the configuration.
    #[must_use]
    pub fn build_uri(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.vhost
        )
    }
}

impl Default for QueueConnectionConfig {
    fn default() -> Self {
        Self {
            username: "admin".to_string(),
            password: "admin123".to_string(),
            host: "localhost".to_string(),
            port: 5672,
            vhost: "%2f".to_string(), // URL-encoded "/"
        }
    }
}

/// Builder for creating queue connections with ergonomic configuration.
///
/// This builder allows for fluent configuration of `RabbitMQ` connection parameters
/// and provides methods to establish connections and create channels.
#[derive(Debug, Clone)]
pub struct QueueConnectionBuilder {
    config: QueueConnectionConfig,
}

impl QueueConnectionBuilder {
    /// Creates a new queue connection builder with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: QueueConnectionConfig::default(),
        }
    }

    /// Creates a new queue connection builder from environment variables.
    ///
    /// Expected environment variables:
    /// - `RABBITMQ_USERNAME` (default: "admin")
    /// - `RABBITMQ_PASSWORD` (default: "admin123")
    /// - `RABBITMQ_HOST` (default: "localhost")
    /// - `RABBITMQ_PORT` (default: "5672")
    /// - `RABBITMQ_VHOST` (default: "%2f")
    ///
    /// # Errors
    /// Returns a `ConfigError` if any required environment variable is invalid.
    pub fn from_env() -> Result<Self, ConfigError> {
        let username = std::env::var("RABBITMQ_USERNAME").unwrap_or_else(|_| "admin".to_string());

        let password =
            std::env::var("RABBITMQ_PASSWORD").unwrap_or_else(|_| "admin123".to_string());

        let host = std::env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string());

        let port = std::env::var("RABBITMQ_PORT")
            .unwrap_or_else(|_| "5672".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidValue {
                key: "RABBITMQ_PORT".to_string(),
                message: "must be a valid port number (0-65535)".to_string(),
            })?;

        let vhost = std::env::var("RABBITMQ_VHOST").unwrap_or_else(|_| "%2f".to_string());

        Ok(Self {
            config: QueueConnectionConfig::new(username, password, host, port, vhost),
        })
    }

    /// Sets the username for authentication.
    #[must_use]
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.config.username = username.into();
        self
    }

    /// Sets the password for authentication.
    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.config.password = password.into();
        self
    }

    /// Sets the hostname or IP address.
    #[must_use]
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }

    /// Sets the port number.
    #[must_use]
    pub const fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// Sets the virtual host.
    #[must_use]
    pub fn vhost(mut self, vhost: impl Into<String>) -> Self {
        self.config.vhost = vhost.into();
        self
    }

    /// Builds and returns the queue connection.
    ///
    /// # Errors
    /// Returns an error if the connection cannot be established.
    pub async fn connect(self) -> Result<QueueConnection, lapin::Error> {
        let uri = self.config.build_uri();
        let connection = Connection::connect(&uri, ConnectionProperties::default()).await?;

        Ok(QueueConnection {
            connection,
            config: self.config,
        })
    }

    /// Gets a reference to the current configuration.
    #[must_use]
    pub const fn config(&self) -> &QueueConnectionConfig {
        &self.config
    }
}

impl Default for QueueConnectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A `RabbitMQ` connection wrapper with additional functionality.
///
/// This struct provides methods for managing the connection, creating channels,
/// and accessing configuration information.
pub struct QueueConnection {
    connection: Connection,
    config: QueueConnectionConfig,
}

impl QueueConnection {
    /// Creates a new channel for the connection.
    ///
    /// # Errors
    /// Returns an error if the channel cannot be created.
    pub async fn create_channel(&self) -> Result<Channel, lapin::Error> {
        self.connection.create_channel().await
    }

    /// Checks if the connection is still active.
    #[must_use]
    pub fn check_connection(&self) -> bool {
        self.connection.status().connected()
    }

    /// Gets a reference to the connection configuration.
    #[must_use]
    pub const fn config(&self) -> &QueueConnectionConfig {
        &self.config
    }

    /// Gets the underlying lapin connection.
    #[must_use]
    pub const fn inner(&self) -> &Connection {
        &self.connection
    }
}

impl fmt::Debug for QueueConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueueConnection")
            .field("config", &self.config)
            .field("status", &self.connection.status())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_default_queue_connection_config() {
        // Arrange & Act
        let config = QueueConnectionConfig::default();

        // Assert
        assert_eq!(config.username, "admin");
        assert_eq!(config.password, "admin123");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5672);
        assert_eq!(config.vhost, "%2f");
    }

    #[test]
    fn should_build_uri_from_config() {
        // Arrange
        let config =
            QueueConnectionConfig::new("test_user", "test_pass", "test_host", 5673, "test_vhost");

        // Define
        let expected_result = "amqp://test_user:test_pass@test_host:5673/test_vhost";

        // Act
        let result = config.build_uri();

        // Assert
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_builder_with_fluent_interface() {
        // Arrange & Act
        let builder = QueueConnectionBuilder::new()
            .username("custom_user")
            .password("custom_pass")
            .host("custom_host")
            .port(1234_u16)
            .vhost("custom_vhost");

        // Assert
        let config = builder.config();
        assert_eq!(config.username, "custom_user");
        assert_eq!(config.password, "custom_pass");
        assert_eq!(config.host, "custom_host");
        assert_eq!(config.port, 1234);
        assert_eq!(config.vhost, "custom_vhost");
    }

    #[test]
    fn should_accept_u16_for_port() {
        // Arrange & Act
        let builder = QueueConnectionBuilder::new().port(8080_u16);

        // Assert
        assert_eq!(builder.config().port, 8080);
    }

    #[test]
    fn should_accept_string_types_for_text_fields() {
        // Arrange
        let string_val = "test_string".to_string();
        let str_val = "test_str";

        // Act
        let builder = QueueConnectionBuilder::new()
            .username(string_val.clone())
            .password(str_val)
            .host("test_host")
            .vhost(&string_val);

        // Assert
        let config = builder.config();
        assert_eq!(config.username, string_val);
        assert_eq!(config.password, str_val);
        assert_eq!(config.host, "test_host");
        assert_eq!(config.vhost, string_val);
    }

    #[test]
    fn should_create_builder_from_env_with_defaults_when_vars_missing() {
        // Act
        let result = QueueConnectionBuilder::from_env();

        // Assert
        assert!(result.is_ok());
        let builder = result.unwrap();
        let config = builder.config();
        assert_eq!(config.username, "admin");
        assert_eq!(config.password, "admin123");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5672);
        assert_eq!(config.vhost, "%2f");
    }
}
