//! # Notes Rows
//!
//! Provides one row type per table of the Notes data set that the parser reads, with the columns
//! it needs, and maps each column into a core type.
//!
//! The transform fills the rows with the text of the TSV fields, so each field is a `String`. The
//! methods here apply the rules of the Notes data set, for example that `dcml` 32767 means an exact
//! value.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use bigdecimal::{BigDecimal, Zero};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeDelta, TimeZone, Utc};
use chrono_tz::America::New_York;

use crate::adapter::edgar::company_facts::{CIK_SCHEME, SEC_FILING_SOURCE};
use crate::core::entity_name::EntityName;
use crate::core::fact::{Decimals, Dimensions, FactValue};
use crate::core::filing::{CompanyAttributes, EntityId, Filing, FilingId, IndustryCode};
use crate::core::fiscal_period::FiscalPeriod;
use crate::core::fiscal_year::FiscalYear;
use crate::core::form::Form;
use crate::core::invalid_value::{InvalidValue, InvalidValueReason};
use crate::core::period::{Duration, Period};
use crate::core::unit::Unit;

/// The format of the dates of the Notes data set, for example `20260930`.
pub const DATE_FORMAT: &str = "%Y%m%d";

/// The format of `sub.accepted`, for example `2026-10-29 16:15:00.0`, in the time of New York.
pub const ACCEPTED_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.f";

/// The name of the table of the `dim` rows.
pub const DIM_TABLE: &str = "dim.tsv";

/// The name of the table of the `num` rows.
pub const NUM_TABLE: &str = "num.tsv";

/// The name of the table of the `txt` rows.
pub const TXT_TABLE: &str = "txt.tsv";

/// The `srclen` of a nil text value.
pub const NIL_SOURCE_LENGTH: &str = "0";

/// The `dcml` of an exact value.
pub const INFINITE_DCML: &str = "32767";

/// The `dcml` of a nil value, which has no precision.
pub const NIL_DCML: &str = "-32768";

/// The `datatype` of a per-share tag in `tag.tsv`.
pub const PER_SHARE_DATA_TYPE: &str = "perShare";

/// The measure that `num.uom` drops from a per-share unit.
pub const SHARES_MEASURE: &str = "shares";

/// The field of a `dim` row that the SEC cuts to 1024 characters.
pub const SEGMENTS_FIELD: &str = "dim.segments";

/// The `segt` of a `dim` row whose `segments` the SEC cut to 1024 characters.
pub const TRUNCATED_SEGMENTS: &str = "1";

/// The axis that `coreg` names a member of, as `dim.tsv` writes it.
pub const LEGAL_ENTITY_AXIS: &str = "LegalEntity";

/// The character after each pair of `dim.segments`, as in `Axis=Member;`.
const SEGMENT_TERMINATOR: char = ';';

/// The character between the axis and the member of a pair of `dim.segments`.
const PAIR_SEPARATOR: char = '=';

/// The character between a prefix and a local name, as in `us-gaap:CostOfRevenue`.
const PREFIX_SEPARATOR: char = ':';

/// The characters other than letters and digits that a prefix or a local name can hold.
const NAME_PUNCTUATION: [char; 3] = ['_', '-', '.'];

/// The character between the whole and the fractional part of `datp`, as in `-15.0`.
const DECIMAL_POINT: char = '.';

/// A whole `datp` has only this digit after the point.
const ZERO_DIGIT: char = '0';

/// The character between the fields of a row in [`fmt::Display`].
const FIELD_SEPARATOR: &str = " ";

