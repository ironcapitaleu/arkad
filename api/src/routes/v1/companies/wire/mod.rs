//! # Wire
//!
//! Provides the serde functions that write `xbrl` domain types in the format of the `/v1`
//! contract.
//!
//! The `xbrl` types serialize with their Rust names and shapes, for example `"Exact"` or
//! `{"Instant": {"end": …}}`. The contract fixes `snake_case` values and its own shapes. The
//! response types hold the `xbrl` types and name these functions in `#[serde(serialize_with)]`.

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use serde::de::Error as _;
use serde::ser::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;
use utoipa::openapi::schema::{Object, ObjectBuilder, Type};
use xbrl::core::confidence::Confidence;
use xbrl::core::elements::CanonicalElement;
use xbrl::core::filing::FilingId;
use xbrl::core::fiscal_period::FiscalPeriod;
use xbrl::core::period::Period;
use xbrl::core::unit::{Measure, Unit};

use crate::routes::v1::concepts::concept_id;

/// The wire shape of a [`Period`]: `duration` with `start` and `end`, or `instant` with `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[schema(as = Period)]
pub enum PeriodJson {
    /// A range of days, for example a fiscal year of an income statement.
    Duration {
        /// The first day, or `null` if the source gives only the number of quarters.
        #[schema(value_type = Option<String>, format = Date, example = "2023-10-01")]
        start: Option<NaiveDate>,
        /// The last day.
        #[schema(value_type = String, format = Date, example = "2024-09-28")]
        end: NaiveDate,
    },
    /// One day, for example a balance sheet date.
    Instant {
        /// The day.
        #[schema(value_type = String, format = Date, example = "2024-09-28")]
        end: NaiveDate,
    },
}

/// Writes a [`Confidence`] as `exact`, `synonym`, `derived`, or `computed`.
///
/// # Errors
///
/// Returns the error of `serializer`.
pub fn confidence<S: Serializer>(
    confidence: &Confidence,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let text = match confidence {
        Confidence::Exact => "exact",
        Confidence::Synonym => "synonym",
        Confidence::Derived => "derived",
        Confidence::Computed => "computed",
    };
    serializer.serialize_str(text)
}

/// Returns the schema of a [`Confidence`] on the wire: one of four strings.
#[must_use]
pub fn confidence_schema() -> Object {
    ObjectBuilder::new()
        .schema_type(Type::String)
        .enum_values(Some(["exact", "synonym", "derived", "computed"]))
        .description(Some(
            "The resolution tier that produced the value. A later tier is less certain.",
        ))
        .build()
}

/// Writes a [`CanonicalElement`] as its concept ID, for example `net_income`.
///
/// # Errors
///
/// Returns a custom error of `serializer` if the variant name is not in `PascalCase`.
pub fn concept<S: Serializer>(
    element: &CanonicalElement,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let id = concept_id(*element).map_err(S::Error::custom)?;
    serializer.serialize_str(&id)
}

/// Writes a [`BigDecimal`] as an exact decimal string without an exponent.
///
/// # Errors
///
/// Returns the error of `serializer`.
pub fn decimal<S: Serializer>(value: &BigDecimal, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_plain_string())
}

/// Writes a [`FilingId`] as its ID within the source, which is the accession number for the SEC.
///
/// # Errors
///
/// Returns the error of `serializer`.
pub fn filing_id<S: Serializer>(id: &FilingId, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(id.id())
}

/// Writes a [`FiscalPeriod`] in lowercase, for example `fy` or `q1`.
///
/// # Errors
///
/// Returns the error of `serializer`.
pub fn fiscal_period<S: Serializer>(
    period: &FiscalPeriod,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&period.as_str().to_ascii_lowercase())
}

/// Reads an optional query value such as `fy` or `q1` into a [`FiscalPeriod`].
///
/// # Errors
///
/// Returns a custom error of `deserializer` if the value is not a valid fiscal period.
pub fn optional_fiscal_period<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<FiscalPeriod>, D::Error> {
    Option::<String>::deserialize(deserializer)?
        .map(|text| FiscalPeriod::new(text.to_ascii_uppercase()).map_err(D::Error::custom))
        .transpose()
}

