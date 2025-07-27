use lapin;
use urlencoding::encode;

use super::connection::Connection;
use super::error::connection_failed::ConnectionFailed;

pub mod builder;
pub use builder::ConnectorBuilder;
pub mod connector_kind;
pub use connector_kind::ConnectorKind;

/// Connector struct that is used to establish a connection to `RabbitMQ` via AMQP protocol.
///
/// This struct encapsulates all the necessary connection parameters and provides methods
/// to generate connection URIs and establish connections to the message broker.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Connector {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub vhost: String,
}

impl Connector {
    /// Gets the user for the connection.
    ///
    /// # Returns
    /// A string slice containing the username for authentication.
    #[must_use]
    pub fn user(&self) -> &str {
        &self.user
    }

    /// Gets the password for the connection.
    ///
    /// # Returns
    /// A string slice containing the password for authentication.
    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Gets the host for the connection.
    ///
    /// # Returns
    /// A string slice containing the hostname or IP address of the message broker.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Gets the port for the connection.
    ///
    /// # Returns
    /// The port number for the message broker connection.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Gets the vhost for the connection.
    ///
    /// # Returns
    /// A string slice containing the virtual host path.
    #[must_use]
    pub fn vhost(&self) -> &str {
        &self.vhost
    }

    /// Constructs the AMQP connection URI based on the connector's fields.
    ///
    /// # Returns
    /// A formatted AMQP URI string. Encodes the user, password, and vhost to ensure they are safe for use in a URI.
    /// Does not encode the port and host, as it is typically not necessary because these values are expected to contain only URL-safe characters (no spaces or special characters).
    ///
    /// # Example
    /// ```
    /// use utils::simplequeue::connector::Connector;
    ///
    /// let connector = Connector {
    ///     user: "admin".into(),
    ///     password: "secret password".into(),
    ///     host: "localhost".into(),
    ///     port: 5672,
    ///     vhost: "/".into(),
    /// };
    /// let uri = connector.uri();
    /// assert_eq!(uri, "amqp://admin:secret%20password@localhost:5672/%2F"); // encodes spaces in password and vhost
    /// println!("AMQP URI: {}", uri);
    /// // Output: AMQP URI: amqp://admin:secret%20password@localhost:5672/%2F
    /// // Note: The vhost "/" is URL-encoded to "%2F"
    /// //       Spaces in the password are encoded to "%20".
    /// //       The host is not encoded, as it is typically not necessary.
    /// //       The port is included as-is.
    /// //       This URI can be used to establish a connection to RabbitMQ.
    /// ```
    #[must_use]
    pub fn uri(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}",
            encode(&self.user),
            encode(&self.password),
            &self.host, // Host is typically not encoded in URIs
            self.port,
            encode(&self.vhost)
        )
    }

    /// Creates a connection to the message broker using the configured parameters.
    ///
    /// This method establishes an asynchronous connection to the AMQP message broker
    /// using the connection parameters stored in this `Connector` instance.
    ///
    /// # Returns
    /// A `Result` containing the `Connection` on success, or a `ConnectionFailed` error on failure.
    ///
    /// # Errors
    /// Returns `ConnectionFailed` if the connection to the message broker cannot be established.
    /// This can happen due to:
    /// - Network connectivity issues
    /// - Authentication failures (invalid credentials)
    /// - Broker unavailability or configuration issues
    /// - Invalid connection parameters
    ///
    /// # Example
    /// ```no_run
    /// use utils::simplequeue::connector::ConnectorBuilder;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = ConnectorBuilder::new()
    ///     .user("admin")
    ///     .password("secret")
    ///     .host("localhost")
    ///     .port(5672)
    ///     .vhost("/")
    ///     .build();
    ///
    /// match connector.create_connection().await {
    ///     Ok(connection) => println!("Successfully connected to message broker"),
    ///     Err(error) => eprintln!("Failed to connect: {}", error),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_connection(&self) -> Result<Connection, ConnectionFailed> {
        let uri = self.uri();
        let connection_result =
            lapin::Connection::connect(&uri, lapin::ConnectionProperties::default()).await;

        match connection_result {
            Ok(lapin_connection) => Ok(Connection::new(lapin_connection, self.clone())), // TODO: Consider using `self` and make this `create_connection()` method consume its `Connector`.
            Err(lapin_error) => Err(ConnectionFailed::from((uri.as_str(), lapin_error))),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_correct_user_when_user_is_called() {
        let connector = Connector {
            user: "admin".to_string(),
            password: "secret".to_string(),
            host: "localhost".to_string(),
            port: 5672,
            vhost: "/".to_string(),
        };

        let expected_result = "admin";

        let result = connector.user();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_password_when_password_is_called() {
        let connector = Connector {
            user: "admin".to_string(),
            password: "secret".to_string(),
            host: "localhost".to_string(),
            port: 5672,
            vhost: "/".to_string(),
        };

        let expected_result = "secret";

        let result = connector.password();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_host_when_host_is_called() {
        let connector = Connector {
            user: "admin".to_string(),
            password: "secret".to_string(),
            host: "localhost".to_string(),
            port: 5672,
            vhost: "/".to_string(),
        };

        let expected_result = "localhost";

        let result = connector.host();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_port_when_port_is_called() {
        let connector = Connector {
            user: "admin".to_string(),
            password: "secret".to_string(),
            host: "localhost".to_string(),
            port: 5672,
            vhost: "/".to_string(),
        };

        let expected_result = 5672;

        let result = connector.port();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_vhost_when_vhost_is_called() {
        let connector = Connector {
            user: "admin".to_string(),
            password: "secret".to_string(),
            host: "localhost".to_string(),
            port: 5672,
            vhost: "/".to_string(),
        };

        let expected_result = "/";

        let result = connector.vhost();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_generate_correct_uri_when_uri_is_called_with_basic_credentials() {
        let connector = Connector {
            user: "admin".to_string(),
            password: "secret".to_string(),
            host: "localhost".to_string(),
            port: 5672,
            vhost: "/".to_string(),
        };

        let expected_result = "amqp://admin:secret@localhost:5672/%2F";

        let result = connector.uri();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_url_encode_special_characters_when_uri_is_called_with_complex_credentials() {
        let connector = Connector {
            user: "admin user".to_string(),
            password: "secret password".to_string(),
            host: "localhost".to_string(),
            port: 5672,
            vhost: "/my vhost".to_string(),
        };

        let expected_result = "amqp://admin%20user:secret%20password@localhost:5672/%2Fmy%20vhost";

        let result = connector.uri();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_handle_non_standard_port_when_uri_is_called() {
        let connector = Connector {
            user: "admin".to_string(),
            password: "secret".to_string(),
            host: "rabbitmq.example.com".to_string(),
            port: 15672,
            vhost: "/production".to_string(),
        };

        let expected_result = "amqp://admin:secret@rabbitmq.example.com:15672/%2Fproduction";

        let result = connector.uri();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_handle_empty_vhost_when_uri_is_called() {
        let connector = Connector {
            user: "admin".to_string(),
            password: "secret".to_string(),
            host: "localhost".to_string(),
            port: 5672,
            vhost: "".to_string(),
        };

        let expected_result = "amqp://admin:secret@localhost:5672/";

        let result = connector.uri();

        assert_eq!(result, expected_result);
    }
}