/// The row of `sub.tsv` of one filing, with the columns the parser needs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubRow {
    /// The accession number, for example `0000320193-26-000001`.
    pub adsh: String,
    /// The CIK without leading zeros.
    pub cik: String,
    /// The company name.
    pub name: String,
    /// The SIC code. Empty if the SEC has none.
    pub sic: String,
    /// The fiscal year end as `MMDD`, for example `0930`.
    pub fye: String,
    /// The form type, for example `10-Q`.
    pub form: String,
    /// The balance sheet date, rounded to a month end, as `YYYYMMDD`.
    pub period: String,
    /// The fiscal year, for example `2026`.
    pub fy: String,
    /// The fiscal period, for example `Q3`.
    pub fp: String,
    /// The filing date as `YYYYMMDD`.
    pub filed: String,
    /// The acceptance time in the time of New York, rounded to the minute.
    pub accepted: String,
}

impl SubRow {
    /// The type name that [`InvalidValue`] names when a field breaks a rule.
    const TYPE_NAME: &'static str = "SubRow";

    /// Returns the ID of the filing, for example `sec:0000320193-26-000001`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if `adsh` is empty or holds whitespace.
    pub fn filing_id(&self) -> Result<FilingId, InvalidValue> {
        FilingId::new(SEC_FILING_SOURCE, &self.adsh)
    }

    /// Maps the row into a [`Filing`].
    ///
    /// `period` becomes the period of report, as the month end that `sub.tsv` writes. `accepted`
    /// moves from the time of New York to UTC.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if a field breaks a rule of its core type, or if a date, a year, or
    /// the CIK does not parse.
    pub fn filing(&self) -> Result<Filing, InvalidValue> {
        let cik: u64 = self.cik.parse().map_err(|_| {
            InvalidValue::new(Self::TYPE_NAME, InvalidValueReason::NotANumber, &self.cik)
        })?;
        let form = Form::new(&self.form)?;
        Ok(Filing {
            id: self.filing_id()?,
            entity: EntityId::new(CIK_SCHEME, format!("{cik:010}"))?,
            is_amendment: form.is_amendment(),
            form,
            filed: date(Self::TYPE_NAME, &self.filed)?,
            accepted: optional(&self.accepted, Self::accepted_utc)?,
            period_of_report: optional(&self.period, |text| date(Self::TYPE_NAME, text))?,
            fiscal_year: optional(&self.fy, |text| {
                text.parse().map(FiscalYear::new).map_err(|_| {
                    InvalidValue::new(Self::TYPE_NAME, InvalidValueReason::NotANumber, text)
                })
            })?,
            fiscal_period: optional(&self.fp, |text| FiscalPeriod::new(text))?,
            company: CompanyAttributes {
                name: optional(&self.name, |text| Ok(EntityName::new(text)))?,
                industry: optional(&self.sic, |text| {
                    IndustryCode::new(IndustryCode::SIC_SCHEME, text)
                })?,
                tickers: Vec::new(),
                fiscal_year_end: optional(&self.fye, |text| Ok(text.to_owned()))?,
            },
        })
    }

    fn accepted_utc(text: &str) -> Result<DateTime<Utc>, InvalidValue> {
        let local = NaiveDateTime::parse_from_str(text, ACCEPTED_FORMAT).map_err(|_| {
            InvalidValue::new(
                Self::TYPE_NAME,
                InvalidValueReason::NotADate(ACCEPTED_FORMAT),
                text,
            )
        })?;
        New_York
            .from_local_datetime(&local)
            .earliest()
            .map(|time| time.with_timezone(&Utc))
            .ok_or_else(|| InvalidValue::new(Self::TYPE_NAME, InvalidValueReason::OutOfRange, text))
    }
}

/// A row of `num.tsv`: one numeric value of one filing, with the columns the parser needs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NumRow {
    /// The local name of the concept, for example `Revenues`.
    pub tag: String,
    /// The taxonomy version, for example `us-gaap/2026`, or the accession number for an extension.
    pub version: String,
    /// The end date, rounded to a month end, as `YYYYMMDD`.
    pub ddate: String,
    /// The length in quarters. `0` for an instant.
    pub qtrs: String,
    /// The unit, without `/shares` on a per-share unit.
    pub uom: String,
    /// The hash of the dimensions, the key of a `dim` row.
    pub dimh: String,
    /// The index of the copy among rows with one primary key.
    pub iprx: String,
    /// The value. Empty for a nil fact.
    pub value: String,
    /// The footnote text. Empty if there is none.
    pub footnote: String,
    /// The member of the legal entity axis. Empty for the registrant.
    pub coreg: String,
    /// `qtrs` minus the length of the fact in quarters, as in `-0.317808`. `0.0` for an instant.
    pub durp: String,
    /// The days from the exact end to `ddate`, as in `-15.0`.
    pub datp: String,
    /// The `decimals` of the value. `32767` for an exact value, `-32768` for a nil value.
    pub dcml: String,
}

