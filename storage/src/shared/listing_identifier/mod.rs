//! # Listing Identifier
//!
//! Provides [`ListingIdentifier`], a `(scheme, value)` pair that names a listing, and
//! [`ListingScheme`], the schemes at the listing level.

use std::fmt::{self, Display, Formatter};

/// A scheme whose values name a listing.
///
/// Each scheme belongs to one level of identity: company, security, or listing. This enum holds
/// the listing schemes only, so a company or security scheme, such as `cik`, cannot reach a
/// [`ListingResolver`](crate::ListingResolver).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ListingScheme {
    /// The ticker symbol of a listing on a US exchange, as `dei:TradingSymbol` gives it.
    UsTicker,
    /// The Financial Instrument Global Identifier of one security on one exchange.
    Figi,
}

impl ListingScheme {
    /// Returns the name of the scheme as the identifier rows store it, such as `us_ticker`.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::UsTicker => "us_ticker",
            Self::Figi => "figi",
        }
    }
}

impl Display for ListingScheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A `(scheme, value)` pair that names a listing, such as `(us_ticker, META)`.
///
/// A resolver matches the value exactly, because grouping uses exact keys only. So the caller
/// passes the value in the form that the identifier rows hold. A value in another form, such as a
/// ticker in lower case, matches no row.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListingIdentifier {
    scheme: ListingScheme,
    value: String,
}

impl ListingIdentifier {
    /// Creates a [`ListingIdentifier`] from a scheme and a value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::{ListingIdentifier, ListingScheme};
    ///
    /// let value = "META";
    ///
    /// let expected_result = "us_ticker:META";
    ///
    /// let result = ListingIdentifier::new(ListingScheme::UsTicker, value).to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn new(scheme: ListingScheme, value: impl Into<String>) -> Self {
        Self {
            scheme,
            value: value.into(),
        }
    }

    /// Returns the scheme.
    #[must_use]
    pub const fn scheme(&self) -> ListingScheme {
        self.scheme
    }

    /// Returns the value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Display for ListingIdentifier {
    /// Writes the scheme and the value, separated by a colon, such as `figi:BBG000MM2P62`.
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
    const fn should_implement_auto_traits_for_listing_identifier() {
        implements_auto_traits::<ListingIdentifier>();
    }

    #[test]
    const fn should_implement_auto_traits_for_listing_scheme() {
        implements_auto_traits::<ListingScheme>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_listing_identifier() {
        implements_value_traits::<ListingIdentifier>();
    }

    #[test]
    const fn should_implement_value_traits_for_listing_scheme() {
        implements_value_traits::<ListingScheme>();
    }

    #[test]
    fn should_return_us_ticker_for_the_us_ticker_scheme() {
        let scheme = ListingScheme::UsTicker;

        let expected_result = "us_ticker";

        let result = scheme.as_str();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_figi_for_the_figi_scheme() {
        let scheme = ListingScheme::Figi;

        let expected_result = "figi";

        let result = scheme.as_str();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_scheme_for_a_listing_identifier() {
        let identifier = ListingIdentifier::new(ListingScheme::Figi, "BBG000MM2P62");

        let expected_result = ListingScheme::Figi;

        let result = identifier.scheme();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_value_for_a_listing_identifier() {
        let identifier = ListingIdentifier::new(ListingScheme::Figi, "BBG000MM2P62");

        let expected_result = "BBG000MM2P62";

        let result = identifier.value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_scheme_and_the_value_for_a_listing_identifier() {
        let identifier = ListingIdentifier::new(ListingScheme::Figi, "BBG000MM2P62");

        let expected_result = "figi:BBG000MM2P62";

        let result = identifier.to_string();

        assert_eq!(result, expected_result);
    }
}