/// Writes a [`Period`] in the shape of [`PeriodJson`].
///
/// # Errors
///
/// Returns a custom error of `serializer` for a kind of period that the contract has no shape for.
pub fn period<S: Serializer>(period: &Period, serializer: S) -> Result<S::Ok, S::Error> {
    let json = match period {
        Period::Instant { end } => PeriodJson::Instant { end: *end },
        Period::Duration(duration) => PeriodJson::Duration {
            start: duration.start(),
            end: duration.end(),
        },
        // `Period` is `#[non_exhaustive]`, so a new kind of period needs a wire shape first.
        _ => {
            return Err(S::Error::custom(format!(
                "[UnsupportedPeriod] The contract has no shape for the period '{period}'"
            )));
        }
    };
    json.serialize(serializer)
}

/// Writes a [`Unit`] as one text: a measure such as `USD`, or `usd_per_share` for currency per
/// share.
///
/// # Errors
///
/// Returns the error of `serializer`.
pub fn unit<S: Serializer>(unit: &Unit, serializer: S) -> Result<S::Ok, S::Error> {
    let text = match (unit.numerator(), unit.denominator()) {
        ([currency], [shares]) if shares.as_str() == "shares" && is_currency(currency) => {
            format!("{}_per_share", currency.as_str().to_ascii_lowercase())
        }
        _ => unit.to_string(),
    };
    serializer.serialize_str(&text)
}

fn is_currency(measure: &Measure) -> bool {
    Unit::new(vec![measure.clone()], Vec::new()).is_ok_and(|unit| unit.is_currency())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use pretty_assertions::assert_eq;
    use serde_json::{Value, json};
    use xbrl::core::period::Duration;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a valid hardcoded date, building it should always succeed")
    }

    #[test]
    fn should_write_snake_case_when_serializing_exact_confidence() {
        let value = Confidence::Exact;

        let expected_result = Ok(json!("exact"));

        let result =
            confidence(&value, serde_json::value::Serializer).map_err(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_concept_id_when_serializing_net_income() {
        let element = CanonicalElement::NetIncome;

        let expected_result = Ok(json!("net_income"));

        let result =
            concept(&element, serde_json::value::Serializer).map_err(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_no_exponent_when_serializing_a_large_decimal() {
        let value = BigDecimal::from_str("391035000000")
            .expect("Given a hardcoded integer text, parsing it should always succeed");

        let expected_result = Ok(json!("391035000000"));

        let result =
            decimal(&value, serde_json::value::Serializer).map_err(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_accession_number_when_serializing_a_filing_id() {
        let id = FilingId::new("sec", "0000320193-24-000123")
            .expect("Given a hardcoded source and ID, the creation should always succeed");

        let expected_result = Ok(json!("0000320193-24-000123"));

        let result =
            filing_id(&id, serde_json::value::Serializer).map_err(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_lowercase_when_serializing_a_fiscal_year() {
        let value = FiscalPeriod::new("FY")
            .expect("Given the hardcoded period 'FY', the creation should always succeed");

        let expected_result = Ok(json!("fy"));

        let result =
            fiscal_period(&value, serde_json::value::Serializer).map_err(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_an_uppercase_fiscal_period_when_the_query_holds_q1() {
        let query = json!("q1");

        let expected_result = Some("Q1".to_owned());

        let result = optional_fiscal_period(query)
            .expect("Given the valid hardcoded value 'q1', reading it should always succeed")
            .map(|period| period.as_str().to_owned());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_only_the_end_when_serializing_an_instant() {
        let value = Period::Instant {
            end: date(2024, 9, 28),
        };

        let expected_result = Ok(json!({ "kind": "instant", "end": "2024-09-28" }));

        let result =
            period(&value, serde_json::value::Serializer).map_err(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_start_and_end_when_serializing_a_duration() {
        let duration = Duration::from_dates(date(2023, 10, 1), date(2024, 9, 28))
            .expect("Given a hardcoded start before the end, the creation should always succeed");
        let value = Period::Duration(duration);

        let expected_result =
            Ok(json!({ "kind": "duration", "start": "2023-10-01", "end": "2024-09-28" }));

        let result =
            period(&value, serde_json::value::Serializer).map_err(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_currency_when_serializing_usd() {
        let value = Unit::from_str("USD")
            .expect("Given the hardcoded unit 'USD', parsing it should always succeed");

        let expected_result = Ok(json!("USD"));

        let result = unit(&value, serde_json::value::Serializer).map_err(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_usd_per_share_when_serializing_usd_per_shares() {
        let value = Unit::from_str("USD/shares")
            .expect("Given the hardcoded unit 'USD/shares', parsing it should always succeed");

        let expected_result: Result<Value, String> = Ok(json!("usd_per_share"));

        let result = unit(&value, serde_json::value::Serializer).map_err(|error| error.to_string());

        assert_eq!(result, expected_result);
    }
}
