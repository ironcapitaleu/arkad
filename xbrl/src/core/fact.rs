//! # Fact
//!
//! Provides [`AsIsFact`], one reported value with the filing it came from, its value key
//! [`FactKey`], and the types of its fields.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};

use super::concept::ConceptName;
use super::filing::{EntityId, FilingId};
use super::invalid_value::{InvalidValue, InvalidValueReason, require_token};
use super::period::Period;
use super::unit::Unit;

/// The family and the year of a taxonomy release, for example `us-gaap/2017`, or the filing ID for an
/// extension concept.
///
/// This is the form of `version` in the Notes `tag` table. The instance parser writes the same form,
/// whether the namespace URI ends in `2017-01-31` or in `2017`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct TaxonomyVersion(String);

impl TaxonomyVersion {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "TaxonomyVersion";

    /// The character between the family and the year, as in `us-gaap/2017`. An extension version,
    /// which is a filing ID, has no separator.
    pub const SEPARATOR: char = '/';

    /// The number of digits of the year.
    pub const YEAR_LENGTH: usize = 4;

    /// Creates a [`TaxonomyVersion`] from its text.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the text is empty or holds whitespace.
    pub fn new(text: impl Into<String>) -> Result<Self, InvalidValue> {
        let text = text.into();
        require_token(Self::TYPE_NAME, &text)?;
        Ok(Self(text))
    }

    /// Returns the text of the version.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TaxonomyVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The axis and member pairs of a fact, sorted by axis. The legal entity axis is one of them.
///
/// The parsers write axis and member names without prefixes, as the SEC does. An empty set marks a
/// consolidated fact.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Dimensions(BTreeMap<String, String>);

impl Dimensions {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "Dimensions";

    /// Creates [`Dimensions`] from axis and member pairs in any order.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if an axis or a member is empty, or if two pairs name one axis.
    pub fn new<I, A, M>(pairs: I) -> Result<Self, InvalidValue>
    where
        I: IntoIterator<Item = (A, M)>,
        A: Into<String>,
        M: Into<String>,
    {
        let mut map = BTreeMap::new();
        for (axis, member) in pairs {
            let axis = axis.into();
            let member = member.into();
            if axis.is_empty() || member.is_empty() {
                return Err(InvalidValue::new(
                    Self::TYPE_NAME,
                    InvalidValueReason::Empty,
                    format!("{axis}={member}"),
                ));
            }
            if map.contains_key(&axis) {
                return Err(InvalidValue::new(
                    Self::TYPE_NAME,
                    InvalidValueReason::DuplicateAxis,
                    axis,
                ));
            }
            map.insert(axis, member);
        }
        Ok(Self(map))
    }

    /// Returns `true` if the fact has no dimensions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the pairs, sorted by axis.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .iter()
            .map(|(axis, member)| (axis.as_str(), member.as_str()))
    }
}

/// The value of a fact: a decimal number, a text, or nil.
///
/// A decimal holds every digit the source writes, with no limit on the scale. Measured values have
/// at most 24 digits before the point and at most 8 after it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum FactValue {
    /// A numeric value, for example `391035000000` or `6.11`.
    Numeric(BigDecimal),
    /// A text value, for example a company name or a text block.
    Text(String),
    /// A fact the filer reports as nil (`xsi:nil`), or with an empty value in the Notes data set.
    Nil,
}

/// The precision of a numeric value: a finite number of decimal places, or infinite.
///
/// `-6` means the value is exact to millions. `2` means it is exact to cents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Decimals {
    /// The value is exact to this many places after the point. A negative number counts places
    /// before the point.
    Finite(i32),
    /// The value is exact.
    Infinite,
}

impl Decimals {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "Decimals";

    /// The text of the XBRL `decimals` attribute for an exact value.
    pub const INFINITE_TEXT: &'static str = "INF";
}

impl FromStr for Decimals {
    type Err = InvalidValue;

    /// Parses the XBRL `decimals` attribute: `INF` or an integer.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] with [`InvalidValueReason::NotANumber`] if the text is neither.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use xbrl::core::fact::Decimals;
    ///
    /// let text = "-6";
    ///
    /// let expected_result = Ok(Decimals::Finite(-6));
    ///
    /// let result = Decimals::from_str(text);
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text == Self::INFINITE_TEXT {
            return Ok(Self::Infinite);
        }
        text.parse()
            .map(Self::Finite)
            .map_err(|_| InvalidValue::new(Self::TYPE_NAME, InvalidValueReason::NotANumber, text))
    }
}

