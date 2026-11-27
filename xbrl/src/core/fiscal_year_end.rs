//! # Fiscal Year End
//!
//! Provides [`FiscalYearEnd`], the month and day on which the fiscal year of a company ends, as a
//! filing states it.

use std::fmt;
use std::str::FromStr;

use jiff::Span;
use jiff::civil::{Date, Time};

use super::invalid_value::{InvalidValue, InvalidValueReason};
use super::period::TOLERANCE_DAYS;

/// The fiscal year end of a company: a month and a day with no year, as the filing writes it.
///
/// The text keeps its form, `0930` from the Notes data set or `--09-27` from an instance document.
/// Two values are equal only if their texts are equal, so `0930` and `--09-30` differ. To match
/// the two forms, compare [`FiscalYearEnd::month`].
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
///
/// use xbrl::core::fiscal_year_end::FiscalYearEnd;
///
/// let text = "--09-27";
///
/// let expected_result = 9;
///
/// let year_end = FiscalYearEnd::from_str(text).expect("Given a hardcoded month and day, the parse should always succeed");
/// let result = year_end.month();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FiscalYearEnd {
    text: String,
    day_of_year: i16,
    month: u8,
}

impl FiscalYearEnd {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "FiscalYearEnd";

    /// A leap year, so that `0229` is a valid fiscal year end.
    const LEAP_YEAR: i16 = 2000;

    /// The forms of the text after [`Self::LEAP_YEAR`]: `0930` as in the Notes data set, and
    /// `--09-30` as in an instance document.
    const FORMATS: [&'static str; 2] = ["%Y%m%d", "%Y--%m-%d"];

    /// The form that [`InvalidValueReason::NotADate`] names.
    const EXPECTED_FORM: &'static str = "%m%d or --%m-%d with an optional time zone";

    /// The length of `--09-30`, the instance form without a time zone.
    const INSTANCE_FORM_LENGTH: usize = 7;

    /// The time zone that ends an instance form in UTC, as in `--09-30Z`.
    const UTC_ZONE: &'static str = "Z";

    /// The form of a time zone offset after its sign, as in `--09-30-05:00`.
    const ZONE_OFFSET_FORMAT: &'static str = "%H:%M";

    /// The days of a leap year.
    const LEAP_YEAR_DAYS: u16 = 366;

    /// Returns the text as the filing writes it, for example `0930` or `--09-27`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Returns the month of the month end nearest to the year end, from 1 to 12.
    ///
    /// A 52-53 week year ends on a weekday near a month end, so `--09-27` gives 9 and `0102`
    /// gives 12. February counts 29 days, so `0215` gives 2 and `0214` gives 1. If the day lies as
    /// far from the end of the previous month as from the end of its own month, as `0415` does,
    /// the month of the day wins.
    #[must_use]
    pub const fn month(&self) -> u8 {
        self.month
    }

    /// Returns `true` if the two year ends lie more than 7 days apart, around the year.
    pub(super) fn differs_from(&self, other: &Self) -> bool {
        let distance = self.day_of_year.abs_diff(other.day_of_year);
        i64::from(distance.min(Self::LEAP_YEAR_DAYS - distance)) > TOLERANCE_DAYS
    }

    /// Returns the month of the month end nearest to `date`, as [`Self::month`] states the rule.
    fn nearest_month_end(date: Date) -> Option<i8> {
        if 2 * date.day() >= date.days_in_month() {
            Some(date.month())
        } else {
            date.checked_sub(Span::new().months(1))
                .ok()
                .map(Date::month)
        }
    }

    /// Returns the text without the time zone that an instance form can carry, as in `--09-30Z`
    /// or `--09-30-05:00`. A fiscal year end names a day of the year, so the zone adds no fact.
    fn without_time_zone(text: &str) -> &str {
        let Some((month_day, zone)) = text
            .starts_with("--")
            .then(|| text.split_at_checked(Self::INSTANCE_FORM_LENGTH))
            .flatten()
        else {
            return text;
        };
        let is_offset = zone
            .strip_prefix(['+', '-'])
            .is_some_and(|offset| Time::strptime(Self::ZONE_OFFSET_FORMAT, offset).is_ok());
        if zone == Self::UTC_ZONE || is_offset {
            month_day
        } else {
            text
        }
    }
}

impl FromStr for FiscalYearEnd {
    type Err = InvalidValue;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let dated = format!("{}{}", Self::LEAP_YEAR, Self::without_time_zone(text));
        let date = Self::FORMATS
            .iter()
            .find_map(|format| Date::strptime(format, &dated).ok())
            .ok_or_else(|| {
                InvalidValue::new(
                    Self::TYPE_NAME,
                    InvalidValueReason::NotADate(Self::EXPECTED_FORM),
                    text,
                )
            })?;
        // The month of a date of 2000 always exists and fits a `u8`, so this error cannot occur.
        let month = Self::nearest_month_end(date)
            .and_then(|month| u8::try_from(month).ok())
            .ok_or_else(|| {
                InvalidValue::new(Self::TYPE_NAME, InvalidValueReason::OutOfRange, text)
            })?;
        Ok(Self {
            text: text.to_owned(),
            day_of_year: date.day_of_year(),
            month,
        })
    }
}

