//! # Scope
//!
//! Provides [`Scope`], the part of a standard concept that a value covers when it covers less than
//! the whole.

use std::fmt;
use std::str::FromStr;

use super::invalid_value::{InvalidValue, InvalidValueReason};

/// The part of a standard concept that a value covers, if it covers less than the whole.
///
/// A value with no scope covers the full scope of its standard concept. An alias or a part of a
/// plug-in names the scope of each as-is concept that covers less.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
///
/// use xbrl::core::scope::Scope;
///
/// let input = "ExcludesRestrictedCash";
///
/// let expected_result = Ok(Scope::ExcludesRestrictedCash);
///
/// let result = Scope::from_str(input);
///
/// assert_eq!(result, expected_result);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Scope {
    /// A cash value that leaves out restricted cash, from a concept before ASU 2016-18, for
    /// example `CashAndCashEquivalentsPeriodIncreaseDecrease`.
    ExcludesRestrictedCash,
}

impl Scope {
    /// Every scope, in declaration order.
    const ALL: [Self; 1] = [Self::ExcludesRestrictedCash];
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExcludesRestrictedCash => write!(f, "ExcludesRestrictedCash"),
        }
    }
}

impl FromStr for Scope {
    type Err = InvalidValue;

    /// Parses the name that [`Scope`] displays, for example `ExcludesRestrictedCash`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] with [`InvalidValueReason::NoMatchingName`] if no scope has the
    /// input as its name.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|scope| scope.to_string() == input)
            .ok_or_else(|| {
                InvalidValue::new("Scope", InvalidValueReason::NoMatchingName("scope"), input)
            })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_scope() {
        implements_auto_traits::<Scope>();
    }

    #[test]
    fn should_display_the_variant_name_for_excludes_restricted_cash() {
        let scope = Scope::ExcludesRestrictedCash;

        let expected_result = "ExcludesRestrictedCash";

        let result = scope.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_each_scope_back_when_the_input_is_its_display_name() {
        let names: Vec<String> = Scope::ALL.iter().map(ToString::to_string).collect();

        let expected_result = vec![Ok(Scope::ExcludesRestrictedCash)];

        let result: Vec<Result<Scope, InvalidValue>> =
            names.iter().map(|name| Scope::from_str(name)).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_no_scope_has_the_input_as_its_name() {
        let input = "IncludesRestrictedCash";

        let expected_result = Err(InvalidValue::new(
            "Scope",
            InvalidValueReason::NoMatchingName("scope"),
            "IncludesRestrictedCash",
        ));

        let result = Scope::from_str(input);

        assert_eq!(result, expected_result);
    }
}