impl NumRow {
    /// The type name that [`InvalidValue`] names when a field breaks a rule.
    const TYPE_NAME: &'static str = "NumRow";

    /// Returns the period: the exact end is `ddate` minus `datp` days.
    ///
    /// The SEC rounds `qtrs`, so a duration under about 45 days has `qtrs` 0. Its `durp` is not 0,
    /// so it becomes a duration of 0 quarters. Only a row with `qtrs` 0 and `durp` 0 is an instant.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if `ddate` is not a date, `datp` is not a whole number of days,
    /// `qtrs` is not a number of 16 bits, or `qtrs` is 0 and `durp` is not a number.
    pub fn period(&self) -> Result<Period, InvalidValue> {
        period(
            Self::TYPE_NAME,
            &self.ddate,
            &self.qtrs,
            &self.durp,
            &self.datp,
        )
    }

    /// Returns the unit, with `/shares` restored if the tag is a per-share tag and `uom` divides by
    /// nothing.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if `uom` is not a unit.
    pub fn unit(&self, per_share: bool) -> Result<Unit, InvalidValue> {
        if per_share && !self.uom.contains(Unit::DIVISION) {
            Unit::from_str(&format!("{}{}{SHARES_MEASURE}", self.uom, Unit::DIVISION))
        } else {
            Unit::from_str(&self.uom)
        }
    }

    /// Returns the value: a number, or nil if the field is empty.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if `value` is neither empty nor a number.
    pub fn value(&self) -> Result<FactValue, InvalidValue> {
        if self.value.is_empty() {
            return Ok(FactValue::Nil);
        }
        BigDecimal::from_str(&self.value)
            .map(FactValue::Numeric)
            .map_err(|_| Self::invalid(InvalidValueReason::NotANumber, &self.value))
    }

    /// Returns the precision: `Infinite` for `32767`, and `None` for `-32768` or an empty field.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if `dcml` is another text that is not a number of 32 bits.
    pub fn decimals(&self) -> Result<Option<Decimals>, InvalidValue> {
        match self.dcml.as_str() {
            INFINITE_DCML => Ok(Some(Decimals::Infinite)),
            text if text.is_empty() || text == NIL_DCML => Ok(None),
            text => text
                .parse()
                .map(|places| Some(Decimals::Finite(places)))
                .map_err(|_| Self::invalid(InvalidValueReason::NotANumber, text)),
        }
    }

    /// Returns the copy index `iprx`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if `iprx` is not a number of 32 bits.
    pub fn copy_index(&self) -> Result<u32, InvalidValue> {
        copy_index(Self::TYPE_NAME, &self.iprx)
    }

    /// Returns the footnote as a list with one entry, or an empty list.
    #[must_use]
    pub fn footnotes(&self) -> Vec<String> {
        footnotes(&self.footnote)
    }

    /// Returns the dimensions: the pairs of the `dim` row that `dimh` names, and the legal entity
    /// pair of `coreg` if that row lacks it.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if no `dim` row has the key `dimh`, a pair has no `=`, or two pairs
    /// name one axis.
    pub fn dimensions(&self, dims: &HashMap<&str, &DimRow>) -> Result<Dimensions, InvalidValue> {
        dimensions(Self::TYPE_NAME, &self.dimh, &self.coreg, dims)
    }

    fn invalid(reason: InvalidValueReason, input: &str) -> InvalidValue {
        InvalidValue::new(Self::TYPE_NAME, reason, input)
    }
}