impl fmt::Display for FiscalYearEnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

impl serde::Serialize for FiscalYearEnd {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.text)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn month(text: &str) -> u8 {
        FiscalYearEnd::from_str(text)
            .expect("Given a hardcoded month and day, the parse should always succeed")
            .month()
    }

    #[test]
    fn should_parse_instance_form_to_day_of_leap_year_when_end_is_september_30() {
        let instance = "--09-30";

        let expected_result = Ok(274);

        let result = FiscalYearEnd::from_str(instance).map(|year_end| year_end.day_of_year);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_fiscal_year_end_when_text_is_not_a_date() {
        let text = "Sept";

        let expected_result = Err(InvalidValue::new(
            "FiscalYearEnd",
            InvalidValueReason::NotADate("%m%d or --%m-%d with an optional time zone"),
            "Sept",
        ));

        let result = FiscalYearEnd::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_change_fiscal_year_end_when_52_53_week_year_drifts() {
        let old = FiscalYearEnd::from_str("--09-24")
            .expect("Given a hardcoded month and day, the parse should always succeed");
        let new = FiscalYearEnd::from_str("--09-30")
            .expect("Given a hardcoded month and day, the parse should always succeed");

        let expected_result = false;

        let result = old.differs_from(&new);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_notes_form_to_day_of_leap_year_when_end_is_september_30() {
        let notes = "0930";

        let expected_result = Ok(274);

        let result = FiscalYearEnd::from_str(notes).map(|year_end| year_end.day_of_year);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_change_fiscal_year_end_when_month_changes() {
        let old = FiscalYearEnd::from_str("1231")
            .expect("Given a hardcoded month and day, the parse should always succeed");
        let new = FiscalYearEnd::from_str("0630")
            .expect("Given a hardcoded month and day, the parse should always succeed");

        let expected_result = true;

        let result = old.differs_from(&new);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_change_fiscal_year_end_when_end_drifts_across_new_year() {
        let old = FiscalYearEnd::from_str("1231")
            .expect("Given a hardcoded month and day, the parse should always succeed");
        let new = FiscalYearEnd::from_str("0102")
            .expect("Given a hardcoded month and day, the parse should always succeed");

        let expected_result = false;

        let result = old.differs_from(&new);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_round_to_january_when_day_is_february_14() {
        let text = "0214";

        let expected_result = 1;

        let result = month(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_month_of_day_when_day_is_as_far_from_both_month_ends() {
        let text = "0415";

        let expected_result = 4;

        let result = month(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_round_to_previous_month_when_day_is_april_14() {
        let text = "0414";

        let expected_result = 3;

        let result = month(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_february_when_day_is_february_29() {
        let text = "0229";

        let expected_result = 2;

        let result = month(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_round_to_august_when_costco_ends_on_september_1() {
        let text = "--09-01";

        let expected_result = 8;

        let result = month(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_round_to_february_when_day_is_march_15() {
        let text = "0315";

        let expected_result = 2;

        let result = month(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_march_when_day_is_march_16() {
        let text = "0316";

        let expected_result = 3;

        let result = month(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_instance_form_when_time_zone_is_utc() {
        let instance = "--09-30Z";

        let expected_result = Ok(274);

        let result = FiscalYearEnd::from_str(instance).map(|year_end| year_end.day_of_year);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_instance_form_when_time_zone_is_an_offset() {
        let instance = "--09-30-05:00";

        let expected_result = Ok(274);

        let result = FiscalYearEnd::from_str(instance).map(|year_end| year_end.day_of_year);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_instance_form_when_suffix_is_not_a_time_zone() {
        let instance = "--09-30X";

        let expected_result = Err(InvalidValue::new(
            "FiscalYearEnd",
            InvalidValueReason::NotADate("%m%d or --%m-%d with an optional time zone"),
            "--09-30X",
        ));

        let result = FiscalYearEnd::from_str(instance);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_as_the_text_of_the_filing_when_form_is_instance() {
        let year_end = FiscalYearEnd::from_str("--09-27")
            .expect("Given a hardcoded month and day, the parse should always succeed");

        let expected_result = Some(r#""--09-27""#.to_owned());

        let result = serde_json::to_string(&year_end).ok();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_text_of_the_filing_when_form_is_notes() {
        let year_end = FiscalYearEnd::from_str("0930")
            .expect("Given a hardcoded month and day, the parse should always succeed");

        let expected_result = "0930";

        let result = year_end.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_fiscal_year_end() {
        implements_auto_traits::<FiscalYearEnd>();
    }
}
