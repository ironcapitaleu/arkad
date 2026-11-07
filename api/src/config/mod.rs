//! # Config
//!
//! Provides the [`Config`] of the `api` binary, read from environment variables.
//!
//! ## Modules
//!
//! - [`config_error`]: The [`ConfigError`] for a missing or invalid variable.

use std::env;
use std::ffi::{OsStr, OsString};
use std::net::{Ipv4Addr, SocketAddr};
use std::num::NonZeroU16;

pub mod config_error;

pub use config_error::ConfigError;

/// The environment variable that holds the port.
pub const PORT_ENV_VAR: &str = "PORT";

/// The environment variable that turns on the Swagger UI at `/internal/docs`.
///
/// The value `true` turns the page on. The value `false` or a missing variable turns it off.
pub const INTERNAL_DOCS_ENABLED_ENV_VAR: &str = "INTERNAL_DOCS_ENABLED";

/// Whether the router serves the Swagger UI at `/internal/docs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InternalDocs {
    /// The router serves the page.
    Enabled,
    /// The router does not serve the page, so `/internal/docs` answers `404`.
    #[default]
    Disabled,
}

/// The configuration of the `api` binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    port: NonZeroU16,
    internal_docs: InternalDocs,
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
        let internal_docs = parse_internal_docs(lookup(INTERNAL_DOCS_ENABLED_ENV_VAR).as_deref())?;
        Ok(Self {
            port,
            internal_docs,
        })
    }

    /// Returns the port the API listens on.
    #[must_use]
    pub const fn port(&self) -> NonZeroU16 {
        self.port
    }

    /// Returns whether the router serves the Swagger UI at `/internal/docs`.
    #[must_use]
    pub const fn internal_docs(&self) -> InternalDocs {
        self.internal_docs
    }

    /// Returns the address the API binds to. It covers every IPv4 interface and uses
    /// [`Config::port`].
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

fn parse_internal_docs(value: Option<&OsStr>) -> Result<InternalDocs, ConfigError> {
    match value.map(OsStr::to_str) {
        Some(Some("true")) => Ok(InternalDocs::Enabled),
        None | Some(Some("false")) => Ok(InternalDocs::Disabled),
        Some(_) => Err(ConfigError::invalid_env_var(
            INTERNAL_DOCS_ENABLED_ENV_VAR,
            "the value is neither 'true' nor 'false'",
        )),
    }
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
    fn should_fail_with_invalid_env_var_when_port_is_above_65535() {
        let lookup = lookup_port(Some("65536"));

        let expected_result = Err(ConfigError::invalid_env_var(
            "PORT",
            "number too large to fit in target type",
        ));

        let result = Config::from_lookup(lookup);

        assert_eq!(result, expected_result);
    }

    #[cfg(unix)]
    #[test]
    fn should_fail_with_invalid_env_var_when_port_is_not_utf8() {
        use std::os::unix::ffi::OsStringExt;

        let value = Some(OsString::from_vec(vec![0x38, 0xFF]));

        let expected_result = Err(ConfigError::invalid_env_var(
            "PORT",
            "the value is not valid UTF-8",
        ));

        let result = parse_port(value);

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

    #[test]
    fn should_disable_internal_docs_when_internal_docs_enabled_is_unset() {
        let value = None;

        let expected_result = Ok(InternalDocs::Disabled);

        let result = parse_internal_docs(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_enable_internal_docs_when_internal_docs_enabled_is_true() {
        let value = Some(OsStr::new("true"));

        let expected_result = Ok(InternalDocs::Enabled);

        let result = parse_internal_docs(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_disable_internal_docs_when_internal_docs_enabled_is_false() {
        let value = Some(OsStr::new("false"));

        let expected_result = Ok(InternalDocs::Disabled);

        let result = parse_internal_docs(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_env_var_when_internal_docs_enabled_is_yes() {
        let value = Some(OsStr::new("yes"));

        let expected_result = Err(ConfigError::invalid_env_var(
            "INTERNAL_DOCS_ENABLED",
            "the value is neither 'true' nor 'false'",
        ));

        let result = parse_internal_docs(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_disable_internal_docs_when_only_port_is_set() {
        let lookup = lookup_port(Some("8080"));

        let expected_result = Ok(InternalDocs::Disabled);

        let result = Config::from_lookup(lookup).map(|config| config.internal_docs());

        assert_eq!(result, expected_result);
    }
}
