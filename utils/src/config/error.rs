//! # Configuration Error Types
//!
//! This module defines error types for configuration management operations.

/// Configuration errors that can occur during environment setup and validation.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    /// Required environment variables are missing
    #[error("Missing required environment variables: {missing:?}")]
    MissingVariables { missing: Vec<String> },

    /// Invalid configuration value format
    #[error("Invalid configuration value for '{key}': {message}")]
    InvalidValue { key: String, message: String },

    /// Configuration validation failed
    #[error("Configuration validation failed: {message}")]
    ValidationFailed { message: String },
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_create_missing_variables_error_when_variables_missing() {
        // Arrange & Define
        let missing_vars = vec!["API_KEY".to_string(), "DATABASE_URL".to_string()];

        let expected_error = ConfigError::MissingVariables {
            missing: missing_vars.clone(),
        };

        // Act
        let result = ConfigError::MissingVariables {
            missing: missing_vars,
        };

        // Assert
        assert_eq!(result, expected_error);
    }

    #[test]
    fn should_create_invalid_value_error_when_value_invalid() {
        // Arrange & Define
        let expected_error = ConfigError::InvalidValue {
            key: "PORT".to_string(),
            message: "not a valid number".to_string(),
        };

        // Act
        let result = ConfigError::InvalidValue {
            key: "PORT".to_string(),
            message: "not a valid number".to_string(),
        };

        // Assert
        assert_eq!(result, expected_error);
    }

    #[test]
    fn should_create_validation_failed_error_when_validation_fails() {
        // Arrange & Define
        let expected_error = ConfigError::ValidationFailed {
            message: "Port must be between 1 and 65535".to_string(),
        };

        // Act
        let result = ConfigError::ValidationFailed {
            message: "Port must be between 1 and 65535".to_string(),
        };

        // Assert
        assert_eq!(result, expected_error);
    }
}
