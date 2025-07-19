use super::Connector;

/// Marker types for tracking which fields have been set
pub struct NoUser;
pub struct NoPassword;
pub struct NoHost;
pub struct NoPort;
pub struct NoVhost;

/// Builder for [`Connector`].
///
/// This builder uses a type-safe consuming builder pattern to ensure all required fields
/// are set before building. Each setter method consumes the builder and returns
/// a new builder with updated type parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectorBuilder<U, PW, H, PO, V> {
    user: U,
    password: PW,
    host: H,
    port: PO,
    vhost: V,
}

impl ConnectorBuilder<NoUser, NoPassword, NoHost, NoPort, NoVhost> {
    /// Creates a new connection builder with no fields set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            user: NoUser,
            password: NoPassword,
            host: NoHost,
            port: NoPort,
            vhost: NoVhost,
        }
    }
}

impl<U, PW, H, PO, V> ConnectorBuilder<U, PW, H, PO, V> {
    /// Sets the user for the connection.
    #[must_use]
    pub fn user(self, user: impl Into<String>) -> ConnectorBuilder<String, PW, H, PO, V> {
        ConnectorBuilder {
            user: user.into(),
            password: self.password,
            host: self.host,
            port: self.port,
            vhost: self.vhost,
        }
    }

    /// Sets the password for the connection.
    #[must_use]
    pub fn password(self, password: impl Into<String>) -> ConnectorBuilder<U, String, H, PO, V> {
        ConnectorBuilder {
            user: self.user,
            password: password.into(),
            host: self.host,
            port: self.port,
            vhost: self.vhost,
        }
    }

    /// Sets the host for the connection.
    #[must_use]
    pub fn host(self, host: impl Into<String>) -> ConnectorBuilder<U, PW, String, PO, V> {
        ConnectorBuilder {
            user: self.user,
            password: self.password,
            host: host.into(),
            port: self.port,
            vhost: self.vhost,
        }
    }

    /// Sets the port for the connection.
    #[must_use]
    pub fn port(self, port: impl Into<u16>) -> ConnectorBuilder<U, PW, H, u16, V> {
        ConnectorBuilder {
            user: self.user,
            password: self.password,
            host: self.host,
            port: port.into(),
            vhost: self.vhost,
        }
    }

    /// Sets the vhost for the connection.
    #[must_use]
    pub fn vhost(self, vhost: impl Into<String>) -> ConnectorBuilder<U, PW, H, PO, String> {
        ConnectorBuilder {
            user: self.user,
            password: self.password,
            host: self.host,
            port: self.port,
            vhost: vhost.into(),
        }
    }
}

impl ConnectorBuilder<String, String, String, u16, String> {
    /// Builds the connection instance from the builder.
    ///
    /// This method is only available when all required fields have been set.
    /// # Compile-time Safety
    ///
    /// The following code will not compile because not all fields are set:
    ///
    /// ```compile_fail
    /// # use utils::simplequeue::connection::conn2::ConnectorBuilder;
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
        }
    }
}

impl Default for ConnectorBuilder<NoUser, NoPassword, NoHost, NoPort, NoVhost> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_build_connector_when_all_fields_are_set() {
        let expected_result = Connector {
            user: "admin".to_string(),
            password: "secret".to_string(),
            host: "localhost".to_string(),
            port: 5672,
            vhost: "/".to_string(),
        };

        let result = ConnectorBuilder::new()
            .user("admin")
            .password("secret")
            .host("localhost")
            .port(5672u16)
            .vhost("/")
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
        };

        let result = ConnectorBuilder::new()
            .port(5672u16)
            .vhost("/")
            .user("admin")
            .host("localhost")
            .password("secret")
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
            .build();

        let expected_result = "amqp://admin:secret%20password@localhost:5672/%2F";

        let result = connector.uri();

        assert_eq!(result, expected_result);
    }
}