/// Where a fact comes from: the filing and when it was filed.
///
/// A point-in-time read filters on `filed` and `accepted`. Both repeat the values of the
/// [`Filing`](super::filing::Filing), so the read needs no join.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Source {
    /// The filing, for example `sec:0000320193-26-000001`.
    pub filing: FilingId,
    /// The filing date.
    pub filed: NaiveDate,
    /// The acceptance time, if the source gives it.
    pub accepted: Option<DateTime<Utc>>,
}

/// What a fact is about: the entity, the period, and the axis and member pairs.
///
/// This is the context of a fact, as XBRL defines it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Context {
    /// The entity the fact is about.
    pub entity: EntityId,
    /// The instant or duration the fact covers.
    pub period: Period,
    /// The axis and member pairs. Empty for a consolidated fact.
    pub dimensions: Dimensions,
}

/// One reported value with the filing it came from, as the source reports it.
///
/// The as-is layer holds every value of every filing, with no mapping to standard concepts and no
/// dedup, so a point-in-time read stays possible. The ingestion provenance, such as the source
/// document and its SHA-256, is not a field: silver stores it next to the fact.
///
/// Every field is public, because a fact is a record of what the source says, and each field type
/// checks its own rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct AsIsFact {
    /// Where the fact comes from.
    pub source: Source,
    /// What the fact is about.
    pub context: Context,
    /// The concept: a namespace family and a local name.
    pub concept: ConceptName,
    /// The release of the family, for example `us-gaap/2026`. The filing ID for an extension
    /// concept. `None` if the source gives no version, as the SEC JSON APIs do.
    pub taxonomy_version: Option<TaxonomyVersion>,
    /// The unit. `None` for a text fact.
    pub unit: Option<Unit>,
    /// The value.
    pub value: FactValue,
    /// The precision of a numeric value. `None` for text, and if the source gives none.
    pub decimals: Option<Decimals>,
    /// The rank of the copy among facts with the same key in one filing. 0 for the first copy.
    ///
    /// Two facts with different exact starts can share a key, because the length in quarters is
    /// rounded. The dedup of the standardized layer decides whether two ranks are one fact.
    pub copy_rank: u16,
    /// The footnotes the filing attaches to the fact.
    pub footnotes: Vec<String>,
}

impl AsIsFact {
    /// Returns the value key of the fact.
    #[must_use]
    pub fn key(&self) -> FactKey {
        FactKey {
            filing: self.source.filing.clone(),
            concept: self.concept.clone(),
            taxonomy_version: self.taxonomy_version.clone(),
            period_end: self.context.period.end(),
            quarters: self.context.period.quarters(),
            unit: self.unit.clone(),
            dimensions: self.context.dimensions.clone(),
            copy_rank: self.copy_rank,
        }
    }
}

