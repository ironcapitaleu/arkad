// src/constants.rs
use std::env;

/// List of environment variables required by the application.
/// Update this slice when adding or removing required variables.
pub const REQUIRED_ENV_VARS: &[&str] = &["API_KEY", "DATABASE_URL"];

/// Returns the value of DATABASE_URL, or panics if it's not set.
pub fn database_url() -> String {
    env::var("DATABASE_URL").expect("Required environment variable `DATABASE_URL` must be set.")
}

/// Returns the value of API_KEY, or panics if it's not set.
pub fn api_key() -> String {
    env::var("API_KEY").expect("Required environment variable `API_KEY` must be set.")
}

/// Validates that all required environment variables are present.
/// Returns Ok(()) if none are missing, or Err(Vec<String>) listing missing ones.
pub fn validate_env_vars() -> Result<(), Vec<String>> {
    let missing: Vec<String> = REQUIRED_ENV_VARS
        .iter()
        .filter(|&&var| env::var(var).is_err())
        .map(|&var| var.to_string())
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use dotenvy;
    use pretty_assertions::{assert_eq, assert_ne};
    use serial_test::serial;

    use super::*;

    #[test]
    fn should_return_required_env_vars_list_when_called() {
        let expected_result = ["API_KEY", "DATABASE_URL"];

        let result = REQUIRED_ENV_VARS;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_miss_comparison_when_compared_with_list_that_includes_unexpected_env_vars() {
        let unexpected_result = ["DATABASE_URL", "API_KEY", "SOME_UNEXPECTED_ENV_VAR"];

        let result = REQUIRED_ENV_VARS;

        assert_ne!(result, unexpected_result);
    }

    #[test]
    #[serial]
    fn should_return_ok_when_all_required_env_vars_are_set() {
        // 'cwd' should be the cwd based on the sub-project's root directory, i.e., where the Cargo.toml of this package is located.
        let cwd = env::current_dir().expect("Should alwazs be able to get currrent working directory.");
        let relative_path = Path::new("src/tests/common/config/.env.valid");
        
        let path_to_dotenv = cwd.join(relative_path);
        println!("Current working directory: {:?}", cwd);
        println!("Relative path to .env file: {:?}", relative_path);
        println!("Using dotenv file at: {:?}", path_to_dotenv);
        // Load the (valid) .env file from the hardcoded path
        dotenvy::from_path(path_to_dotenv)
            .expect("Hardcoded path to .env fixture should always exist.");

        let expected_result = Ok(());

        let result = validate_env_vars();

        assert_eq!(result, expected_result);
    }
}
