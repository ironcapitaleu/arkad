//! # Key Environment
//!
//! Provides [`KeyEnvironment`], the deployment an API key works in.
//!
//! ## Modules
//!
//! - [`invalid_key_environment`]: The [`InvalidKeyEnvironment`] for a string that is not an
//!   environment.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

pub mod invalid_key_environment;

pub use invalid_key_environment::InvalidKeyEnvironment;

/// The deployment an API key works in: `live` in production, `test` in development.
///
/// A key of one environment fails in the other. The environment is part of the key prefix, such
/// as `ak_live_`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyEnvironment {
    /// A key for the production deployment.
    Live,
    /// A key for a development deployment.
    Test,
}

impl KeyEnvironment {
    /// Returns the name of the environment as Postgres stores it: `live` or `test`.
    ///
    /// Stored rows hold these names, so a changed name breaks them.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Test => "test",
        }
    }
}

impl Display for KeyEnvironment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for KeyEnvironment {
    type Err = InvalidKeyEnvironment;

    /// Parses `live` or `test` into a [`KeyEnvironment`].
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidKeyEnvironment`] for any other string, including `Live`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::KeyEnvironment;
    ///
    /// let value = "test";
    ///
    /// let expected_result = Ok(KeyEnvironment::Test);
    ///
    /// let result = value.parse::<KeyEnvironment>();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "live" => Ok(Self::Live),
            "test" => Ok(Self::Test),
            _ => Err(InvalidKeyEnvironment::new(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_key_environment() {
        implements_auto_traits::<KeyEnvironment>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_key_environment() {
        implements_value_traits::<KeyEnvironment>();
    }

    #[test]
    fn should_display_live_for_the_live_environment() {
        let environment = KeyEnvironment::Live;

        let expected_result = "live";

        let result = environment.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_test_for_the_test_environment() {
        let environment = KeyEnvironment::Test;

        let expected_result = "test";

        let result = environment.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_the_displayed_text_back_to_the_same_environment_for_every_environment() {
        let environments = [KeyEnvironment::Live, KeyEnvironment::Test];

        let expected_result = vec![Ok(KeyEnvironment::Live), Ok(KeyEnvironment::Test)];

        let result: Vec<_> = environments
            .iter()
            .map(|environment| environment.to_string().parse::<KeyEnvironment>())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_key_environment_when_the_string_is_capitalized() {
        let value = "Live";

        let expected_result = Err(InvalidKeyEnvironment::new("Live"));

        let result = value.parse::<KeyEnvironment>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_key_environment_when_the_string_is_empty() {
        let value = "";

        let expected_result = Err(InvalidKeyEnvironment::new(""));

        let result = value.parse::<KeyEnvironment>();

        assert_eq!(result, expected_result);
    }
}
