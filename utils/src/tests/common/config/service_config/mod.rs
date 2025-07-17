//! # Sample Service Configuration Fixture
//!
//! This module provides a complete example of how to implement service-specific configuration
//! using the ServiceConfig trait. It serves as a template for other services in the project.

use std::env;

use crate::config::{service::ServiceConfig, error::ConfigError};

/// Configuration for a sample service demonstrating the configuration pattern.
/// 
/// This struct shows how to organize service-specific settings and provides
/// a template for implementing other service configurations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SampleServiceConfig {
    /// API key for external service authentication
    pub api_key: String,
    /// Database connection URL
    pub database_url: String,
    /// Service hostname
    pub host: String,
    /// Service port number
    pub port: u16,
    /// Enable/disable debug mode
    pub debug_mode: bool,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
}

impl SampleServiceConfig {
    /// Constructs a complete service URL from host and port.
    #[must_use]
    pub fn service_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Returns whether the service is running in debug mode.
    #[must_use]
    pub fn is_debug_enabled(&self) -> bool {
        self.debug_mode
    }

    /// Returns the timeout duration as a `std::time::Duration`.
    #[must_use]
    pub fn timeout_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds)
    }
}

impl ServiceConfig for SampleServiceConfig {

    fn required_env_vars() -> &'static [&'static str] {
        &[
            "API_KEY",
            "DATABASE_URL", 
            "SAMPLE_SERVICE_HOST",
            "SAMPLE_SERVICE_PORT",
            "SAMPLE_SERVICE_DEBUG",
            "SAMPLE_SERVICE_MAX_RETRIES",
            "SAMPLE_SERVICE_TIMEOUT_SECONDS",
        ]
    }

    fn parse_from_env() -> Result<Self, ConfigError> {
        // Parse port number
        let port_str = env::var("SAMPLE_SERVICE_PORT")
            .expect("SAMPLE_SERVICE_PORT should be validated above");
        let port = port_str.parse::<u16>()
            .map_err(|_| ConfigError::InvalidValue {
                key: "SAMPLE_SERVICE_PORT".to_string(),
                message: "must be a valid port number (0-65535)".to_string(),
            })?;

        // Parse debug mode
        let debug_str = env::var("SAMPLE_SERVICE_DEBUG")
            .expect("SAMPLE_SERVICE_DEBUG should be validated above");
        let debug_mode = debug_str.parse::<bool>()
            .map_err(|_| ConfigError::InvalidValue {
                key: "SAMPLE_SERVICE_DEBUG".to_string(),
                message: "must be 'true' or 'false'".to_string(),
            })?;

        // Parse max retries
        let max_retries_str = env::var("SAMPLE_SERVICE_MAX_RETRIES")
            .expect("SAMPLE_SERVICE_MAX_RETRIES should be validated above");
        let max_retries = max_retries_str.parse::<u32>()
            .map_err(|_| ConfigError::InvalidValue {
                key: "SAMPLE_SERVICE_MAX_RETRIES".to_string(),
                message: "must be a valid positive number".to_string(),
            })?;

        // Parse timeout seconds
        let timeout_str = env::var("SAMPLE_SERVICE_TIMEOUT_SECONDS")
            .expect("SAMPLE_SERVICE_TIMEOUT_SECONDS should be validated above");
        let timeout_seconds = timeout_str.parse::<u64>()
            .map_err(|_| ConfigError::InvalidValue {
                key: "SAMPLE_SERVICE_TIMEOUT_SECONDS".to_string(),
                message: "must be a valid positive number".to_string(),
            })?;

        Ok(SampleServiceConfig {
            api_key: env::var("API_KEY").expect("Should be validated"),
            database_url: env::var("DATABASE_URL").expect("Should be validated"),
            host: env::var("SAMPLE_SERVICE_HOST").expect("Should be validated"),
            port,
            debug_mode,
            max_retries,
            timeout_seconds,
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
        if config.api_key.is_empty() 
            || config.database_url.is_empty() 
            || config.host.is_empty() {
            return Err(ConfigError::ValidationFailed {
                message: "One or more configuration values are empty".to_string(),
            });
        }

        // Validate max retries is reasonable
        if config.max_retries > 100 {
            return Err(ConfigError::ValidationFailed {
                message: "Max retries must be <= 100".to_string(),
            });
        }

        // Validate timeout is reasonable (not too short or too long)
        if config.timeout_seconds == 0 || config.timeout_seconds > 3600 {
            return Err(ConfigError::ValidationFailed {
                message: "Timeout must be between 1 and 3600 seconds".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use dotenvy;
    use pretty_assertions::assert_eq;
    use serial_test::serial;

    use super::*;

    /// Creates a test .env file with sample service configuration.
    fn create_sample_env_content() -> &'static str {
        r#"
# Core configuration (shared)
API_KEY=test_api_key_12345
DATABASE_URL=postgres://test_user:test_pass@localhost:5432/test_db

# Sample service specific configuration
SAMPLE_SERVICE_HOST=localhost
SAMPLE_SERVICE_PORT=8080
SAMPLE_SERVICE_DEBUG=true
SAMPLE_SERVICE_MAX_RETRIES=3
SAMPLE_SERVICE_TIMEOUT_SECONDS=30
"#
    }

    #[test]
    #[serial]
    fn should_load_sample_service_config_when_all_vars_set() {
        // Arrange: Create temporary .env file for testing
        let temp_dir = std::env::temp_dir();
        let env_file_path = temp_dir.join("sample_service_test.env");
        std::fs::write(&env_file_path, create_sample_env_content())
            .expect("Should be able to write test .env file");

        dotenvy::from_path(&env_file_path).expect("Test .env file should load");

        // Define
        let expected_result = SampleServiceConfig {
            api_key: "test_api_key_12345".to_string(),
            database_url: "postgres://test_user:test_pass@localhost:5432/test_db".to_string(),
            host: "localhost".to_string(),
            port: 8080,
            debug_mode: true,
            max_retries: 3,
            timeout_seconds: 30,
        };

        // Act
        let result = SampleServiceConfig::load_from_env().unwrap();

        // Assert
        assert_eq!(result, expected_result);

        // Cleanup
        std::fs::remove_file(&env_file_path).ok();
    }

    #[test]
    #[serial]
    fn should_validate_successfully_when_config_is_valid() {
        // Arrange: Create temporary .env file for testing
        let temp_dir = std::env::temp_dir();
        let env_file_path = temp_dir.join("sample_service_validate_test.env");
        std::fs::write(&env_file_path, create_sample_env_content())
            .expect("Should be able to write test .env file");

        dotenvy::from_path(&env_file_path).expect("Test .env file should load");

        let config = SampleServiceConfig::load_from_env().unwrap();

        // Act
        let result = SampleServiceConfig::validate(&config);

        // Assert
        assert!(result.is_ok());

        // Cleanup
        std::fs::remove_file(&env_file_path).ok();
    }

    #[test]
    fn should_generate_correct_service_url_when_using_sample_config() {
        // Arrange
        let config = SampleServiceConfig {
            api_key: "test_key".to_string(),
            database_url: "postgres://test".to_string(),
            host: "example.com".to_string(),
            port: 9000,
            debug_mode: false,
            max_retries: 5,
            timeout_seconds: 60,
        };

        // Define
        let expected_result = "http://example.com:9000";

        // Act
        let result = config.service_url();

        // Assert
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_debug_status_when_debug_enabled() {
        // Arrange
        let config = SampleServiceConfig {
            api_key: "test_key".to_string(),
            database_url: "postgres://test".to_string(),
            host: "localhost".to_string(),
            port: 8080,
            debug_mode: true,
            max_retries: 3,
            timeout_seconds: 30,
        };

        // Act & Assert
        assert!(config.is_debug_enabled());
    }

    #[test]
    fn should_return_correct_timeout_duration_when_configured() {
        // Arrange
        let config = SampleServiceConfig {
            api_key: "test_key".to_string(),
            database_url: "postgres://test".to_string(),
            host: "localhost".to_string(),
            port: 8080,
            debug_mode: false,
            max_retries: 3,
            timeout_seconds: 45,
        };

        // Define
        let expected_result = Duration::from_secs(45);

        // Act
        let result = config.timeout_duration();

        // Assert
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_port_is_zero() {
        // Arrange
        let config = SampleServiceConfig {
            api_key: "test_key".to_string(),
            database_url: "postgres://test".to_string(),
            host: "localhost".to_string(),
            port: 0, // Invalid port
            debug_mode: false,
            max_retries: 3,
            timeout_seconds: 30,
        };

        // Act
        let result = SampleServiceConfig::validate(&config);

        // Assert
        assert!(result.is_err());
        if let Err(ConfigError::ValidationFailed { message }) = result {
            assert!(message.contains("Port must be greater than 0"));
        }
    }

    #[test]
    fn should_return_error_when_max_retries_too_high() {
        // Arrange
        let config = SampleServiceConfig {
            api_key: "test_key".to_string(),
            database_url: "postgres://test".to_string(),
            host: "localhost".to_string(),
            port: 8080,
            debug_mode: false,
            max_retries: 150, // Too high
            timeout_seconds: 30,
        };

        // Act
        let result = SampleServiceConfig::validate(&config);

        // Assert
        assert!(result.is_err());
        if let Err(ConfigError::ValidationFailed { message }) = result {
            assert!(message.contains("Max retries must be <= 100"));
        }
    }

    #[test]
    fn should_return_error_when_timeout_too_long() {
        // Arrange
        let config = SampleServiceConfig {
            api_key: "test_key".to_string(),
            database_url: "postgres://test".to_string(),
            host: "localhost".to_string(),
            port: 8080,
            debug_mode: false,
            max_retries: 3,
            timeout_seconds: 7200, // Too long (2 hours)
        };

        // Act
        let result = SampleServiceConfig::validate(&config);

        // Assert
        assert!(result.is_err());
        if let Err(ConfigError::ValidationFailed { message }) = result {
            assert!(message.contains("Timeout must be between 1 and 3600 seconds"));
        }
    }
}
