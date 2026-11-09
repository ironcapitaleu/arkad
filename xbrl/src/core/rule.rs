//! # Rule
//!
//! Provides [`RuleId`], the ID of the rule that produced a standardized value.

use std::fmt;
use std::str::FromStr;

use super::invalid_value::{InvalidValue, InvalidValueReason, require_token};

/// The ID of a rule in the form `<owner>/<kind>/<name>@<version>`, for example
/// `us-gaap/alias/revenue@1` or `core/dedup/newest-filing@1`.
///
/// A new taxonomy release does not change the version. `xbrl/DESIGN.md`, "Rule IDs", says when
/// it goes up.
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
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "RuleId";

    /// The character between the path and the version, as in `revenue@1`.
    pub const VERSION_SEPARATOR: char = '@';

    /// The character between the owner, the kind, and the name.
    pub const PATH_SEPARATOR: char = '/';

    /// The number of parts of the path: owner, kind, and name.
    pub const PATH_PARTS: usize = 3;

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
    /// Returns [`InvalidValue`] if the text holds whitespace, lacks `@`, does not hold exactly two `/`, has
    /// an empty part, or has a version that is not plain digits without a leading zero.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        require_token(Self::TYPE_NAME, text)?;
        let invalid = |reason| InvalidValue::new(Self::TYPE_NAME, reason, text);
        let (path, version) = text.rsplit_once(Self::VERSION_SEPARATOR).ok_or_else(|| {
            invalid(InvalidValueReason::MissingSeparator(
                Self::VERSION_SEPARATOR,
            ))
        })?;
        let parts: Vec<&str> = path.split(Self::PATH_SEPARATOR).collect();
        if parts.len() < Self::PATH_PARTS {
            return Err(invalid(InvalidValueReason::MissingSeparator(
                Self::PATH_SEPARATOR,
            )));
        }
        if parts.len() > Self::PATH_PARTS {
            return Err(invalid(InvalidValueReason::ContainsSeparator(
                Self::PATH_SEPARATOR,
            )));
        }
        if parts.iter().any(|part| part.is_empty()) {
            return Err(invalid(InvalidValueReason::Empty));
        }
        // Only plain digits without a leading zero, so one version has one spelling.
        let is_plain = !version.is_empty()
            && version.bytes().all(|byte| byte.is_ascii_digit())
            && !(version.len() > 1 && version.starts_with('0'));
        if !is_plain {
            return Err(invalid(InvalidValueReason::NotANumber));
        }
        let version = version
            .parse()
            .map_err(|_| invalid(InvalidValueReason::OutOfRange))?;
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
        let text = "core/dedup/newest-filing@1";

        let expected_result = "core/dedup/newest-filing@1";

        let result = RuleId::from_str(text)
            .expect(
                "Given a hardcoded rule ID in the documented form, the parse should always succeed",
            )
            .to_string();

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
    fn should_reject_text_when_path_has_four_parts() {
        let text = "a/b/c/d@1";

        let expected_result = Err(InvalidValue::new(
            "RuleId",
            InvalidValueReason::ContainsSeparator('/'),
            "a/b/c/d@1",
        ));

        let result = RuleId::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_text_when_version_has_a_leading_zero() {
        let text = "us-gaap/alias/revenue@01";

        let expected_result = Err(InvalidValue::new(
            "RuleId",
            InvalidValueReason::NotANumber,
            "us-gaap/alias/revenue@01",
        ));

        let result = RuleId::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_text_when_version_has_a_plus_sign() {
        let text = "us-gaap/alias/revenue@+1";

        let expected_result = Err(InvalidValue::new(
            "RuleId",
            InvalidValueReason::NotANumber,
            "us-gaap/alias/revenue@+1",
        ));

        let result = RuleId::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_text_when_version_exceeds_thirty_two_bits() {
        let text = "us-gaap/alias/revenue@4294967296";

        let expected_result = Err(InvalidValue::new(
            "RuleId",
            InvalidValueReason::OutOfRange,
            "us-gaap/alias/revenue@4294967296",
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
