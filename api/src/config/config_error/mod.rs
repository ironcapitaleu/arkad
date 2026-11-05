//! # Config Error
//!
//! Provides the [`ConfigError`] for an environment variable that is missing or holds an invalid
//! value.

use thiserror::Error;

/// Separates the kinds of failure to read the configuration of the `api` binary.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// A required environment variable is not set.
    #[error("[MissingEnvVar] Required environment variable '{name}' is not set")]
    MissingEnvVar {
        /// The name of the variable.
        name: String,
    },

    /// An environment variable is set, but its value is not usable.
    #[error(
        "[InvalidEnvVar] Environment variable '{name}' holds an invalid value, Reason: '{reason}'"
    )]
    InvalidEnvVar {
        /// The name of the variable.
        name: String,
        /// Why the value is not usable.
        reason: String,
    },
}

impl ConfigError {
    /// Creates a [`ConfigError::MissingEnvVar`] for the variable `name`.
    #[must_use]
    pub fn missing_env_var(name: impl Into<String>) -> Self {
        Self::MissingEnvVar { name: name.into() }
    }

    /// Creates a [`ConfigError::InvalidEnvVar`] for the variable `name`.
    ///
    /// `reason` must not contain the value, because a variable can hold a secret.
    #[must_use]
    pub fn invalid_env_var(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidEnvVar {
            name: name.into(),
            reason: reason.into(),
        }
    }

    /// Returns the name of the environment variable that failed.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::MissingEnvVar { name } | Self::InvalidEnvVar { name, .. } => name,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_config_error() {
        implements_auto_traits::<ConfigError>();
    }

    #[test]
    fn should_return_the_variable_name_for_missing_env_var() {
        let error = ConfigError::missing_env_var("PORT");

        let expected_result = "PORT";

        let result = error.name();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_variable_name_for_invalid_env_var() {
        let error = ConfigError::invalid_env_var("PORT", "invalid digit found in string");

        let expected_result = "PORT";

        let result = error.name();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_variable_when_displaying_missing_env_var() {
        let error = ConfigError::missing_env_var("PORT");

        let expected_result = "[MissingEnvVar] Required environment variable 'PORT' is not set";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
