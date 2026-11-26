//! # Usage Key
//!
//! Provides [`UsageKey`], the key of the call count of one account, channel, and UTC day.
//!
//! ## Modules
//!
//! - [`invalid_usage_key`]: The [`InvalidUsageKey`] for a string that is not a usage key, and its
//!   [`InvalidUsageKeyReason`].

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use chrono::NaiveDate;

use crate::shared::{AccountId, Channel};

pub mod invalid_usage_key;

pub use invalid_usage_key::{InvalidUsageKey, InvalidUsageKeyReason};

/// The key of the call count of one account, on one channel, on one UTC day.
///
/// The quota store keeps one count per key. The day is the calendar day in UTC, so a new count
/// starts at 00:00 UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UsageKey {
    account_id: AccountId,
    channel: Channel,
    day: NaiveDate,
}

impl UsageKey {
    /// Creates a [`UsageKey`] from an account, a channel, and a UTC day.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chrono::NaiveDate;
    /// use storage::{AccountId, Channel, UsageKey};
    /// use uuid::Uuid;
    ///
    /// let account_id = AccountId::from_uuid(Uuid::from_u128(1));
    /// let day = NaiveDate::from_ymd_opt(2026, 11, 26)
    ///     .expect("Given a real calendar date, the creation of a date should always succeed");
    ///
    /// let expected_result = "00000000-0000-0000-0000-000000000001:api_key:2026-11-26";
    ///
    /// let result = UsageKey::new(account_id, Channel::ApiKey, day).to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(account_id: AccountId, channel: Channel, day: NaiveDate) -> Self {
        Self {
            account_id,
            channel,
            day,
        }
    }

    /// Returns the account.
    #[must_use]
    pub const fn account_id(&self) -> AccountId {
        self.account_id
    }

    /// Returns the channel.
    #[must_use]
    pub const fn channel(&self) -> Channel {
        self.channel
    }

    /// Returns the UTC day.
    #[must_use]
    pub const fn day(&self) -> NaiveDate {
        self.day
    }
}

impl Display for UsageKey {
    /// Writes the account, the channel, and the day, separated by colons, such as
    /// `00000000-0000-0000-0000-000000000001:login:2026-11-26`.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.account_id, self.channel, self.day)
    }
}

impl FromStr for UsageKey {
    type Err = InvalidUsageKey;

    /// Parses `<account_id>:<channel>:<day>` into a [`UsageKey`], the form that
    /// [`Display`] writes.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidUsageKey`] with the [`InvalidUsageKeyReason`] of the first part that
    /// fails:
    ///
    /// - [`InvalidUsageKeyReason::WrongPartCount`] if the string has more or fewer than three
    ///   parts.
    /// - [`InvalidUsageKeyReason::InvalidAccountId`] if the first part is not a UUID.
    /// - [`InvalidUsageKeyReason::InvalidChannel`] if the second part is not a [`Channel`].
    /// - [`InvalidUsageKeyReason::InvalidDay`] if the third part is not a date such as
    ///   `2026-11-26`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::{InvalidUsageKey, InvalidUsageKeyReason, UsageKey};
    ///
    /// let value = "00000000-0000-0000-0000-000000000001:web:2026-11-26";
    ///
    /// let expected_result = Err(InvalidUsageKey::new(
    ///     InvalidUsageKeyReason::InvalidChannel,
    ///     "00000000-0000-0000-0000-000000000001:web:2026-11-26",
    /// ));
    ///
    /// let result = value.parse::<UsageKey>();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let reject = |reason| InvalidUsageKey::new(reason, value);
        let parts: Vec<&str> = value.split(':').collect();
        let [account_id, channel, day] = parts[..] else {
            return Err(reject(InvalidUsageKeyReason::WrongPartCount {
                count: parts.len(),
            }));
        };
        let account_id = AccountId::parse(account_id)
            .map_err(|_| reject(InvalidUsageKeyReason::InvalidAccountId))?;
        let channel = channel
            .parse::<Channel>()
            .map_err(|_| reject(InvalidUsageKeyReason::InvalidChannel))?;
        let day = day
            .parse::<NaiveDate>()
            .map_err(|_| reject(InvalidUsageKeyReason::InvalidDay))?;
        Ok(Self::new(account_id, channel, day))
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;

    fn sample_usage_key(channel: Channel) -> UsageKey {
        let account_id =
            AccountId::from_uuid(Uuid::from_u128(0x0193_b2c1_7d4e_7a2b_9c3d_4e5f_6a7b_8c9d));
        let day = NaiveDate::from_ymd_opt(2026, 11, 26)
            .expect("Given a real calendar date, the creation of a date should always succeed");
        UsageKey::new(account_id, channel, day)
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_usage_key() {
        implements_auto_traits::<UsageKey>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_usage_key() {
        implements_value_traits::<UsageKey>();
    }

    #[test]
    fn should_display_the_account_the_channel_and_the_day_for_a_usage_key() {
        let usage_key = sample_usage_key(Channel::Login);

        let expected_result = "0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d:login:2026-11-26";

        let result = usage_key.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_the_displayed_text_back_to_the_same_usage_key_for_every_channel() {
        let usage_keys = [
            sample_usage_key(Channel::ApiKey),
            sample_usage_key(Channel::Login),
        ];

        let expected_result = vec![
            Ok(sample_usage_key(Channel::ApiKey)),
            Ok(sample_usage_key(Channel::Login)),
        ];

        let result: Vec<_> = usage_keys
            .iter()
            .map(|usage_key| usage_key.to_string().parse::<UsageKey>())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_wrong_part_count_when_the_string_has_two_parts() {
        let value = "0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d:login";

        let expected_result = Err(InvalidUsageKey::new(
            InvalidUsageKeyReason::WrongPartCount { count: 2 },
            "0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d:login",
        ));

        let result = value.parse::<UsageKey>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_wrong_part_count_when_the_account_id_is_a_urn() {
        let value = "urn:uuid:0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d:login:2026-11-26";

        let expected_result = Err(InvalidUsageKey::new(
            InvalidUsageKeyReason::WrongPartCount { count: 5 },
            "urn:uuid:0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d:login:2026-11-26",
        ));

        let result = value.parse::<UsageKey>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_account_id_when_the_first_part_is_not_a_uuid() {
        let value = "user-42:login:2026-11-26";

        let expected_result = Err(InvalidUsageKey::new(
            InvalidUsageKeyReason::InvalidAccountId,
            "user-42:login:2026-11-26",
        ));

        let result = value.parse::<UsageKey>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_channel_when_the_second_part_is_not_a_channel() {
        let value = "0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d:web:2026-11-26";

        let expected_result = Err(InvalidUsageKey::new(
            InvalidUsageKeyReason::InvalidChannel,
            "0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d:web:2026-11-26",
        ));

        let result = value.parse::<UsageKey>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_day_when_the_third_part_is_not_a_calendar_date() {
        let value = "0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d:login:2026-02-30";

        let expected_result = Err(InvalidUsageKey::new(
            InvalidUsageKeyReason::InvalidDay,
            "0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d:login:2026-02-30",
        ));

        let result = value.parse::<UsageKey>();

        assert_eq!(result, expected_result);
    }
}
