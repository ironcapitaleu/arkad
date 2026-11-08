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
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "FilingId";

    /// The character between the source and the ID in the text form.
    pub const SEPARATOR: char = ':';

    /// Creates a [`FilingId`] from a source, for example `sec`, and an ID within that source.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if a part is empty or holds whitespace, or if the source holds a `:`.
    pub fn new(source: impl Into<String>, id: impl Into<String>) -> Result<Self, InvalidValue> {
        let source = source.into();
        let id = id.into();
        require_token(Self::TYPE_NAME, &source)?;
        require_token(Self::TYPE_NAME, &id)?;
        if source.contains(Self::SEPARATOR) {
            return Err(InvalidValue::new(
                Self::TYPE_NAME,
                InvalidValueReason::ContainsSeparator(Self::SEPARATOR),
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
        let (source, id) = text.split_once(Self::SEPARATOR).ok_or_else(|| {
            InvalidValue::new(
                Self::TYPE_NAME,
                InvalidValueReason::MissingSeparator(Self::SEPARATOR),
                text,
            )
        })?;
        Self::new(source, id)
    }
}

impl fmt::Display for FilingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.source, Self::SEPARATOR, self.id)
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
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "EntityId";

    /// Creates an [`EntityId`] from a scheme and an ID.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if a part is empty or holds whitespace.
    pub fn new(scheme: impl Into<String>, id: impl Into<String>) -> Result<Self, InvalidValue> {
        let scheme = scheme.into();
        let id = id.into();
        require_token(Self::TYPE_NAME, &scheme)?;
        require_token(Self::TYPE_NAME, &id)?;
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

/// An industry classification code and the scheme it belongs to, for example `sic:3571` or
/// `nace:26.20`.
///
/// The set of schemes is open: the SEC uses SIC, and European and Korean filers use NACE and KSIC.
///
/// # Examples
///
/// ```
/// use xbrl::core::filing::IndustryCode;
///
/// let scheme = IndustryCode::SIC_SCHEME;
/// let code = "3571";
///
/// let expected_result = "sic:3571";
///
/// let industry = IndustryCode::new(scheme, code).expect("Given a hardcoded scheme and code, the creation should always succeed");
/// let result = industry.to_string();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct IndustryCode {
    scheme: String,
    code: String,
}

impl IndustryCode {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "IndustryCode";

    /// The character between the scheme and the code in the text form.
    pub const SEPARATOR: char = ':';

    /// The scheme of the Standard Industrial Classification, which the SEC uses.
    pub const SIC_SCHEME: &'static str = "sic";

    /// Creates an [`IndustryCode`] from a scheme, for example `sic`, and a code within it.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if a part is empty or holds whitespace, or if the scheme holds a `:`.
    pub fn new(scheme: impl Into<String>, code: impl Into<String>) -> Result<Self, InvalidValue> {
        let scheme = scheme.into();
        let code = code.into();
        require_token(Self::TYPE_NAME, &scheme)?;
        require_token(Self::TYPE_NAME, &code)?;
        if scheme.contains(Self::SEPARATOR) {
            return Err(InvalidValue::new(
                Self::TYPE_NAME,
                InvalidValueReason::ContainsSeparator(Self::SEPARATOR),
                scheme,
            ));
        }
        Ok(Self { scheme, code })
    }

    /// Returns the scheme, for example `sic`.
    #[must_use]
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    /// Returns the code within the scheme, for example `3571`.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }
}

impl fmt::Display for IndustryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.scheme, Self::SEPARATOR, self.code)
    }
}

/// The attributes of the company as one filing states them.
///
/// The attributes belong to the company, not to the filing. A filing records them as they were
/// when it was filed, so a later filing with another value records a change over time, not a
/// conflict.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize)]
pub struct CompanyAttributes {
    /// The company name.
    pub name: Option<EntityName>,
    /// The industry classification, for example `sic:3571`.
    pub industry: Option<IndustryCode>,
    /// One entry per listed share class.
    pub tickers: Vec<Ticker>,
    /// The fiscal year end as the source writes it, for example `0930` or `--09-27`.
    pub fiscal_year_end: Option<String>,
}

/// One filing and its attributes as filed.
///
/// Every field is public, because a filing is a record of what the source says, and each field
/// type checks its own rules.
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
    /// The attributes of the company as the filing states them.
    pub company: CompanyAttributes,
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
    const fn should_implement_auto_traits_for_industry_code() {
        implements_auto_traits::<IndustryCode>();
    }

    #[test]
    const fn should_implement_auto_traits_for_company_attributes() {
        implements_auto_traits::<CompanyAttributes>();
    }

    #[test]
    fn should_keep_code_when_scheme_is_not_sic() {
        let industry = IndustryCode::new("nace", "26.20")
            .expect("Given a hardcoded scheme and code, the creation should always succeed");

        let expected_result = "26.20";

        let result = industry.code();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_industry_code_when_code_is_empty() {
        let code = "";

        let expected_result = Err(InvalidValue::new(
            "IndustryCode",
            InvalidValueReason::Empty,
            "",
        ));

        let result = IndustryCode::new("sic", code);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_industry_code_when_scheme_holds_a_colon() {
        let scheme = "sic:2";

        let expected_result = Err(InvalidValue::new(
            "IndustryCode",
            InvalidValueReason::ContainsSeparator(':'),
            "sic:2",
        ));

        let result = IndustryCode::new(scheme, "3571");

        assert_eq!(result, expected_result);
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
