use urlencoding::encode;

pub mod builder;

/// Connector struct that is used to establish a connection to RabbitMQ via AMQP protocol.
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
    #[must_use]
    pub fn user(&self) -> &str {
        &self.user
    }

    /// Gets the password for the connection.
    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Gets the host for the connection.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Gets the port for the connection.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }
    /// Gets the vhost for the connection.
    #[must_use]
    pub fn vhost(&self) -> &str {
        &self.vhost
    }

    /// Constructs the AMQP connection URI based on the connector's fields.
    /// # Returns
    /// A formatted AMQP URI string. Encodes the user, password, and vhost to ensure they are safe for use in a URI.
    /// Does not encode the port and host, as it is typically not necessary because these values are expected to contain only URL-safe characters (no spaces or special characters).
    /// # Example
    /// ```
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

    pub fn create_connection() -> super::connection::Connection{
        super::connection::Connection
    }
}