impl fmt::Display for NumRow {
    /// Writes the table and the primary key of the row, without `adsh`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = [
            NUM_TABLE,
            &self.tag,
            &self.version,
            &self.ddate,
            &self.qtrs,
            &self.uom,
            &self.dimh,
            &self.iprx,
        ];
        write!(f, "{}", fields.join(FIELD_SEPARATOR))
    }
}

/// A row of `txt.tsv`: one text value of one filing, with the columns the parser needs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TxtRow {
    /// The local name of the concept, for example `AccountingPoliciesTextBlock`.
    pub tag: String,
    /// The taxonomy version, for example `us-gaap/2026`, or the accession number for an extension.
    pub version: String,
    /// The end date, rounded to a month end, as `YYYYMMDD`.
    pub ddate: String,
    /// The length in quarters. `0` for an instant or a duration under about 45 days.
    pub qtrs: String,
    /// The hash of the dimensions, the key of a `dim` row.
    pub dimh: String,
    /// The index of the copy among rows with one primary key.
    pub iprx: String,
    /// The text with its whitespace collapsed and its markup removed.
    pub value: String,
    /// The number of bytes of the value in the source. `0` for a nil fact.
    pub srclen: String,
    /// The footnote text. Empty if there is none.
    pub footnote: String,
    /// The member of the legal entity axis. Empty for the registrant.
    pub coreg: String,
    /// `qtrs` minus the length of the fact in quarters, as in `-0.317808`. `0.0` for an instant.
    pub durp: String,
    /// The days from the exact end to `ddate`, as in `-15.0`.
    pub datp: String,
}

impl TxtRow {
    /// The type name that [`InvalidValue`] names when a field breaks a rule.
    const TYPE_NAME: &'static str = "TxtRow";

    /// Returns the period, by the rules of [`NumRow::period`].
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if `ddate`, `qtrs`, `durp`, or `datp` breaks a rule of
    /// [`NumRow::period`].
    pub fn period(&self) -> Result<Period, InvalidValue> {
        period(
            Self::TYPE_NAME,
            &self.ddate,
            &self.qtrs,
            &self.durp,
            &self.datp,
        )
    }

    /// Returns the value: nil if `srclen` is `0`, else the text, which can be empty.
    ///
    /// The SEC removes markup from a text block, so a value can be empty although the source had
    /// bytes. Only `srclen` tells a nil fact from an empty text.
    #[must_use]
    pub fn value(&self) -> FactValue {
        if self.value.is_empty() && self.srclen == NIL_SOURCE_LENGTH {
            FactValue::Nil
        } else {
            FactValue::Text(self.value.clone())
        }
    }

    /// Returns the copy index `iprx`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if `iprx` is not a number of 32 bits.
    pub fn copy_index(&self) -> Result<u32, InvalidValue> {
        copy_index(Self::TYPE_NAME, &self.iprx)
    }

    /// Returns the footnote as a list with one entry, or an empty list.
    #[must_use]
    pub fn footnotes(&self) -> Vec<String> {
        footnotes(&self.footnote)
    }

    /// Returns the dimensions, by the rules of [`NumRow::dimensions`].
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if no `dim` row has the key `dimh`, a pair has no `=`, or two pairs
    /// name one axis.
    pub fn dimensions(&self, dims: &HashMap<&str, &DimRow>) -> Result<Dimensions, InvalidValue> {
        dimensions(Self::TYPE_NAME, &self.dimh, &self.coreg, dims)
    }
}

impl fmt::Display for TxtRow {
    /// Writes the table and the primary key of the row, without `adsh`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = [
            TXT_TABLE,
            &self.tag,
            &self.version,
            &self.ddate,
            &self.qtrs,
            &self.dimh,
            &self.iprx,
        ];
        write!(f, "{}", fields.join(FIELD_SEPARATOR))
    }
}

