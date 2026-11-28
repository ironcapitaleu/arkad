//! # Failed Instance Parse Error
//!
//! Provides the [`FailedInstanceParse`] error: a [`ParseErrorKind`] of the instance parser, with
//! the name of the state in which the parse failed.

use std::error::Error;
use std::fmt;

use xbrl::error::parsing::ParseErrorKind;

use super::State as StateError;
use crate::traits::error::FromDomainError;

/// Error for an instance that the parser rejects, tagged with the state it occurred in.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FailedInstanceParse {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The error of the parser. It is boxed, because it is larger than the other state errors.
    pub parse_error: Box<ParseErrorKind>,
}

// Deviation: `Display` and `source()` are written by hand, because the derive of `thiserror`
// makes `source()` return the `Box` and not the `ParseErrorKind` in it.
impl fmt::Display for FailedInstanceParse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[FailedInstanceParse] Failed to parse the instance filing, Caused by: {}",
            self.parse_error
        )
    }
}

impl Error for FailedInstanceParse {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.parse_error.as_ref())
    }
}

impl FailedInstanceParse {
    /// Creates a new [`FailedInstanceParse`] error.
    #[must_use]
    pub fn new(state_name: impl Into<String>, parse_error: ParseErrorKind) -> Self {
        Self {
            state_name: state_name.into(),
            parse_error: Box::new(parse_error),
        }
    }
}

impl From<FailedInstanceParse> for StateError {
    /// Wraps the error in the [`StateError::FailedInstanceParse`] variant.
    fn from(error: FailedInstanceParse) -> Self {
        Self::FailedInstanceParse(error)
    }
}

impl FromDomainError<ParseErrorKind> for FailedInstanceParse {
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
    use xbrl::core::filing::FilingId;

    use super::*;

    const STATE_NAME: &str = "Parse Instance Filing";

    fn invalid_xml() -> ParseErrorKind {
        ParseErrorKind::InvalidXml {
            filing: FilingId::new("sec", "0000320193-26-000001").expect(
                "Given a hardcoded source and accession number, the creation should always succeed",
            ),
            reason: "unexpected end of stream".to_owned(),
        }
    }

    #[test]
    fn should_chain_the_parse_error_after_caused_by_when_displayed() {
        let error = FailedInstanceParse::new(STATE_NAME, invalid_xml());

        let expected_result = "[FailedInstanceParse] Failed to parse the instance filing, Caused by: [InvalidXml] Failed to parse the instance document of the filing 'sec:0000320193-26-000001', Reason: 'unexpected end of stream'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expose_the_parse_error_as_source_when_wrapping_it() {
        let error = FailedInstanceParse::new(STATE_NAME, invalid_xml());

        let expected_result = Some(&invalid_xml());

        let result =
            Error::source(&error).and_then(|source| source.downcast_ref::<ParseErrorKind>());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_from_is_called() {
        let error = FailedInstanceParse::new(STATE_NAME, invalid_xml());

        let expected_result = StateError::FailedInstanceParse(FailedInstanceParse {
            state_name: "Parse Instance Filing".to_owned(),
            parse_error: Box::new(invalid_xml()),
        });

        let result = StateError::from(error);

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_failed_instance_parse() {
        implements_auto_traits::<FailedInstanceParse>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_failed_instance_parse() {
        implements_value_traits::<FailedInstanceParse>();
    }
}
