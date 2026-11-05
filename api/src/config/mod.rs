//! # Config
//!
//! Provides the [`Config`] of the `api` binary, read from environment variables.
//!
//! ## Modules
//!
//! - [`config_error`]: The [`ConfigError`] for a missing or invalid variable.

use std::env;
use std::ffi::OsString;
use std::net::{Ipv4Addr, SocketAddr};
use std::num::NonZeroU16;

pub mod config_error;

pub use config_error::ConfigError;

/// The environment variable that holds the port. Cloud Run sets it for every container.
pub const PORT_ENV_VAR: &str = "PORT";

/// The configuration of the `api` binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    port: NonZeroU16,
}

impl Config {
    /// Reads the configuration from the environment variables of the process.
    ///
    /// # Errors
    ///
    /// Returns a [`ConfigError`] if a required variable is missing or holds an invalid value.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| env::var_os(name))
    }

    fn from_lookup(lookup: impl Fn(&str) -> Option<OsString>) -> Result<Self, ConfigError> {
        let port = parse_port(lookup(PORT_ENV_VAR))?;
        Ok(Self { port })
    }

    /// Returns the port the API listens on.
    #[must_use]
    pub const fn port(&self) -> NonZeroU16 {
        self.port
    }

    /// Returns the address the API binds to. It covers every IPv4 interface and uses
    /// [`Config::port`].
    ///
    /// Cloud Run sends traffic to the container's interface, not to `localhost`.
    #[must_use]
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::from((Ipv4Addr::UNSPECIFIED, self.port.get()))
    }
}

fn parse_port(value: Option<OsString>) -> Result<NonZeroU16, ConfigError> {
    let value = value.ok_or_else(|| ConfigError::missing_env_var(PORT_ENV_VAR))?;
    let text = value.to_str().ok_or_else(|| {
        ConfigError::invalid_env_var(PORT_ENV_VAR, "the value is not valid UTF-8")
    })?;
    text.parse::<NonZeroU16>()
        .map_err(|error| ConfigError::invalid_env_var(PORT_ENV_VAR, error.to_string()))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn lookup_port(value: Option<&'static str>) -> impl Fn(&str) -> Option<OsString> {
        move |name| {
            (name == PORT_ENV_VAR)
                .then_some(value)
                .flatten()
                .map(OsString::from)
        }
    }

    #[test]
    fn should_read_the_port_when_port_is_a_valid_number() {
        let lookup = lookup_port(Some("8080"));

        let expected_result = Ok(8080);

        let result = Config::from_lookup(lookup).map(|config| config.port().get());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_missing_env_var_when_port_is_unset() {
        let lookup = lookup_port(None);

        let expected_result = Err(ConfigError::missing_env_var("PORT"));

        let result = Config::from_lookup(lookup);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_env_var_when_port_is_not_a_number() {
        let lookup = lookup_port(Some("eighty"));

        let expected_result = Err(ConfigError::invalid_env_var(
            "PORT",
            "invalid digit found in string",
        ));

        let result = Config::from_lookup(lookup);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_env_var_when_port_is_zero() {
        let lookup = lookup_port(Some("0"));

        let expected_result = Err(ConfigError::invalid_env_var(
            "PORT",
            "number would be zero for non-zero type",
        ));

        let result = Config::from_lookup(lookup);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_bind_to_every_interface_for_config() {
        let config = Config::from_lookup(lookup_port(Some("8080"))).expect(
            "Given the valid hardcoded port 8080, reading the config should always succeed",
        );

        let expected_result = SocketAddr::from(([0, 0, 0, 0], 8080));

        let result = config.socket_addr();

        assert_eq!(result, expected_result);
    }
}
