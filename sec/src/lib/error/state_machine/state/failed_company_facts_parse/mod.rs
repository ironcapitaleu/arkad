//! # Failed Company Facts Parse Error
//!
//! Provides the [`FailedCompanyFactsParse`] error: a [`ParseErrorKind`] of the Company Facts
//! parser, with the name of the state in which the parse failed.

use std::error::Error;
use std::fmt;

use xbrl::error::parsing::ParseErrorKind;

use super::State as StateError;
use crate::traits::error::FromDomainError;

/// Error for a Company Facts body that the parser rejects, tagged with the state it occurred in.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FailedCompanyFactsParse {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The error of the parser. It is boxed, because it is larger than the other state errors.
    pub parse_error: Box<ParseErrorKind>,
}

// Deviation: `Display` and `source()` are written by hand, because the derive of `thiserror`
// makes `source()` return the `Box` and not the `ParseErrorKind` in it.
impl fmt::Display for FailedCompanyFactsParse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[FailedCompanyFactsParse] Failed to parse the Company Facts body, Caused by: {}",
            self.parse_error
        )
    }
}

impl Error for FailedCompanyFactsParse {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.parse_error.as_ref())
    }
}

impl FailedCompanyFactsParse {
    /// Creates a new [`FailedCompanyFactsParse`] error.
    #[must_use]
    pub fn new(state_name: impl Into<String>, parse_error: ParseErrorKind) -> Self {
        Self {
            state_name: state_name.into(),
            parse_error: Box::new(parse_error),
        }
    }
}

impl From<FailedCompanyFactsParse> for StateError {
    /// Wraps the error in the [`StateError::FailedCompanyFactsParse`] variant.
    fn from(error: FailedCompanyFactsParse) -> Self {
        Self::FailedCompanyFactsParse(error)
    }
}

impl FromDomainError<ParseErrorKind> for FailedCompanyFactsParse {
    type DomainErr = ParseErrorKind;

    /// Wraps a [`ParseErrorKind`] with the name of the state.
    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;

    const STATE_NAME: &str = "Parse Company Facts";

    fn missing_facts_key() -> ParseErrorKind {
        ParseErrorKind::MissingTopLevelKey {
            key: "facts".to_string(),
        }
    }

    #[test]
    fn should_chain_the_parse_error_after_caused_by_when_displayed() {
        let error = FailedCompanyFactsParse::new(STATE_NAME, missing_facts_key());

        let expected_result = "[FailedCompanyFactsParse] Failed to parse the Company Facts body, Caused by: [MissingTopLevelKey] Required key 'facts' not found in response";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expose_the_parse_error_as_source_when_wrapping_it() {
        let error = FailedCompanyFactsParse::new(STATE_NAME, missing_facts_key());

        let expected_result = Some(&missing_facts_key());

        let result =
            Error::source(&error).and_then(|source| source.downcast_ref::<ParseErrorKind>());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_from_is_called() {
        let error = FailedCompanyFactsParse::new(STATE_NAME, missing_facts_key());

        let expected_result = StateError::FailedCompanyFactsParse(FailedCompanyFactsParse {
            state_name: "Parse Company Facts".to_owned(),
            parse_error: Box::new(ParseErrorKind::MissingTopLevelKey {
                key: "facts".to_owned(),
            }),
        });

        let result = StateError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_from_domain_error_when_from_domain_error_is_called() {
        let parse_error = missing_facts_key();

        let expected_result = FailedCompanyFactsParse::new(STATE_NAME, missing_facts_key());

        let result = FailedCompanyFactsParse::from_domain_error(STATE_NAME, parse_error);

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_failed_company_facts_parse() {
        implements_auto_traits::<FailedCompanyFactsParse>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_failed_company_facts_parse() {
        implements_value_traits::<FailedCompanyFactsParse>();
    }
}