/// A row of `dim.tsv`: the axis and member pairs of one dimension hash.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DimRow {
    /// The hash, the key that `num.dimh` names.
    pub dimh: String,
    /// The pairs, as in `Axis=Member;Axis=Member;`. Empty for the hash of no dimensions.
    pub segments: String,
    /// `1` if the SEC cut `segments` to 1024 characters, else `0`.
    pub segt: String,
}

impl DimRow {
    /// The type name that [`InvalidValue`] names when a field breaks a rule.
    const TYPE_NAME: &'static str = "DimRow";

    /// Returns the axis and member pairs, with the prefix stripped from each name that looks like
    /// a qualified name: a lowercase prefix and a local name that starts with a capital letter, as
    /// in `us-gaap:CostOfRevenue`.
    ///
    /// The Notes data set writes a typed member as the filer wrote it, for example a security name
    /// with spaces or `ISIN:US0378331005`. Neither looks like a qualified name, so both stay as
    /// written. A typed member such as `abc:Def` loses its prefix.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the SEC cut `segments`, or if a pair has no `=`.
    pub fn pairs(&self) -> Result<Vec<(&str, &str)>, InvalidValue> {
        if self.segt == TRUNCATED_SEGMENTS {
            return Err(InvalidValue::new(
                Self::TYPE_NAME,
                InvalidValueReason::Truncated(SEGMENTS_FIELD),
                &self.dimh,
            ));
        }
        self.segments
            .split(SEGMENT_TERMINATOR)
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                pair.split_once(PAIR_SEPARATOR)
                    .map(|(axis, member)| (without_prefix(axis), without_prefix(member)))
                    .ok_or_else(|| {
                        InvalidValue::new(
                            Self::TYPE_NAME,
                            InvalidValueReason::MissingSeparator(PAIR_SEPARATOR),
                            pair,
                        )
                    })
            })
            .collect()
    }
}

/// A row of `tag.tsv`: the data type of one tag of one version.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagRow {
    /// The local name of the concept.
    pub tag: String,
    /// The taxonomy version, or the accession number for an extension.
    pub version: String,
    /// The data type, for example `monetary` or `perShare`.
    pub datatype: String,
}

impl TagRow {
    /// Returns `true` if the tag is a per-share tag, so `num.uom` lacks `/shares`.
    #[must_use]
    pub fn is_per_share(&self) -> bool {
        self.datatype == PER_SHARE_DATA_TYPE
    }
}

fn period(
    type_name: &'static str,
    ddate: &str,
    qtrs: &str,
    durp: &str,
    datp: &str,
) -> Result<Period, InvalidValue> {
    let invalid = |reason, input: &str| InvalidValue::new(type_name, reason, input);
    let end = date(type_name, ddate)?
        .checked_sub_signed(datp_days(type_name, datp)?)
        .ok_or_else(|| invalid(InvalidValueReason::OutOfRange, datp))?;
    let quarters: u16 = qtrs
        .parse()
        .map_err(|_| invalid(InvalidValueReason::NotANumber, qtrs))?;
    let is_instant = quarters == 0
        && BigDecimal::from_str(durp)
            .map_err(|_| invalid(InvalidValueReason::NotANumber, durp))?
            .is_zero();
    Ok(if is_instant {
        Period::Instant { end }
    } else {
        Period::Duration(Duration::from_end_and_quarters(end, quarters))
    })
}

fn datp_days(type_name: &'static str, datp: &str) -> Result<TimeDelta, InvalidValue> {
    let invalid = |reason| InvalidValue::new(type_name, reason, datp);
    let whole = match datp.split_once(DECIMAL_POINT) {
        Some((whole, fraction)) if fraction.trim_end_matches(ZERO_DIGIT).is_empty() => whole,
        Some(_) => return Err(invalid(InvalidValueReason::NotANumber)),
        None => datp,
    };
    let days: i64 = whole
        .parse()
        .map_err(|_| invalid(InvalidValueReason::NotANumber))?;
    TimeDelta::try_days(days).ok_or_else(|| invalid(InvalidValueReason::OutOfRange))
}

