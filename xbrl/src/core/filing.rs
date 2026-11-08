//! # Filing
//!
//! Provides [`Filing`], one filing with its attributes as filed, and the IDs it carries:
//! [`FilingId`] and [`EntityId`].

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};

use super::entity_name::EntityName;
use super::fiscal_period::FiscalPeriod;
use super::fiscal_year::FiscalYear;
use super::form::Form;
use super::invalid_value::{InvalidValue, InvalidValueReason, require_token};

/// The ID of a filing: a source and an ID within that source, for example
/// `sec:0000320193-26-000001`.
///
/// The text form is the same text as `source_ref` in the storage design. An extension concept of a
/// filing has this text as its namespace family.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
///
/// use xbrl::core::filing::FilingId;
///
/// let text = "sec:0000320193-26-000001";
///
/// let expected_result = "0000320193-26-000001";
///
/// let filing_id = FilingId::from_str(text).expect("Given a hardcoded ID with a source, the parse should always succeed");
/// let result = filing_id.id();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct FilingId {
    source: String,
    id: String,
}

impl FilingId {
    /// Creates a [`FilingId`] from a source, for example `sec`, and an ID within that source.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if a part is empty or holds whitespace, or if the source holds a `:`.
    pub fn new(source: impl Into<String>, id: impl Into<String>) -> Result<Self, InvalidValue> {
        let source = source.into();
        let id = id.into();
        require_token("FilingId", &source)?;
        require_token("FilingId", &id)?;
        if source.contains(':') {
            return Err(InvalidValue::new(
                "FilingId",
                InvalidValueReason::ContainsSeparator(':'),
                source,
            ));
        }
        Ok(Self { source, id })
    }

    /// Returns the source, for example `sec`.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the ID within the source, for example an SEC accession number.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl FromStr for FilingId {
    type Err = InvalidValue;

    /// Parses a [`FilingId`] from its text form `<source>:<id>`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the text has no `:` or a part is invalid.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (source, id) = text.split_once(':').ok_or_else(|| {
            InvalidValue::new("FilingId", InvalidValueReason::MissingSeparator(':'), text)
        })?;
        Self::new(source, id)
    }
}

impl fmt::Display for FilingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.source, self.id)
    }
}

/// The XBRL entity identifier: a scheme and an ID, for example the SEC CIK scheme
/// `http://www.sec.gov/CIK` and `0000320193`, or an LEI.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct EntityId {
    scheme: String,
    id: String,
}

impl EntityId {
    /// Creates an [`EntityId`] from a scheme and an ID.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if a part is empty or holds whitespace.
    pub fn new(scheme: impl Into<String>, id: impl Into<String>) -> Result<Self, InvalidValue> {
        let scheme = scheme.into();
        let id = id.into();
        require_token("EntityId", &scheme)?;
        require_token("EntityId", &id)?;
        Ok(Self { scheme, id })
    }

    /// Returns the scheme, for example `http://www.sec.gov/CIK`.
    #[must_use]
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    /// Returns the ID within the scheme, for example `0000320193`.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// One listing of a share class: a ticker symbol and the exchange, if the filing names one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Ticker {
    /// The ticker symbol, for example `AAPL`.
    pub symbol: String,
    /// The exchange, for example `NASDAQ`.
    pub exchange: Option<String>,
}

/// One filing and its attributes as filed.
///
/// A later filing with another company name or fiscal year end records a change over time, not a
/// conflict. Every field is public, because a filing is a record of what the source says, and each
/// field type checks its own rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Filing {
    /// The ID of the filing, for example `sec:0000320193-26-000001`.
    pub id: FilingId,
    /// The entity that filed it.
    pub entity: EntityId,
    /// The form type as filed, for example `10-K/A`.
    pub form: Form,
    /// Whether the filing amends an earlier filing.
    pub is_amendment: bool,
    /// The filing date.
    pub filed: NaiveDate,
    /// The acceptance time, if the source gives it.
    pub accepted: Option<DateTime<Utc>>,
    /// The end of the period the filing reports on, if the source gives it.
    pub period_of_report: Option<NaiveDate>,
    /// The fiscal year as filed.
    pub fiscal_year: Option<FiscalYear>,
    /// The fiscal period as filed.
    pub fiscal_period: Option<FiscalPeriod>,
    /// The company name as filed.
    pub company_name: Option<EntityName>,
    /// The SIC code as filed, for example `3571`.
    pub sic: Option<String>,
    /// One entry per listed share class.
    pub tickers: Vec<Ticker>,
    /// The fiscal year end as the source writes it, for example `0930` or `--09-27`.
    pub fiscal_year_end: Option<String>,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filing_id() {
        implements_auto_traits::<FilingId>();
    }

    #[test]
    const fn should_implement_auto_traits_for_entity_id() {
        implements_auto_traits::<EntityId>();
    }

    #[test]
    const fn should_implement_auto_traits_for_filing() {
        implements_auto_traits::<Filing>();
    }

    #[test]
    fn should_print_source_and_id_when_filing_id_is_displayed() {
        let filing_id = FilingId::new("sec", "0000320193-26-000001")
            .expect("Given a hardcoded source and ID, the creation should always succeed");

        let expected_result = "sec:0000320193-26-000001";

        let result = filing_id.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_source_when_filing_id_is_parsed() {
        let text = "esef:529900T8BM49AURSDO55-2025-12-31";

        let expected_result = "esef";

        let result = FilingId::from_str(text)
            .expect("Given a hardcoded ID with a source, the parse should always succeed")
            .source()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_text_when_filing_id_has_no_source() {
        let text = "0000320193-26-000001";

        let expected_result = Err(InvalidValue::new(
            "FilingId",
            InvalidValueReason::MissingSeparator(':'),
            "0000320193-26-000001",
        ));

        let result = FilingId::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_filing_id_when_id_is_empty() {
        let id = "";

        let expected_result = Err(InvalidValue::new("FilingId", InvalidValueReason::Empty, ""));

        let result = FilingId::new("sec", id);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_filing_id_when_source_holds_a_colon() {
        let source = "sec:x";

        let expected_result = Err(InvalidValue::new(
            "FilingId",
            InvalidValueReason::ContainsSeparator(':'),
            "sec:x",
        ));

        let result = FilingId::new(source, "1");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_scheme_when_entity_id_is_created() {
        let scheme = "http://www.sec.gov/CIK";

        let expected_result = "http://www.sec.gov/CIK";

        let entity = EntityId::new(scheme, "0000320193")
            .expect("Given a hardcoded scheme and ID, the creation should always succeed");
        let result = entity.scheme();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_entity_id_when_id_holds_whitespace() {
        let id = "0000 320193";

        let expected_result = Err(InvalidValue::new(
            "EntityId",
            InvalidValueReason::ContainsWhitespace,
            "0000 320193",
        ));

        let result = EntityId::new("http://www.sec.gov/CIK", id);

        assert_eq!(result, expected_result);
    }
}
