//! # Account Plan
//!
//! Provides [`AccountPlan`], the plan an account is on.
//!
//! ## Modules
//!
//! - [`invalid_account_plan`]: The [`InvalidAccountPlan`] for a string that is not a plan.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

pub mod invalid_account_plan;

pub use invalid_account_plan::InvalidAccountPlan;

/// The plan an account is on: Free, Screener Pro, API Developer, or API Business.
///
/// The plan sets the API access, the quotas, and the depth of data of an account. This type names
/// the plan only. It holds no limit values, because the limits stay out of the public repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccountPlan {
    /// The plan of an account without a subscription.
    Free,
    /// The paid plan for the screener in the frontend.
    ScreenerPro,
    /// The paid plan for a developer who calls the API.
    ApiDeveloper,
    /// The paid plan for a business that calls the API.
    ApiBusiness,
}

impl AccountPlan {
    /// Returns the name of the plan as Postgres stores it, such as `screener_pro`.
    ///
    /// Stored rows hold these names, so a changed name breaks them.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::ScreenerPro => "screener_pro",
            Self::ApiDeveloper => "api_developer",
            Self::ApiBusiness => "api_business",
        }
    }
}

impl Display for AccountPlan {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AccountPlan {
    type Err = InvalidAccountPlan;

    /// Parses `free`, `screener_pro`, `api_developer`, or `api_business` into an [`AccountPlan`].
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidAccountPlan`] for any other string, including `Screener Pro`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::AccountPlan;
    ///
    /// let value = "screener_pro";
    ///
    /// let expected_result = Ok(AccountPlan::ScreenerPro);
    ///
    /// let result = value.parse::<AccountPlan>();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "free" => Ok(Self::Free),
            "screener_pro" => Ok(Self::ScreenerPro),
            "api_developer" => Ok(Self::ApiDeveloper),
            "api_business" => Ok(Self::ApiBusiness),
            _ => Err(InvalidAccountPlan::new(value)),
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
    const fn should_implement_auto_traits_for_account_plan() {
        implements_auto_traits::<AccountPlan>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_account_plan() {
        implements_value_traits::<AccountPlan>();
    }

    #[test]
    fn should_display_the_snake_case_name_for_every_plan() {
        let plans = [
            AccountPlan::Free,
            AccountPlan::ScreenerPro,
            AccountPlan::ApiDeveloper,
            AccountPlan::ApiBusiness,
        ];

        let expected_result = vec!["free", "screener_pro", "api_developer", "api_business"];

        let result: Vec<String> = plans.iter().map(ToString::to_string).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_the_displayed_text_back_to_the_same_plan_for_every_plan() {
        let plans = [
            AccountPlan::Free,
            AccountPlan::ScreenerPro,
            AccountPlan::ApiDeveloper,
            AccountPlan::ApiBusiness,
        ];

        let expected_result = vec![
            Ok(AccountPlan::Free),
            Ok(AccountPlan::ScreenerPro),
            Ok(AccountPlan::ApiDeveloper),
            Ok(AccountPlan::ApiBusiness),
        ];

        let result: Vec<_> = plans
            .iter()
            .map(|plan| plan.to_string().parse::<AccountPlan>())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_account_plan_when_the_string_is_the_marketing_name() {
        let value = "Screener Pro";

        let expected_result = Err(InvalidAccountPlan::new("Screener Pro"));

        let result = value.parse::<AccountPlan>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_account_plan_when_the_string_is_empty() {
        let value = "";

        let expected_result = Err(InvalidAccountPlan::new(""));

        let result = value.parse::<AccountPlan>();

        assert_eq!(result, expected_result);
    }
}