/// The value key of an [`AsIsFact`]: unique within one filing, and the same for a Notes row and an
/// instance fact of one filing.
///
/// The key holds the exact end and the length in quarters, not the exact start, because the Notes
/// data set often gives no exact start. So two durations with one end and one rounded length, such
/// as a 61-day stub and a 91-day quarter, differ only in `copy_rank`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct FactKey {
    /// The filing.
    pub filing: FilingId,
    /// The concept.
    pub concept: ConceptName,
    /// The release of the family.
    pub taxonomy_version: Option<TaxonomyVersion>,
    /// The exact end of the period.
    pub period_end: NaiveDate,
    /// The length in quarters. 0 for an instant.
    pub quarters: u16,
    /// The unit.
    pub unit: Option<Unit>,
    /// The axis and member pairs.
    pub dimensions: Dimensions,
    /// The rank of the copy.
    pub copy_rank: u16,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::concept::NamespaceFamily;
    use crate::core::period::Duration;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded valid date, the creation should always succeed")
    }

    fn filing_id() -> FilingId {
        FilingId::new("sec", "0000320193-24-000123")
            .expect("Given a hardcoded source and ID, the creation should always succeed")
    }

    fn concept(local_name: &str) -> ConceptName {
        let family = NamespaceFamily::new("us-gaap").expect(
            "Given a hardcoded family without whitespace, the creation should always succeed",
        );
        ConceptName::new(family, local_name).expect(
            "Given a hardcoded local name without a colon, the creation should always succeed",
        )
    }

    fn unit_usd() -> Unit {
        Unit::from_str("USD")
            .expect("Given a hardcoded unit with one measure, the parse should always succeed")
    }

    fn decimal(text: &str) -> BigDecimal {
        BigDecimal::from_str(text)
            .expect("Given a hardcoded decimal text, the parse should always succeed")
    }

    fn revenue_fact() -> AsIsFact {
        AsIsFact {
            source: Source {
                filing: filing_id(),
                filed: date(2024, 11, 1),
                accepted: None,
            },
            context: Context {
                entity: EntityId::new("http://www.sec.gov/CIK", "0000320193")
                    .expect("Given a hardcoded scheme and ID, the creation should always succeed"),
                period: Period::Duration(
                    Duration::from_dates(date(2023, 10, 1), date(2024, 9, 28))
                        .expect("Given an end after the start, the creation should always succeed"),
                ),
                dimensions: Dimensions::default(),
            },
            concept: concept("Revenues"),
            taxonomy_version: Some(TaxonomyVersion::new("us-gaap/2024").expect(
                "Given a hardcoded version without whitespace, the creation should always succeed",
            )),
            unit: Some(unit_usd()),
            value: FactValue::Numeric(decimal("391035000000")),
            decimals: Some(Decimals::Finite(-6)),
            copy_rank: 0,
            footnotes: vec![],
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_as_is_fact() {
        implements_auto_traits::<AsIsFact>();
    }

    #[test]
    const fn should_implement_auto_traits_for_source() {
        implements_auto_traits::<Source>();
    }

    #[test]
    const fn should_implement_auto_traits_for_context() {
        implements_auto_traits::<Context>();
    }

    #[test]
    const fn should_implement_auto_traits_for_fact_key() {
        implements_auto_traits::<FactKey>();
    }

    #[test]
    const fn should_implement_auto_traits_for_fact_value() {
        implements_auto_traits::<FactValue>();
    }

    #[test]
    const fn should_implement_auto_traits_for_dimensions() {
        implements_auto_traits::<Dimensions>();
    }

    #[test]
    fn should_build_key_from_end_and_quarters_when_fact_is_a_duration() {
        let fact = revenue_fact();

        let expected_result = FactKey {
            filing: filing_id(),
            concept: concept("Revenues"),
            taxonomy_version: Some(TaxonomyVersion::new("us-gaap/2024").expect(
                "Given a hardcoded version without whitespace, the creation should always succeed",
            )),
            period_end: date(2024, 9, 28),
            quarters: 4,
            unit: Some(unit_usd()),
            dimensions: Dimensions::default(),
            copy_rank: 0,
        };

        let result = fact.key();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_one_key_when_two_sources_differ_only_in_the_start() {
        let instance_fact = revenue_fact();
        let instance_context = revenue_fact().context;
        let notes_fact = AsIsFact {
            context: Context {
                period: Period::Duration(Duration::from_end_and_quarters(date(2024, 9, 28), 4)),
                ..instance_context
            },
            ..revenue_fact()
        };

        let expected_result = instance_fact.key();

        let result = notes_fact.key();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_eight_places_when_decimal_has_twenty_four_digits_before_the_point() {
        let text = "123456789012345678901234.12345678";

        let expected_result = "123456789012345678901234.12345678";

        let result = decimal(text).to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_infinite_when_decimals_is_inf() {
        let text = "INF";

        let expected_result = Ok(Decimals::Infinite);

        let result = Decimals::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_decimals_when_text_is_not_a_number() {
        let text = "six";

        let expected_result = Err(InvalidValue::new(
            "Decimals",
            InvalidValueReason::NotANumber,
            "six",
        ));

        let result = Decimals::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_sort_pairs_by_axis_when_dimensions_are_created() {
        let pairs = [
            ("StatementGeographicalAxis", "US"),
            ("LegalEntityAxis", "Subsidiary"),
        ];

        let expected_result = vec![
            ("LegalEntityAxis", "Subsidiary"),
            ("StatementGeographicalAxis", "US"),
        ];

        let dimensions = Dimensions::new(pairs)
            .expect("Given two distinct axes, the creation should always succeed");
        let result: Vec<(&str, &str)> = dimensions.iter().collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_dimensions_when_two_pairs_name_one_axis() {
        let pairs = [("LegalEntityAxis", "A"), ("LegalEntityAxis", "B")];

        let expected_result = Err(InvalidValue::new(
            "Dimensions",
            InvalidValueReason::DuplicateAxis,
            "LegalEntityAxis",
        ));

        let result = Dimensions::new(pairs);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_dimensions_when_an_axis_is_empty() {
        let pairs = [("", "Subsidiary")];

        let expected_result = Err(InvalidValue::new(
            "Dimensions",
            InvalidValueReason::Empty,
            "=Subsidiary",
        ));

        let result = Dimensions::new(pairs);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_taxonomy_version_when_text_is_empty() {
        let text = "";

        let expected_result = Err(InvalidValue::new(
            "TaxonomyVersion",
            InvalidValueReason::Empty,
            "",
        ));

        let result = TaxonomyVersion::new(text);

        assert_eq!(result, expected_result);
    }
}
