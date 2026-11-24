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

use reqwest::Url;

pub mod config_error;

pub use config_error::ConfigError;

/// The environment variable that holds the port.
pub const PORT_ENV_VAR: &str = "PORT";

/// The environment variable that turns on the Swagger UI at `/internal/docs`.
///
/// The value `true` turns the page on. The value `false` or a missing variable turns it off.
pub const INTERNAL_DOCS_ENABLED_ENV_VAR: &str = "INTERNAL_DOCS_ENABLED";

/// The environment variable that holds the base URL of the Supabase project, such as
/// `http://127.0.0.1:54321` for the local stack.
///
/// The API fetches the public keys for login tokens from this project. The variable is required.
/// Its scheme must be `http` or `https`, and it has no path, query, or fragment.
pub const SUPABASE_URL_ENV_VAR: &str = "SUPABASE_URL";

/// Whether the router serves the Swagger UI at `/internal/docs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InternalDocs {
    /// The router serves the page.
    Enabled,
    /// The router does not serve the page, so `/internal/docs` answers `404`.
    Disabled,
}

/// The configuration of the `api` binary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    port: NonZeroU16,
    internal_docs: InternalDocs,
    supabase_url: String,
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
        let supabase_url = parse_supabase_url(lookup(SUPABASE_URL_ENV_VAR))?;
        Ok(Self {
            port,
            internal_docs,
            supabase_url,
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

    /// Returns the base URL of the Supabase project from [`SUPABASE_URL_ENV_VAR`], without a
    /// trailing `/`.
    #[must_use]
    pub fn supabase_url(&self) -> &str {
        &self.supabase_url
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

fn parse_supabase_url(value: Option<OsString>) -> Result<String, ConfigError> {
    let value = value.ok_or_else(|| ConfigError::missing_env_var(SUPABASE_URL_ENV_VAR))?;
    let text = value.into_string().map_err(|_| {
        ConfigError::invalid_env_var(SUPABASE_URL_ENV_VAR, "the value is not valid UTF-8")
    })?;
    let url = Url::parse(&text)
        .map_err(|error| ConfigError::invalid_env_var(SUPABASE_URL_ENV_VAR, error.to_string()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(ConfigError::invalid_env_var(
            SUPABASE_URL_ENV_VAR,
            "the scheme is neither 'http' nor 'https'",
        ));
    }
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return Err(ConfigError::invalid_env_var(
            SUPABASE_URL_ENV_VAR,
            "the URL has a path, a query, or a fragment",
        ));
    }
    Ok(url.as_str().trim_end_matches('/').to_owned())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const SUPABASE_URL: &str = "http://127.0.0.1:54321";

    /// Returns a lookup that holds `port` and a valid `SUPABASE_URL`, and no other variable.
    fn lookup_port(port: Option<&'static str>) -> impl Fn(&str) -> Option<OsString> {
        lookup(port, Some(SUPABASE_URL))
    }

    /// Returns a lookup that holds `port` and `supabase_url`, and no other variable.
    fn lookup(
        port: Option<&'static str>,
        supabase_url: Option<&'static str>,
    ) -> impl Fn(&str) -> Option<OsString> {
        move |name| {
            match name {
                PORT_ENV_VAR => port,
                SUPABASE_URL_ENV_VAR => supabase_url,
                _ => None,
            }
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

    #[test]
    fn should_read_the_supabase_url_when_supabase_url_is_an_http_url() {
        let lookup = lookup(Some("8080"), Some("http://127.0.0.1:54321"));

        let expected_result = Ok("http://127.0.0.1:54321".to_owned());

        let result = Config::from_lookup(lookup).map(|config| config.supabase_url().to_owned());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_missing_env_var_when_supabase_url_is_unset() {
        let lookup = lookup(Some("8080"), None);

        let expected_result = Err(ConfigError::missing_env_var("SUPABASE_URL"));

        let result = Config::from_lookup(lookup);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_env_var_when_supabase_url_is_not_a_url() {
        let value = Some(OsString::from("supabase.local"));

        let result = parse_supabase_url(value);

        // The test leaves out the reason, which holds the text of the `url` crate.
        assert!(matches!(
            result,
            Err(ConfigError::InvalidEnvVar { name, .. }) if name == "SUPABASE_URL"
        ));
    }

    #[test]
    fn should_fail_with_invalid_env_var_when_supabase_url_has_no_http_scheme() {
        let value = Some(OsString::from("ftp://127.0.0.1:54321"));

        let expected_result = Err(ConfigError::invalid_env_var(
            "SUPABASE_URL",
            "the scheme is neither 'http' nor 'https'",
        ));

        let result = parse_supabase_url(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_the_trailing_slash_when_supabase_url_ends_with_a_slash() {
        let value = Some(OsString::from("https://project.supabase.co/"));

        let expected_result = Ok("https://project.supabase.co".to_owned());

        let result = parse_supabase_url(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_env_var_when_supabase_url_has_a_path() {
        let value = Some(OsString::from("https://project.supabase.co/auth/v1"));

        let expected_result = Err(ConfigError::invalid_env_var(
            "SUPABASE_URL",
            "the URL has a path, a query, or a fragment",
        ));

        let result = parse_supabase_url(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_env_var_when_supabase_url_has_a_query() {
        let value = Some(OsString::from("https://project.supabase.co?a=1"));

        let expected_result = Err(ConfigError::invalid_env_var(
            "SUPABASE_URL",
            "the URL has a path, a query, or a fragment",
        ));

        let result = parse_supabase_url(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_env_var_when_supabase_url_has_a_fragment() {
        let value = Some(OsString::from("https://project.supabase.co#top"));

        let expected_result = Err(ConfigError::invalid_env_var(
            "SUPABASE_URL",
            "the URL has a path, a query, or a fragment",
        ));

        let result = parse_supabase_url(value);

        assert_eq!(result, expected_result);
    }
}
