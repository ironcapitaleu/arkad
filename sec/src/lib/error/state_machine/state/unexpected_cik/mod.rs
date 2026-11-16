//! # Unexpected CIK Error
//!
//! Provides the [`UnexpectedCik`] error: a Company Facts body about another company than the CIK
//! of the state.

use thiserror::Error;

use super::State as StateError;
use crate::shared::cik::Cik;

/// Error for a Company Facts body about another company, tagged with the state it occurred in.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[UnexpectedCik] Company Facts body is about CIK '{found}', not CIK '{expected}'")]
pub struct UnexpectedCik {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The CIK of the state context.
    pub expected: Cik,
    /// The CIK of the body, padded to 10 digits.
    pub found: String,
}

impl UnexpectedCik {
    /// Creates a new [`UnexpectedCik`] error.
    #[must_use]
    pub fn new(state_name: impl Into<String>, expected: Cik, found: impl Into<String>) -> Self {
        Self {
            state_name: state_name.into(),
            expected,
            found: found.into(),
        }
    }
}

impl From<UnexpectedCik> for StateError {
    /// Wraps the error in the [`StateError::UnexpectedCik`] variant.
    fn from(error: UnexpectedCik) -> Self {
        Self::UnexpectedCik(error)
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;

    const STATE_NAME: &str = "Parse Company Facts";

    fn berkshire_cik() -> Cik {
        Cik::new("0001067983")
            .expect("Given a hardcoded valid CIK, the creation should always succeed")
    }

    #[test]
    fn should_name_both_ciks_when_displayed() {
        let error = UnexpectedCik::new(STATE_NAME, berkshire_cik(), "0000320193");

        let expected_result =
            "[UnexpectedCik] Company Facts body is about CIK '0000320193', not CIK '0001067983'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_from_is_called() {
        let error = UnexpectedCik::new(STATE_NAME, berkshire_cik(), "0000320193");

        let expected_result = StateError::UnexpectedCik(error.clone());

        let result = StateError::from(error);

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_unexpected_cik() {
        implements_auto_traits::<UnexpectedCik>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_unexpected_cik() {
        implements_value_traits::<UnexpectedCik>();
    }
}
