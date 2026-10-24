//! # URL
//!
//! Provides the [`Url`] newtype guaranteeing its inner string is a parseable URL.
//!
//! ## Modules
//!
//! - [`conversions`]: Conversions into [`Url`] from external URL types.
//! - [`url_error`]: The [`UrlError`] returned when parsing fails.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use self::url_error::{InvalidUrlReason, UrlError};

pub mod conversions;
pub mod url_error;

/// A string validated as a parseable URL.
///
/// A newtype over [`String`] whose inner value is validated at construction, so holding a `Url`
/// guarantees it parses.
///
/// A `Url` serializes as its string. Deserializing goes through [`Url::from_string`], so a URL
/// read from a file passes the same check.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Url {
    value: String,
}

impl FromStr for Url {
    type Err = UrlError;

    /// Validates a string slice into a [`Url`].
    ///
    /// # Errors
    ///
    /// Returns [`UrlError`] ([`InvalidUrlReason::FailedToParse`]) if the input is not a valid URL.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::url::Url;
    ///
    /// let url: Url = "https://data.sec.gov/submissions/CIK0001067983.json"
    ///     .parse()
    ///     .expect("A hardcoded valid URL string should always parse");
    ///
    /// let expected_result = "https://data.sec.gov/submissions/CIK0001067983.json";
    ///
    /// let result = url.as_str();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    fn from_str(url: &str) -> Result<Self, Self::Err> {
        url::Url::parse(url).map_or_else(
            |_| Err(UrlError::new(InvalidUrlReason::FailedToParse, url)),
            |parsed| {
                Ok(Self {
                    value: parsed.to_string(),
                })
            },
        )
    }
}

impl Url {
    /// Validates an owned [`String`] into a [`Url`], avoiding the borrow that [`Url::from_str`] takes.
    ///
    /// # Errors
    ///
    /// Returns [`UrlError`] ([`InvalidUrlReason::FailedToParse`]) if the input is not a valid URL.
    pub fn from_string(url: String) -> Result<Self, UrlError> {
        url::Url::parse(&url).map_or_else(
            |_| Err(UrlError::new(InvalidUrlReason::FailedToParse, url)),
            |parsed| {
                Ok(Self {
                    value: parsed.to_string(),
                })
            },
        )
    }

    /// Returns the URL as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl TryFrom<String> for Url {
    type Error = UrlError;

    /// Validates an owned [`String`] into a [`Url`], as [`Url::from_string`] does.
    ///
    /// # Errors
    ///
    /// Returns [`UrlError`] ([`InvalidUrlReason::FailedToParse`]) if the input is not a valid URL.
    fn try_from(url: String) -> Result<Self, Self::Error> {
        Self::from_string(url)
    }
}

impl From<Url> for String {
    /// Returns the URL string of a [`Url`].
    fn from(url: Url) -> Self {
        url.value
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_create_url_when_input_is_valid_https_url_slice() {
        let valid_url = "https://data.sec.gov/submissions/CIK0001067983.json";

        let expected_result = true;

        let result = Url::from_str(valid_url).is_ok();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_string_when_calling_as_str() {
        let valid_url = "https://data.sec.gov/submissions/CIK0001067983.json";

        let expected_result = "https://data.sec.gov/submissions/CIK0001067983.json";

        let result = Url::from_str(valid_url)
            .expect("A hardcoded valid URL string should always parse successfully")
            .as_str()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_url_string_when_formatting() {
        let valid_url = "https://example.com/path";

        let expected_result = "https://example.com/path";

        let result = Url::from_str(valid_url)
            .expect("A hardcoded valid URL string should always parse successfully")
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_input_is_not_a_valid_url() {
        let invalid_url = "not a url";

        let expected_result = InvalidUrlReason::FailedToParse;

        let result = Url::from_str(invalid_url)
            .expect_err("A hardcoded invalid URL string should always fail to parse")
            .reason;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_input_is_empty() {
        let invalid_url = "";
        let expected_result = InvalidUrlReason::FailedToParse;

        let result = Url::from_str(invalid_url)
            .expect_err("A hardcoded empty string should always fail to parse")
            .reason;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_url_when_input_is_valid_http_url() {
        let unvalidated_url = "http://example.com";

        let expected_result = true;

        let result = Url::from_str(unvalidated_url).is_ok();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_preserve_original_input_in_error_when_parsing_fails() {
        let input = "not a url";

        let expected_result = "not a url";

        let result = Url::from_str(input)
            .expect_err("A hardcoded invalid URL string should always fail to parse")
            .invalid_url;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_url_from_owned_string_when_input_is_valid() {
        let unvalidated_url = String::from("https://data.sec.gov/submissions/CIK0001067983.json");

        let expected_result = true;

        let result = Url::from_string(unvalidated_url).is_ok();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_string_when_calling_as_str_on_url_from_owned_string() {
        let unvalidated_url = String::from("https://data.sec.gov/submissions/CIK0001067983.json");

        let expected_result = "https://data.sec.gov/submissions/CIK0001067983.json";

        let result = Url::from_string(unvalidated_url)
            .expect("A hardcoded valid URL string should always parse successfully")
            .as_str()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_owned_string_is_not_a_valid_url() {
        let invalid_url = String::from("not a url");

        let expected_result = InvalidUrlReason::FailedToParse;

        let result = Url::from_string(invalid_url)
            .expect_err("A hardcoded invalid URL string should always fail to parse")
            .reason;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_preserve_original_owned_string_in_error_when_parsing_fails() {
        let invalid_url = String::from("not a url");

        let expected_result = "not a url";

        let result = Url::from_string(invalid_url)
            .expect_err("A hardcoded invalid URL string should always fail to parse")
            .invalid_url;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_as_its_string_for_a_url() {
        let url: Url = "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
            .parse()
            .expect("A hardcoded valid URL string should always parse");

        let expected_result =
            serde_json::json!("https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json");

        let result =
            serde_json::to_value(url).expect("Given a URL, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_same_url_when_deserializing_a_serialized_url() {
        let url: Url = "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
            .parse()
            .expect("A hardcoded valid URL string should always parse");
        let json =
            serde_json::to_string(&url).expect("Given a URL, serialization should always succeed");

        let expected_result = url;

        let result: Url = serde_json::from_str(&json)
            .expect("Given JSON this type produced, deserialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_deserialize_when_the_string_is_not_a_url() {
        let json = r#""not a url""#;

        let expected_result =
            UrlError::new(InvalidUrlReason::FailedToParse, "not a url").to_string();

        let result = serde_json::from_str::<Url>(json)
            .expect_err("Given a string that is not a URL, deserialization should always fail")
            .to_string();

        assert!(result.starts_with(&expected_result));
    }
}
