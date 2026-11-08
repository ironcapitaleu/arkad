//! # Rule
//!
//! Provides [`RuleId`], the ID of the rule that produced a standardized value.

use std::fmt;
use std::str::FromStr;

use super::invalid_value::{InvalidValue, InvalidValueReason, require_token};

/// The ID of a rule in the form `<owner>/<kind>/<name>@<version>`, for example
/// `us-gaap/alias/revenue@1` or `core/dedup/newest-filing@1`.
///
/// The version goes up when the rule itself changes. A new taxonomy release does not change it.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
///
/// use xbrl::core::rule::RuleId;
///
/// let text = "us-gaap/alias/revenue@1";
///
/// let expected_result = 1;
///
/// let rule = RuleId::from_str(text).expect("Given a hardcoded rule ID in the documented form, the parse should always succeed");
/// let result = rule.version();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct RuleId {
    text: String,
    version: u32,
}

impl RuleId {
    /// Returns the version of the rule.
    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    /// Returns the text of the ID.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl FromStr for RuleId {
    type Err = InvalidValue;

    /// Parses a [`RuleId`] from its text form.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the text holds whitespace, lacks `@` or one of the two `/`, has
    /// an empty part, or has a version that is not a number.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        require_token("RuleId", text)?;
        let invalid = |reason| InvalidValue::new("RuleId", reason, text);
        let (path, version) = text
            .rsplit_once('@')
            .ok_or_else(|| invalid(InvalidValueReason::MissingSeparator('@')))?;
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() != 3 {
            return Err(invalid(InvalidValueReason::MissingSeparator('/')));
        }
        if parts.iter().any(|part| part.is_empty()) {
            return Err(invalid(InvalidValueReason::Empty));
        }
        let version = version
            .parse()
            .map_err(|_| invalid(InvalidValueReason::NotANumber))?;
        Ok(Self {
            text: text.to_string(),
            version,
        })
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_rule_id() {
        implements_auto_traits::<RuleId>();
    }

    #[test]
    fn should_print_text_when_rule_id_is_displayed() {
        let rule = RuleId::from_str("core/dedup/newest-filing@1").expect(
            "Given a hardcoded rule ID in the documented form, the parse should always succeed",
        );

        let expected_result = "core/dedup/newest-filing@1";

        let result = rule.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_text_when_version_is_missing() {
        let text = "us-gaap/alias/revenue";

        let expected_result = Err(InvalidValue::new(
            "RuleId",
            InvalidValueReason::MissingSeparator('@'),
            "us-gaap/alias/revenue",
        ));

        let result = RuleId::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_text_when_kind_is_missing() {
        let text = "us-gaap/revenue@1";

        let expected_result = Err(InvalidValue::new(
            "RuleId",
            InvalidValueReason::MissingSeparator('/'),
            "us-gaap/revenue@1",
        ));

        let result = RuleId::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_text_when_version_is_not_a_number() {
        let text = "us-gaap/alias/revenue@one";

        let expected_result = Err(InvalidValue::new(
            "RuleId",
            InvalidValueReason::NotANumber,
            "us-gaap/alias/revenue@one",
        ));

        let result = RuleId::from_str(text);

        assert_eq!(result, expected_result);
    }
}
