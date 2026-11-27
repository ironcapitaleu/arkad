//! # API Key ID
//!
//! Provides [`ApiKeyId`], the key of one API key of an account.
//!
//! ## Modules
//!
//! - [`invalid_api_key_id`]: The [`InvalidApiKeyId`] for a string that is not a UUID.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use uuid::Uuid;

pub mod invalid_api_key_id;

pub use invalid_api_key_id::InvalidApiKeyId;

/// The key of one API key, a UUID.
///
/// The ID names a key in the list and revoke calls. It is not the secret of the key, so it can
/// appear in a URL or a log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApiKeyId(Uuid);

impl ApiKeyId {
    /// Wraps a UUID that a store read back.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::ApiKeyId;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::from_u128(1);
    ///
    /// let expected_result = uuid;
    ///
    /// let result = ApiKeyId::from_uuid(uuid).as_uuid();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the UUID, for a store that writes the key.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Display for ApiKeyId {
    /// Writes the UUID in its hyphenated lowercase form, the form Postgres prints for a `uuid`.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ApiKeyId {
    type Err = InvalidApiKeyId;

    /// Parses the text of a UUID into an [`ApiKeyId`].
    ///
    /// It accepts every text form of a UUID: hyphenated, simple, braced, and URN.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidApiKeyId`] if the text is not a UUID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::{ApiKeyId, InvalidApiKeyId};
    ///
    /// let value = "not-a-uuid";
    ///
    /// let expected_result = Err(InvalidApiKeyId::new("not-a-uuid"));
    ///
    /// let result = value.parse::<ApiKeyId>();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::try_parse(value)
            .map(Self)
            .map_err(|_| InvalidApiKeyId::new(value))
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
    const fn should_implement_auto_traits_for_api_key_id() {
        implements_auto_traits::<ApiKeyId>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_api_key_id() {
        implements_value_traits::<ApiKeyId>();
    }

    #[test]
    fn should_create_the_api_key_id_when_the_string_is_a_hyphenated_uuid() {
        let value = "0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d";

        let expected_result = Ok(ApiKeyId::from_uuid(Uuid::from_u128(
            0x0193_b2c1_7d4e_7a2b_9c3d_4e5f_6a7b_8c9d,
        )));

        let result = value.parse::<ApiKeyId>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_api_key_id_when_the_string_is_not_a_uuid() {
        let value = "key-42";

        let expected_result = Err(InvalidApiKeyId::new("key-42"));

        let result = value.parse::<ApiKeyId>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_api_key_id_when_the_string_is_empty() {
        let value = "";

        let expected_result = Err(InvalidApiKeyId::new(""));

        let result = value.parse::<ApiKeyId>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_hyphenated_uuid_for_an_api_key_id() {
        let api_key_id = ApiKeyId::from_uuid(Uuid::from_u128(1));

        let expected_result = "00000000-0000-0000-0000-000000000001";

        let result = api_key_id.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_the_displayed_text_back_to_the_same_api_key_id() {
        let api_key_id =
            ApiKeyId::from_uuid(Uuid::from_u128(0x0193_b2c1_7d4e_7a2b_9c3d_4e5f_6a7b_8c9d));
        let text = api_key_id.to_string();

        let expected_result = Ok(api_key_id);

        let result = text.parse::<ApiKeyId>();

        assert_eq!(result, expected_result);
    }
}
