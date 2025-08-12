use super::{Connector, ConnectorKind};

/// Marker types for tracking which fields have been set
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoUser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoPassword;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoHost;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoPort;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoVhost;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoConnectorKind;

/// Builder for [`Connector`].
///
/// This builder uses a type-safe consuming builder pattern to ensure all required fields
/// are set before building. Each setter method consumes the builder and returns
/// a new builder with updated type parameters.
///
/// # Examples
///
/// ```
/// use utils::queue::implementations::connector::{ConnectorBuilder, ConnectorKind};
///
/// let connector = ConnectorBuilder::new()
///     .user("admin")
///     .password("secret")
///     .host("localhost")
///     .port(5672u16)
///     .vhost("/")
///     .connector_kind(ConnectorKind::BatchExtractor)
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectorBuilder<U, PW, H, PO, V, CK> {
    user: U,
    password: PW,
    host: H,
    port: PO,
    vhost: V,
    connector_kind: CK,
}

impl ConnectorBuilder<NoUser, NoPassword, NoHost, NoPort, NoVhost, NoConnectorKind> {
    /// Creates a new [`ConnectorBuilder`] with no fields set.
    ///
    /// # Returns
    /// A new [`ConnectorBuilder`] instance with all fields unset.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            user: NoUser,
            password: NoPassword,
            host: NoHost,
            port: NoPort,
            vhost: NoVhost,
            connector_kind: NoConnectorKind,
        }
    }
}

impl<U, PW, H, PO, V, CK> ConnectorBuilder<U, PW, H, PO, V, CK> {
    /// Sets the user for the connection.
    ///
    /// # Arguments
    /// * `user` - The username for authentication
    ///
    /// # Returns
    /// A new builder instance with the user field set.
    #[must_use]
    pub fn user(self, user: impl Into<String>) -> ConnectorBuilder<String, PW, H, PO, V, CK> {
        ConnectorBuilder {
            user: user.into(),
            password: self.password,
            host: self.host,
            port: self.port,
            vhost: self.vhost,
            connector_kind: self.connector_kind,
        }
    }

    /// Sets the password for the connection.
    ///
    /// # Arguments
    /// * `password` - The password for authentication
    ///
    /// # Returns
    /// A new builder instance with the password field set.
    #[must_use]
    pub fn password(
        self,
        password: impl Into<String>,
    ) -> ConnectorBuilder<U, String, H, PO, V, CK> {
        ConnectorBuilder {
            user: self.user,
            password: password.into(),
            host: self.host,
            port: self.port,
            vhost: self.vhost,
            connector_kind: self.connector_kind,
        }
    }

    /// Sets the host for the connection.
    ///
    /// # Arguments
    /// * `host` - The hostname or IP address of the message broker
    ///
    /// # Returns
    /// A new builder instance with the host field set.
    #[must_use]
    pub fn host(self, host: impl Into<String>) -> ConnectorBuilder<U, PW, String, PO, V, CK> {
        ConnectorBuilder {
            user: self.user,
            password: self.password,
            host: host.into(),
            port: self.port,
            vhost: self.vhost,
            connector_kind: self.connector_kind,
        }
    }

    /// Sets the port for the connection.
    ///
    /// # Arguments
    /// * `port` - The port number for the message broker (typically 5672 for AMQP)
    ///
    /// # Returns
    /// A new builder instance with the port field set.
    #[must_use]
    pub fn port(self, port: impl Into<u16>) -> ConnectorBuilder<U, PW, H, u16, V, CK> {
        ConnectorBuilder {
            user: self.user,
            password: self.password,
            host: self.host,
            port: port.into(),
            vhost: self.vhost,
            connector_kind: self.connector_kind,
        }
    }

    /// Sets the vhost for the connection.
    ///
    /// # Arguments
    /// * `vhost` - The virtual host on the message broker.
    ///
    /// # Returns
    /// A new builder instance with the vhost field set.
    #[must_use]
    pub fn vhost(self, vhost: impl Into<String>) -> ConnectorBuilder<U, PW, H, PO, String, CK> {
        ConnectorBuilder {
            user: self.user,
            password: self.password,
            host: self.host,
            port: self.port,
            vhost: vhost.into(),
            connector_kind: self.connector_kind,
        }
    }