fn copy_index(type_name: &'static str, iprx: &str) -> Result<u32, InvalidValue> {
    iprx.parse()
        .map_err(|_| InvalidValue::new(type_name, InvalidValueReason::NotANumber, iprx))
}

fn footnotes(footnote: &str) -> Vec<String> {
    if footnote.is_empty() {
        Vec::new()
    } else {
        vec![footnote.to_owned()]
    }
}

fn dimensions(
    type_name: &'static str,
    dimh: &str,
    coreg: &str,
    dim_rows: &HashMap<&str, &DimRow>,
) -> Result<Dimensions, InvalidValue> {
    let dim = dim_rows.get(dimh).ok_or_else(|| {
        InvalidValue::new(
            type_name,
            InvalidValueReason::NoMatchingRow(DIM_TABLE),
            dimh,
        )
    })?;
    let mut pairs = dim.pairs()?;
    let coreg_pair = (LEGAL_ENTITY_AXIS, coreg);
    if !coreg.is_empty() && !pairs.contains(&coreg_pair) {
        pairs.push(coreg_pair);
    }
    Dimensions::new(pairs)
}

fn date(type_name: &'static str, text: &str) -> Result<NaiveDate, InvalidValue> {
    NaiveDate::parse_from_str(text, DATE_FORMAT)
        .map_err(|_| InvalidValue::new(type_name, InvalidValueReason::NotADate(DATE_FORMAT), text))
}

fn optional<T>(
    text: &str,
    map: impl FnOnce(&str) -> Result<T, InvalidValue>,
) -> Result<Option<T>, InvalidValue> {
    if text.is_empty() {
        Ok(None)
    } else {
        map(text).map(Some)
    }
}

fn without_prefix(name: &str) -> &str {
    match name.split_once(PREFIX_SEPARATOR) {
        Some((prefix, local_name)) if is_prefix(prefix) && is_local_name(local_name) => local_name,
        _ => name,
    }
}

fn is_prefix(text: &str) -> bool {
    text.chars()
        .next()
        .is_some_and(|first| first.is_ascii_lowercase())
        && text.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || NAME_PUNCTUATION.contains(&character)
        })
}

