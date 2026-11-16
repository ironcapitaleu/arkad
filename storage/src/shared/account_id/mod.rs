//! # Account ID
//!
//! Provides [`AccountId`], the key of an account: the user ID of Supabase Auth.
//!
//! ## Modules
//!
//! - [`invalid_account_id`]: The [`InvalidAccountId`] for a string that is not a UUID.

use std::fmt::{self, Display, Formatter};

use uuid::Uuid;

pub mod invalid_account_id;

pub use invalid_account_id::InvalidAccountId;

/// The key of an account: the user ID of Supabase Auth (`auth.users.id`), a UUID.
///
/// One account is one Supabase user. A login token carries the ID in its `sub` claim. The API keys,
/// the plan, and the call counts of an account key on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId(Uuid);

impl AccountId {
    /// Creates an [`AccountId`] from the text of a UUID, such as the `sub` claim of a login token.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidAccountId`] if the text is not a UUID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::{AccountId, InvalidAccountId};
    ///
    /// let value = "not-a-uuid";
    ///
    /// let expected_result = Err(InvalidAccountId::new("not-a-uuid"));
    ///
    /// let result = AccountId::parse(value);
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn parse(value: &str) -> Result<Self, InvalidAccountId> {
        Uuid::try_parse(value)
            .map(Self)
            .map_err(|_| InvalidAccountId::new(value))
    }

    /// Wraps a UUID that a store read back.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::AccountId;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::from_u128(1);
    ///
    /// let expected_result = uuid;
    ///
    /// let result = AccountId::from_uuid(uuid).as_uuid();
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

impl Display for AccountId {
    /// Writes the UUID in its hyphenated lowercase form.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
    const fn should_implement_auto_traits_for_account_id() {
        implements_auto_traits::<AccountId>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_account_id() {
        implements_value_traits::<AccountId>();
    }

    #[test]
    fn should_create_the_account_id_when_the_string_is_a_hyphenated_uuid() {
        let value = "0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d";

        let expected_result = Ok(AccountId::from_uuid(Uuid::from_u128(
            0x0193_b2c1_7d4e_7a2b_9c3d_4e5f_6a7b_8c9d,
        )));

        let result = AccountId::parse(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_account_id_when_the_string_is_not_a_uuid() {
        let value = "user-42";

        let expected_result = Err(InvalidAccountId::new("user-42"));

        let result = AccountId::parse(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_account_id_when_the_string_is_empty() {
        let value = "";

        let expected_result = Err(InvalidAccountId::new(""));

        let result = AccountId::parse(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_account_id_when_the_uuid_misses_a_digit() {
        let value = "0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9";

        let expected_result = Err(InvalidAccountId::new("0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9"));

        let result = AccountId::parse(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_hyphenated_uuid_for_an_account_id() {
        let account_id = AccountId::from_uuid(Uuid::from_u128(1));

        let expected_result = "00000000-0000-0000-0000-000000000001";

        let result = account_id.to_string();

        assert_eq!(result, expected_result);
    }
}
