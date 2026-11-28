//! # Channel
//!
//! Provides [`Channel`], the kind of credential a `/v1` call arrives with.
//!
//! ## Modules
//!
//! - [`invalid_channel`]: The [`InvalidChannel`] for a string that is not a channel.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

pub mod invalid_channel;

pub use invalid_channel::InvalidChannel;

/// The kind of credential a `/v1` call arrives with: an API key or a login token.
///
/// The quota counts each channel apart. A screener user does not use up the API quota of the same
/// account, and an API client does not use up the daily cap of the login channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Channel {
    /// A call with an API key.
    ApiKey,
    /// A call with a Supabase login token.
    Login,
}

impl Channel {
    /// Returns the name of the channel as Postgres stores it: `api_key` or `login`.
    ///
    /// Stored rows hold these names, so a changed name breaks them.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ApiKey => "api_key",
            Self::Login => "login",
        }
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Channel {
    type Err = InvalidChannel;

    /// Parses `api_key` or `login` into a [`Channel`].
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidChannel`] for any other string, including `ApiKey`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::Channel;
    ///
    /// let value = "api_key";
    ///
    /// let expected_result = Ok(Channel::ApiKey);
    ///
    /// let result = value.parse::<Channel>();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "api_key" => Ok(Self::ApiKey),
            "login" => Ok(Self::Login),
            _ => Err(InvalidChannel::new(value)),
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
    const fn should_implement_auto_traits_for_channel() {
        implements_auto_traits::<Channel>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_channel() {
        implements_value_traits::<Channel>();
    }

    #[test]
    fn should_display_api_key_for_the_api_key_channel() {
        let channel = Channel::ApiKey;

        let expected_result = "api_key";

        let result = channel.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_login_for_the_login_channel() {
        let channel = Channel::Login;

        let expected_result = "login";

        let result = channel.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_the_displayed_text_back_to_the_same_channel_for_every_channel() {
        let channels = [Channel::ApiKey, Channel::Login];

        let expected_result = vec![Ok(Channel::ApiKey), Ok(Channel::Login)];

        let result: Vec<_> = channels
            .iter()
            .map(|channel| channel.to_string().parse::<Channel>())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_channel_when_the_string_is_the_variant_name() {
        let value = "ApiKey";

        let expected_result = Err(InvalidChannel::new("ApiKey"));

        let result = value.parse::<Channel>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_channel_when_the_string_is_empty() {
        let value = "";

        let expected_result = Err(InvalidChannel::new(""));

        let result = value.parse::<Channel>();

        assert_eq!(result, expected_result);
    }
}