fn is_local_name(text: &str) -> bool {
    text.chars()
        .next()
        .is_some_and(|first| first.is_ascii_uppercase())
        && text
            .chars()
            .all(|character| character.is_alphanumeric() || NAME_PUNCTUATION.contains(&character))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn num_row() -> NumRow {
        NumRow {
            tag: "Revenues".to_owned(),
            version: "us-gaap/2026".to_owned(),
            ddate: "20260930".to_owned(),
            qtrs: "1".to_owned(),
            uom: "USD".to_owned(),
            dimh: "0x00000000".to_owned(),
            iprx: "0".to_owned(),
            value: "100.0000".to_owned(),
            durp: "0.0".to_owned(),
            datp: "0.0".to_owned(),
            dcml: "-6".to_owned(),
            ..NumRow::default()
        }
    }

    fn dim_row(dimh: &str, segments: &str) -> DimRow {
        DimRow {
            dimh: dimh.to_owned(),
            segments: segments.to_owned(),
            segt: "0".to_owned(),
        }
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded valid date, the creation should always succeed")
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_sub_row() {
        implements_auto_traits::<SubRow>();
    }

    #[test]
    const fn should_implement_auto_traits_for_num_row() {
        implements_auto_traits::<NumRow>();
    }

    #[test]
    const fn should_implement_auto_traits_for_dim_row() {
        implements_auto_traits::<DimRow>();
    }

    #[test]
    const fn should_implement_auto_traits_for_tag_row() {
        implements_auto_traits::<TagRow>();
    }

    #[test]
    fn should_strip_the_prefix_when_a_member_is_a_qualified_name() {
        let dim = dim_row(
            "0xb5",
            "StatementOfIncomeLocationBalance=us-gaap:LaborAndRelatedExpense;",
        );

        let expected_result = Ok(vec![(
            "StatementOfIncomeLocationBalance",
            "LaborAndRelatedExpense",
        )]);

        let result = dim.pairs();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_a_typed_member_as_written_when_it_holds_a_colon_and_spaces() {
        let dim = dim_row(
            "0x00",
            "InvestmentIdentifier=JABIL INC COM USD0.001 SEDOL : 2471789;",
        );

        let expected_result = Ok(vec![(
            "InvestmentIdentifier",
            "JABIL INC COM USD0.001 SEDOL : 2471789",
        )]);

        let result = dim.pairs();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_a_typed_member_as_written_when_its_prefix_is_not_lowercase() {
        let dim = dim_row("0x05", "InvestmentIdentifier=ISIN:US0378331005;");

        let expected_result = Ok(vec![("InvestmentIdentifier", "ISIN:US0378331005")]);

        let result = dim.pairs();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_pairs_when_the_sec_cut_the_segments() {
        let dim = DimRow {
            segt: "1".to_owned(),
            ..dim_row("0x06", "InvestmentIdentifier=Corporate Stock;")
        };

        let expected_result = Err(InvalidValue::new(
            "DimRow",
            InvalidValueReason::Truncated("dim.segments"),
            "0x06",
        ));

        let result = dim.pairs();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_pair_when_it_has_no_equals_sign() {
        let dim = dim_row("0x01", "Segments;");

        let expected_result = Err(InvalidValue::new(
            "DimRow",
            InvalidValueReason::MissingSeparator('='),
            "Segments",
        ));

        let result = dim.pairs();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_restore_shares_when_the_tag_is_a_per_share_tag() {
        let row = num_row();

        let expected_result = Unit::from_str("USD/shares");

        let result = row.unit(true);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_unit_when_a_per_share_unit_already_divides() {
        let row = NumRow {
            uom: "USD/bbl".to_owned(),
            ..num_row()
        };

        let expected_result = Unit::from_str("USD/bbl");

        let result = row.unit(true);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_subtract_datp_from_ddate_when_the_exact_end_lies_after_the_month_end() {
        let row = NumRow {
            ddate: "20251231".to_owned(),
            qtrs: "0".to_owned(),
            datp: "-15.0".to_owned(),
            ..num_row()
        };

        let expected_result = Ok(Period::Instant {
            end: date(2026, 1, 15),
        });

        let result = row.period();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_duration_with_no_start_when_qtrs_is_not_zero() {
        let row = NumRow {
            ddate: "20250930".to_owned(),
            qtrs: "4".to_owned(),
            datp: "3.0".to_owned(),
            ..num_row()
        };

        let expected_result = Ok(Period::Duration(Duration::from_end_and_quarters(
            date(2025, 9, 27),
            4,
        )));

        let result = row.period();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_duration_of_no_quarters_when_qtrs_is_zero_and_durp_is_not() {
        let row = NumRow {
            ddate: "20250630".to_owned(),
            qtrs: "0".to_owned(),
            durp: "-0.317808".to_owned(),
            ..num_row()
        };

        let expected_result = Ok(Period::Duration(Duration::from_end_and_quarters(
            date(2025, 6, 30),
            0,
        )));

        let result = row.period();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_duration_without_reading_durp_when_qtrs_is_not_zero() {
        let row = NumRow {
            qtrs: "4".to_owned(),
            durp: String::new(),
            ..num_row()
        };

        let expected_result = Ok(Period::Duration(Duration::from_end_and_quarters(
            date(2026, 9, 30),
            4,
        )));

        let result = row.period();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_period_when_datp_is_not_a_whole_number_of_days() {
        let row = NumRow {
            datp: "1.5".to_owned(),
            ..num_row()
        };

        let expected_result = Err(InvalidValue::new(
            "NumRow",
            InvalidValueReason::NotANumber,
            "1.5",
        ));

        let result = row.period();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_infinite_decimals_when_dcml_is_32767() {
        let row = NumRow {
            dcml: "32767".to_owned(),
            ..num_row()
        };

        let expected_result = Ok(Some(Decimals::Infinite));

        let result = row.decimals();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_decimals_when_dcml_is_the_nil_marker() {
        let row = NumRow {
            value: String::new(),
            dcml: "-32768".to_owned(),
            ..num_row()
        };

        let expected_result = Ok(None);

        let result = row.decimals();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_nil_when_the_value_is_empty() {
        let row = NumRow {
            value: String::new(),
            ..num_row()
        };

        let expected_result = Ok(FactValue::Nil);

        let result = row.value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_add_the_legal_entity_pair_when_coreg_is_set_and_the_dim_row_lacks_it() {
        let row = NumRow {
            dimh: "0x02".to_owned(),
            coreg: "OhioPowerCompany".to_owned(),
            ..num_row()
        };
        let dim = dim_row("0x02", "ProductOrService=Electricity;");
        let dims = HashMap::from([("0x02", &dim)]);

        let expected_result = Dimensions::new([
            ("LegalEntity", "OhioPowerCompany"),
            ("ProductOrService", "Electricity"),
        ]);

        let result = row.dimensions(&dims);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_one_legal_entity_pair_when_the_dim_row_already_holds_coreg() {
        let row = NumRow {
            dimh: "0x03".to_owned(),
            coreg: "OhioPowerCompany".to_owned(),
            ..num_row()
        };
        let dim = dim_row("0x03", "LegalEntity=OhioPowerCompany;");
        let dims = HashMap::from([("0x03", &dim)]);

        let expected_result = Dimensions::new([("LegalEntity", "OhioPowerCompany")]);

        let result = row.dimensions(&dims);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_dimensions_when_no_dim_row_has_the_hash() {
        let row = NumRow {
            dimh: "0x04".to_owned(),
            ..num_row()
        };
        let dims = HashMap::new();

        let expected_result = Err(InvalidValue::new(
            "NumRow",
            InvalidValueReason::NoMatchingRow("dim.tsv"),
            "0x04",
        ));

        let result = row.dimensions(&dims);

        assert_eq!(result, expected_result);
    }

    #[test]
    const fn should_implement_auto_traits_for_txt_row() {
        implements_auto_traits::<TxtRow>();
    }

    #[test]
    fn should_give_nil_when_the_text_is_empty_and_the_source_had_no_bytes() {
        let row = TxtRow {
            srclen: "0".to_owned(),
            ..TxtRow::default()
        };

        let expected_result = FactValue::Nil;

        let result = row.value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_an_empty_text_when_the_text_is_empty_and_the_source_had_bytes() {
        let row = TxtRow {
            srclen: "37".to_owned(),
            ..TxtRow::default()
        };

        let expected_result = FactValue::Text(String::new());

        let result = row.value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_accepted_to_utc_when_new_york_is_on_standard_time() {
        let sub = SubRow {
            adsh: "0000000001-26-000001".to_owned(),
            cik: "1".to_owned(),
            form: "10-Q".to_owned(),
            filed: "20260115".to_owned(),
            accepted: "2026-01-15 16:15:00.0".to_owned(),
            ..SubRow::default()
        };

        let expected_result = Some(Utc.with_ymd_and_hms(2026, 1, 15, 21, 15, 0).single());

        let result = sub.filing().ok().map(|filing| filing.accepted);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_filing_when_the_filed_date_is_not_a_date() {
        let sub = SubRow {
            adsh: "0000000001-26-000001".to_owned(),
            cik: "1".to_owned(),
            form: "10-Q".to_owned(),
            filed: "2026-10-29".to_owned(),
            ..SubRow::default()
        };

        let expected_result = Some(InvalidValue::new(
            "SubRow",
            InvalidValueReason::NotADate("%Y%m%d"),
            "2026-10-29",
        ));

        let result = sub.filing().err();

        assert_eq!(result, expected_result);
    }
}
