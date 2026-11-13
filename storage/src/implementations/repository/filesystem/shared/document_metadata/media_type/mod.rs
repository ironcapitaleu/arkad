//! # Media Type
//!
//! Provides [`MediaType`], the media type of the bytes of a stored document.
//!
//! ## Modules
//!
//! - [`invalid_media_type`]: The [`InvalidMediaType`] for a string that is not a media type.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub mod invalid_media_type;

pub use invalid_media_type::{InvalidMediaType, InvalidMediaTypeReason};

/// The media type of the bytes of a stored document, of the form `type/subtype`.
///
/// [`MediaType::new`] normalizes the input before it checks it. It drops the parameters after the
/// first `;`, such as `; charset=utf-8`, trims whitespace, and lowercases the text, because media
/// types are case-insensitive (RFC 9110, section 8.3.1). Then the type and the subtype must each
/// be a non-empty RFC 9110 token other than the wildcard `*`.
///
/// [`Display`] and serde write the normalized text. Deserializing runs [`MediaType::new`], so a
/// stored value that is not a media type fails to read.
///
/// # Examples
///
/// ```rust
/// use storage::implementations::repository::filesystem::MediaType;
///
/// let content_type = "Application/JSON; charset=utf-8";
///
/// let expected_result = "application/json";
///
/// let media_type = MediaType::new(content_type)
///     .expect("Given a type and a subtype of token characters, the media type should be valid");
/// let result = media_type.as_str();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct MediaType(String);

impl MediaType {
    /// Creates a new [`MediaType`] from the text of a media type, such as a `Content-Type` header.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidMediaType`] if, after the normalization in [`MediaType`], the text has
    /// no `/`, an empty type or subtype, a wildcard type or subtype, or a character that is not
    /// allowed in a token, such as a space or a second `/`.
    pub fn new(media_type: &str) -> Result<Self, InvalidMediaType> {
        let normalized = media_type
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        let reject = |reason| InvalidMediaType::new(media_type, reason);
        let (type_part, subtype) = normalized
            .split_once('/')
            .ok_or_else(|| reject(InvalidMediaTypeReason::MissingSlash))?;
        if type_part.is_empty() {
            return Err(reject(InvalidMediaTypeReason::EmptyType));
        }
        if subtype.is_empty() {
            return Err(reject(InvalidMediaTypeReason::EmptySubtype));
        }
        if type_part == "*" || subtype == "*" {
            return Err(reject(InvalidMediaTypeReason::Wildcard));
        }
        if let Some(character) = type_part
            .chars()
            .chain(subtype.chars())
            .find(|character| !is_token_character(*character))
        {
            return Err(reject(InvalidMediaTypeReason::InvalidCharacter(character)));
        }
        Ok(Self(normalized))
    }

    /// Returns the media type `application/octet-stream`, for bytes of an unknown type.
    ///
    /// RFC 9110, section 8.3, lets a recipient treat bytes without a media type as
    /// `application/octet-stream`.
    #[must_use]
    pub fn octet_stream() -> Self {
        Self("application/octet-stream".to_owned())
    }

