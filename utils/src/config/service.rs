//! # Configuration Trait
//!
//! This module defines traits for configuration-specific management and validation.

use std::{env, fmt::Debug, hash::Hash};
use crate::config::error::ConfigError;

/// Trait for configuration types that can be loaded from environment variables.
/// 
/// This trait should be implemented by configuration structs to define how they
/// load and validate their required environment variables.
///
pub trait ServiceConfig: Debug + Send + Sync + Clone + PartialEq + Eq + Hash + Sized {
    /// Returns the list of required environment variables for this configuration.
    fn required_env_vars() -> &'static [&'static str];

    /// Loads configuration from environment variables.
    ///
    /// This method provides a default implementation that checks for missing variables
    /// and delegates the actual parsing to `parse_from_env`.
    ///
    /// # Errors
    /// Returns a `ConfigError` if required environment variables are missing or invalid.
    fn load_from_env() -> Result<Self, ConfigError>
    {
        // Check for missing variables first
        let missing: Vec<String> = Self::required_env_vars()
            .iter()
            .filter(|&&var| env::var(var).is_err())
            .map(|&var| var.to_string())
            .collect();

        if !missing.is_empty() {
            return Err(ConfigError::MissingVariables { missing });
        }

        // Delegate to the specific implementation
        Self::parse_from_env()
    }

    /// Parses configuration from environment variables.
    /// 
    /// This method should be implemented by each configuration struct to handle
    /// the specific parsing logic for their fields.
    ///
    /// # Errors
    /// Returns a `ConfigError` if environment variables cannot be parsed.
    fn parse_from_env() -> Result<Self, ConfigError>;

    /// Validates the loaded configuration.
    ///
    /// # Errors
    /// Returns a `ConfigError` if the configuration values are invalid.
    fn validate(config: &Self) -> Result<(), ConfigError>;
}

/// Example configuration demonstrating the ServiceConfig trait usage.
/// 
/// This serves as a template for implementing configuration structs in the project.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MyServiceConfig {
    /// API key for external service authentication
    pub api_key: String,
    /// Database connection URL
    pub database_url: String,
    /// Service port number
    pub port: u16,
    /// Enable debug mode
    pub debug_mode: bool,
}

impl ServiceConfig for MyServiceConfig {
    fn required_env_vars() -> &'static [&'static str] {
        &["API_KEY", "DATABASE_URL", "MY_SERVICE_PORT", "MY_SERVICE_DEBUG"]
    }

    fn parse_from_env() -> Result<Self, ConfigError> {
        let api_key = env::var("API_KEY")
            .expect("API_KEY should be validated by load_from_env");

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL should be validated by load_from_env");

        let port = env::var("MY_SERVICE_PORT")
            .expect("MY_SERVICE_PORT should be validated by load_from_env")
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidValue {
                key: "MY_SERVICE_PORT".to_string(),
                message: "must be a valid port number (0-65535)".to_string(),
            })?;

        let debug_mode = env::var("MY_SERVICE_DEBUG")
            .expect("MY_SERVICE_DEBUG should be validated by load_from_env")
            .parse::<bool>()
            .map_err(|_| ConfigError::InvalidValue {
                key: "MY_SERVICE_DEBUG".to_string(),
                message: "must be 'true' or 'false'".to_string(),
            })?;

        Ok(MyServiceConfig {
            api_key,
            database_url,
            port,
            debug_mode,
        })
    }

    fn validate(config: &Self) -> Result<(), ConfigError> {
        // Validate port range
        if config.port == 0 {
            return Err(ConfigError::ValidationFailed {
                message: "Port must be greater than 0".to_string(),
            });
        }

        // Validate required fields are not empty
        if config.api_key.is_empty() {
            return Err(ConfigError::ValidationFailed {
                message: "API key cannot be empty".to_string(),
            });
        }

        if config.database_url.is_empty() {
            return Err(ConfigError::ValidationFailed {
                message: "Database URL cannot be empty".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_validate_my_service_config_successfully() {
        // Arrange
        let config = MyServiceConfig {
            api_key: "test_key".to_string(),
            database_url: "postgres://test".to_string(),
            port: 8080,
            debug_mode: false,
        };

        // Act
        let result = MyServiceConfig::validate(&config);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_return_error_when_my_service_config_invalid() {
        // Arrange
        let config = MyServiceConfig {
            api_key: "".to_string(), // Invalid: empty
            database_url: "postgres://test".to_string(),
            port: 8080,
            debug_mode: false,
        };

        // Act
        let result = MyServiceConfig::validate(&config);

        // Assert
        assert!(result.is_err());
        if let Err(ConfigError::ValidationFailed { message }) = result {
            assert!(message.contains("API key cannot be empty"));
        }
    }

    #[test]  
    fn should_return_error_when_port_is_zero() {
        // Arrange
        let config = MyServiceConfig {
            api_key: "test_key".to_string(),
            database_url: "postgres://test".to_string(),
            port: 0, // Invalid: zero
            debug_mode: false,
        };

        // Act
        let result = MyServiceConfig::validate(&config);

        // Assert
        assert!(result.is_err());
        if let Err(ConfigError::ValidationFailed { message }) = result {
            assert!(message.contains("Port must be greater than 0"));
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Clone + PartialEq + Eq + Hash>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_service_config_trait() {
        implements_auto_traits::<MyServiceConfig>();
    }

    const fn implements_send<T: Send>() {}
    #[test]
    const fn should_implement_send_when_implementing_service_config_trait() {
        implements_send::<MyServiceConfig>();
    }

    const fn implements_sync<T: Sync>() {}
    #[test]
    const fn should_implement_sync_when_implementing_service_config_trait() {
        implements_sync::<MyServiceConfig>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_service_config_trait() {
        implements_clone::<MyServiceConfig>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_service_config_trait() {
        implements_debug::<MyServiceConfig>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_service_config_trait() {
        implements_partial_eq::<MyServiceConfig>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_service_config_trait() {
        implements_eq::<MyServiceConfig>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_service_config_trait() {
        implements_hash::<MyServiceConfig>();
    }
}