    /// Sets the connector kind for the connection.
    ///
    /// # Arguments
    /// * `kind` - The kind of the connector.
    ///
    /// # Returns
    /// A new builder instance with the kind field set.
    #[must_use]
    pub fn connector_kind(
        self,
        connector_kind: ConnectorKind,
    ) -> ConnectorBuilder<U, PW, H, PO, V, super::ConnectorKind> {
        ConnectorBuilder {
            user: self.user,
            password: self.password,
            host: self.host,
            port: self.port,
            vhost: self.vhost,
            connector_kind,
        }
    }
}

impl ConnectorBuilder<String, String, String, u16, String, ConnectorKind> {
    /// Builds the [`Connector`] instance from the [`ConnectorBuilder`].
    ///
    /// This method is only available when all required fields have been set.
    ///
    /// # Returns
    /// A fully configured [`Connector`] instance.
    ///
    /// # Compile-time Safety
    ///
    /// The following code will not compile because not all fields are set:
    ///
    /// ```compile_fail
    /// # use utils::queue::connector::ConnectorBuilder;
    /// let result = ConnectorBuilder::new()
    ///     .user("some_user")
    ///     .password("some_password")
    ///     .build(); // This will fail to compile since not all required fields are set
    /// ```
    #[must_use]
    pub fn build(self) -> Connector {
        Connector {
            user: self.user,
            password: self.password,
            host: self.host,
            port: self.port,
            vhost: self.vhost,
            kind: self.connector_kind,
        }
    }
}

impl Default for ConnectorBuilder<NoUser, NoPassword, NoHost, NoPort, NoVhost, NoConnectorKind> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_build_connector_when_all_fields_are_set() {
        let expected_result = Connector {
            user: "admin".to_string(),
            password: "secret".to_string(),
            host: "localhost".to_string(),
            port: 5672,
            vhost: "/".to_string(),
            kind: ConnectorKind::BatchExtractor,
        };

        let result = ConnectorBuilder::new()
            .user("admin")
            .password("secret")
            .host("localhost")
            .port(5672u16)
            .vhost("/")
            .connector_kind(ConnectorKind::BatchExtractor)
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_allow_setting_fields_in_any_order_when_using_connector_builder() {
        let expected_result = Connector {
            user: "admin".to_string(),
            password: "secret".to_string(),
            host: "localhost".to_string(),
            port: 5672,
            vhost: "/".to_string(),
            kind: ConnectorKind::BatchExtractor,
        };

        let result = ConnectorBuilder::new()
            .port(5672u16)
            .vhost("/")
            .user("admin")
            .host("localhost")
            .password("secret")
            .connector_kind(ConnectorKind::BatchExtractor)
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_generate_correct_uri_when_building_connector() {
        let connector = ConnectorBuilder::new()
            .user("admin")
            .password("secret")
            .host("localhost")
            .port(5672u16)
            .vhost("/")
            .connector_kind(ConnectorKind::BatchExtractor)
            .build();

        let expected_result = "amqp://admin:secret@localhost:5672/%2F";

        let result = connector.uri();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_generate_correct_uri_when_having_whitespace_in_username() {
        let connector = ConnectorBuilder::new()
            .user("admin user")
            .password("secret")
            .host("localhost")
            .port(5672u16)
            .vhost("/")
            .connector_kind(ConnectorKind::BatchExtractor)
            .build();

        let expected_result = "amqp://admin%20user:secret@localhost:5672/%2F";

        let result = connector.uri();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_generate_correct_uri_when_having_whitespace_in_password() {
        let connector = ConnectorBuilder::new()
            .user("admin")
            .password("secret password")
            .host("localhost")
            .port(5672u16)
            .vhost("/")
            .connector_kind(ConnectorKind::BatchExtractor)
            .build();

        let expected_result = "amqp://admin:secret%20password@localhost:5672/%2F";

        let result = connector.uri();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_default_builder_when_new_is_used_with_no_fields_set() {
        let expected_result = ConnectorBuilder::default();

        let result = ConnectorBuilder::new();

        assert_eq!(result, expected_result);
    }
}
