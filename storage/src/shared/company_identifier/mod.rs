//! # Company Identifier
//!
//! Provides [`CompanyIdentifier`], a `(scheme, value)` pair that names a company, and
//! [`CompanyScheme`], the schemes at the company level.

use std::fmt::{self, Display, Formatter};

/// A scheme whose values name a company.
///
/// Each scheme belongs to one level of identity (ADR 0009, Decision 2). This enum holds the
/// company schemes only, so a security or listing scheme, such as `us_ticker`, cannot reach an
/// [`EntityResolver`](crate::EntityResolver).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompanyScheme {
    /// The Central Index Key that the SEC gives a registrant.
    Cik,
    /// The Legal Entity Identifier of ISO 17442.
    Lei,
}

impl CompanyScheme {
    /// Returns the name of the scheme as the identifier rows store it, such as `cik`.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Cik => "cik",
            Self::Lei => "lei",
        }
    }
}

impl Display for CompanyScheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A `(scheme, value)` pair that names a company, such as `(cik, 0001652044)`.
///
/// A resolver matches the value exactly, because grouping uses exact keys only (ADR 0001, "Design
/// Rules" 3). So the caller passes the value in the form that the identifier rows hold. A value in
/// another form, such as a CIK without its leading zeros, matches no row. The `sec` crate converts
/// a validated `Cik` into a [`CompanyIdentifier`] with the 10-digit value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompanyIdentifier {
    scheme: CompanyScheme,
    value: String,
}

impl CompanyIdentifier {
    /// Creates a [`CompanyIdentifier`] from a scheme and a value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::{CompanyIdentifier, CompanyScheme};
    ///
    /// let value = "0001652044";
    ///
    /// let expected_result = "cik:0001652044";
    ///
    /// let result = CompanyIdentifier::new(CompanyScheme::Cik, value).to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn new(scheme: CompanyScheme, value: impl Into<String>) -> Self {
        Self {
            scheme,
            value: value.into(),
        }
    }

    /// Returns the scheme.
    #[must_use]
    pub const fn scheme(&self) -> CompanyScheme {
        self.scheme
    }

    /// Returns the value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Display for CompanyIdentifier {
    /// Writes the scheme and the value, separated by a colon, such as `lei:5493006MHB84DD0ZWV18`.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.scheme, self.value)
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
    const fn should_implement_auto_traits_for_company_identifier() {
        implements_auto_traits::<CompanyIdentifier>();
    }

    #[test]
    const fn should_implement_auto_traits_for_company_scheme() {
        implements_auto_traits::<CompanyScheme>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_company_identifier() {
        implements_value_traits::<CompanyIdentifier>();
    }

    #[test]
    const fn should_implement_value_traits_for_company_scheme() {
        implements_value_traits::<CompanyScheme>();
    }

    #[test]
    fn should_return_cik_for_the_cik_scheme() {
        let scheme = CompanyScheme::Cik;

        let expected_result = "cik";

        let result = scheme.as_str();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_lei_for_the_lei_scheme() {
        let scheme = CompanyScheme::Lei;

        let expected_result = "lei";

        let result = scheme.as_str();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_scheme_for_a_company_identifier() {
        let identifier = CompanyIdentifier::new(CompanyScheme::Lei, "5493006MHB84DD0ZWV18");

        let expected_result = CompanyScheme::Lei;

        let result = identifier.scheme();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_value_for_a_company_identifier() {
        let identifier = CompanyIdentifier::new(CompanyScheme::Lei, "5493006MHB84DD0ZWV18");

        let expected_result = "5493006MHB84DD0ZWV18";

        let result = identifier.value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_scheme_and_the_value_for_a_company_identifier() {
        let identifier = CompanyIdentifier::new(CompanyScheme::Lei, "5493006MHB84DD0ZWV18");

        let expected_result = "lei:5493006MHB84DD0ZWV18";

        let result = identifier.to_string();

        assert_eq!(result, expected_result);
    }
}
