//! # Invalid Media Type
//!
//! Provides [`InvalidMediaType`], the error raised when a string is not a media type, and
//! [`InvalidMediaTypeReason`], the rule the string breaks.

use std::fmt::{self, Display, Formatter};

use thiserror::Error;

/// Error raised when a string is not a media type of the form `type/subtype`.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[InvalidMediaType] Media type '{value}' is not of the form 'type/subtype', Reason: '{reason}'"
)]
pub struct InvalidMediaType {
    /// The string that was rejected.
    pub value: String,
    /// The rule that the string breaks.
    pub reason: InvalidMediaTypeReason,
}

impl InvalidMediaType {
    /// Creates a new [`InvalidMediaType`] for the rejected string and the rule it breaks.
    #[must_use]
    pub fn new(value: impl Into<String>, reason: InvalidMediaTypeReason) -> Self {
        Self {
            value: value.into(),
            reason,
        }
    }
}

/// The rule of the form `type/subtype` that a string breaks.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidMediaTypeReason {
    /// The string has no `/` between the type and the subtype.
    MissingSlash,
    /// The part before the `/` is empty.
    EmptyType,
    /// The part after the `/` is empty.
    EmptySubtype,
    /// The type or the subtype holds a character that RFC 9110 does not allow in a token.
    InvalidCharacter(char),
    /// The type or the subtype is the wildcard `*`, which names a media range, not a media type.
    Wildcard,
}

impl Display for InvalidMediaTypeReason {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::MissingSlash => write!(f, "No slash between the type and the subtype"),
            Self::EmptyType => write!(f, "The type before the slash is empty"),
            Self::EmptySubtype => write!(f, "The subtype after the slash is empty"),
            Self::InvalidCharacter(character) => {
                write!(
                    f,
                    "Character U+{:04X} is not allowed in a token",
                    u32::from(*character)
                )
            }
            Self::Wildcard => write!(f, "The wildcard '*' names a media range, not a media type"),
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
    const fn should_implement_auto_traits_for_invalid_media_type() {
        implements_auto_traits::<InvalidMediaType>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_media_type() {
        implements_value_traits::<InvalidMediaType>();
    }

    #[test]
    fn should_format_display_as_expected_for_a_media_type_without_a_slash() {
        let error = InvalidMediaType::new("unknown", InvalidMediaTypeReason::MissingSlash);

        let expected_result = "[InvalidMediaType] Media type 'unknown' is not of the form 'type/subtype', Reason: 'No slash between the type and the subtype'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_for_a_media_type_with_an_invalid_character() {
        let error =
            InvalidMediaType::new("text/a b", InvalidMediaTypeReason::InvalidCharacter(' '));

        let expected_result = "[InvalidMediaType] Media type 'text/a b' is not of the form 'type/subtype', Reason: 'Character U+0020 is not allowed in a token'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_for_a_wildcard_media_type() {
        let error = InvalidMediaType::new("text/*", InvalidMediaTypeReason::Wildcard);

        let expected_result = "[InvalidMediaType] Media type 'text/*' is not of the form 'type/subtype', Reason: 'The wildcard '*' names a media range, not a media type'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