    /// Returns the normalized text of the media type.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Returns whether RFC 9110, section 5.6.2, allows the character in a token.
const fn is_token_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(
            character,
            '!' | '#'
                | '$'
                | '%'
                | '&'
                | '\''
                | '*'
                | '+'
                | '-'
                | '.'
                | '^'
                | '_'
                | '`'
                | '|'
                | '~'
        )
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for MediaType {
    type Err = InvalidMediaType;

    /// Parses the text of a media type with [`MediaType::new`].
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidMediaType`] in the cases [`MediaType::new`] names.
    fn from_str(media_type: &str) -> Result<Self, Self::Err> {
        Self::new(media_type)
    }
}

impl TryFrom<String> for MediaType {
    type Error = InvalidMediaType;

    /// Parses the text of a media type with [`MediaType::new`].
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidMediaType`] in the cases [`MediaType::new`] names.
    fn try_from(media_type: String) -> Result<Self, Self::Error> {
        Self::new(&media_type)
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
    const fn should_implement_auto_traits_for_media_type() {
        implements_auto_traits::<MediaType>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_media_type() {
        implements_value_traits::<MediaType>();
    }

    #[test]
    fn should_keep_the_text_when_the_media_type_is_already_normalized() {
        let media_type = "application/vnd.sec+xml";

        let expected_result = Ok("application/vnd.sec+xml".to_owned());

        let result = MediaType::new(media_type).map(|media_type| media_type.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_the_parameters_when_the_media_type_has_parameters() {
        let media_type = "text/html; charset=utf-8";

        let expected_result = Ok("text/html".to_owned());

        let result = MediaType::new(media_type).map(|media_type| media_type.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_lowercase_and_trim_the_text_when_the_media_type_has_uppercase_letters_and_spaces() {
        let media_type = " Application/JSON ";

        let expected_result = Ok("application/json".to_owned());

        let result = MediaType::new(media_type).map(|media_type| media_type.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_slash_when_the_text_has_no_slash() {
        let media_type = "unknown";

        let expected_result = Err(InvalidMediaType::new(
            "unknown",
            InvalidMediaTypeReason::MissingSlash,
        ));

        let result = MediaType::new(media_type);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_empty_type_when_the_text_starts_with_a_slash() {
        let media_type = "/json";

        let expected_result = Err(InvalidMediaType::new(
            "/json",
            InvalidMediaTypeReason::EmptyType,
        ));

        let result = MediaType::new(media_type);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_empty_subtype_when_the_text_ends_with_a_slash() {
        let media_type = "application/";

        let expected_result = Err(InvalidMediaType::new(
            "application/",
            InvalidMediaTypeReason::EmptySubtype,
        ));

        let result = MediaType::new(media_type);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_character_when_the_subtype_holds_a_second_slash() {
        let media_type = "application/json/x";

        let expected_result = Err(InvalidMediaType::new(
            "application/json/x",
            InvalidMediaTypeReason::InvalidCharacter('/'),
        ));

        let result = MediaType::new(media_type);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_character_when_the_type_holds_a_space() {
        let media_type = "text plain/x";

        let expected_result = Err(InvalidMediaType::new(
            "text plain/x",
            InvalidMediaTypeReason::InvalidCharacter(' '),
        ));

        let result = MediaType::new(media_type);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_wildcard_when_the_subtype_is_a_wildcard() {
        let media_type = "text/*";

        let expected_result = Err(InvalidMediaType::new(
            "text/*",
            InvalidMediaTypeReason::Wildcard,
        ));

        let result = MediaType::new(media_type);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_character_when_the_subtype_holds_a_non_ascii_character() {
        let media_type = "text/plaın";

        let expected_result = Err(InvalidMediaType::new(
            "text/plaın",
            InvalidMediaTypeReason::InvalidCharacter('ı'),
        ));

        let result = MediaType::new(media_type);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_slash_when_the_text_is_empty() {
        let media_type = "";

        let expected_result = Err(InvalidMediaType::new(
            "",
            InvalidMediaTypeReason::MissingSlash,
        ));

        let result = MediaType::new(media_type);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_application_octet_stream_for_the_octet_stream_media_type() {
        let media_type = MediaType::octet_stream();

        let expected_result = "application/octet-stream";

        let result = media_type.as_str();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_display_text_for_a_media_type() {
        let media_type = MediaType::new("application/json")
            .expect("Given a hardcoded valid media type, creation should always succeed");

        let expected_result = serde_json::json!("application/json");

        let result = serde_json::to_value(media_type)
            .expect("Given a newtype of a string, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_the_media_type_when_the_text_is_a_media_type() {
        let json = "\"application/json\"";

        let expected_result = MediaType::new("application/json")
            .expect("Given a hardcoded valid media type, creation should always succeed");

        let result: MediaType = serde_json::from_str(json)
            .expect("Given the text of a media type, deserialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_read_the_media_type_when_the_text_is_not_a_media_type() {
        let json = "\"unknown\"";

        let expected_result = true;

        let result = serde_json::from_str::<MediaType>(json).is_err();

        assert_eq!(result, expected_result);
    }
}
