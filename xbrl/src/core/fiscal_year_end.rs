//! # Fiscal Year End
//!
//! Provides [`FiscalYearEnd`], the month and day on which the fiscal year of a company ends, and
//! the month of its nearest month end.

use std::str::FromStr;

use chrono::{Datelike, Months, NaiveDate};

use super::chronology::TOLERANCE_DAYS;
use super::invalid_value::{InvalidValue, InvalidValueReason};

/// A fiscal year end: a month and a day, as the day of a leap year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FiscalYearEnd(u32);

impl FiscalYearEnd {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "FiscalYearEnd";

    /// A leap year, so that `0229` is a valid fiscal year end.
    const LEAP_YEAR: i32 = 2000;

    /// The forms of the text after [`Self::LEAP_YEAR`]: `0930` as in the Notes data set, and
    /// `--09-30` as in an instance document.
    const FORMATS: [&'static str; 2] = ["%Y%m%d", "%Y--%m-%d"];

    /// The form that [`InvalidValueReason::NotADate`] names.
    const EXPECTED_FORM: &'static str = "%m%d or --%m-%d";

    /// The days of a leap year.
    const LEAP_YEAR_DAYS: u32 = 366;

    /// Returns `true` if the two year ends lie more than 7 days apart, around the year.
    pub(super) fn differs_from(self, other: Self) -> bool {
        let distance = self.0.abs_diff(other.0);
        i64::from(distance.min(Self::LEAP_YEAR_DAYS - distance)) > TOLERANCE_DAYS
    }

    /// Returns the month of the month end nearest to the year end, from 1 to 12.
    ///
    /// A 52-53 week year ends on a weekday near a month end, so `--09-27` gives 9 and `--01-02`
    /// gives 12. February has 29 days here. If the day lies as far from the end of the previous
    /// month as from the end of its own month, as `0415` does, the month of the day wins.
    pub(super) fn month(self) -> Option<u8> {
        let date = NaiveDate::from_yo_opt(Self::LEAP_YEAR, self.0)?;
        let month = if 2 * date.day() >= u32::from(date.num_days_in_month()) {
            date.month()
        } else {
            date.checked_sub_months(Months::new(1))?.month()
        };
        u8::try_from(month).ok()
    }
}

impl FromStr for FiscalYearEnd {
    type Err = InvalidValue;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let dated = format!("{}{text}", Self::LEAP_YEAR);
        Self::FORMATS
            .iter()
            .find_map(|format| NaiveDate::parse_from_str(&dated, format).ok())
            .map(|date| Self(date.ordinal()))
            .ok_or_else(|| {
                InvalidValue::new(
                    Self::TYPE_NAME,
                    InvalidValueReason::NotADate(Self::EXPECTED_FORM),
                    text,
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn month(text: &str) -> Option<u8> {
        FiscalYearEnd::from_str(text)
            .expect("Given a hardcoded month and day, the parse should always succeed")
            .month()
    }

    #[test]
    fn should_parse_instance_form_to_day_of_leap_year_when_end_is_september_30() {
        let instance = "--09-30";

        let expected_result = Ok(FiscalYearEnd(274));

        let result = FiscalYearEnd::from_str(instance);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_fiscal_year_end_when_text_is_not_a_date() {
        let text = "Sept";

        let expected_result = Err(InvalidValue::new(
            "FiscalYearEnd",
            InvalidValueReason::NotADate("%m%d or --%m-%d"),
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

        let result = old.differs_from(new);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_notes_form_to_day_of_leap_year_when_end_is_september_30() {
        let notes = "0930";

        let expected_result = Ok(FiscalYearEnd(274));

        let result = FiscalYearEnd::from_str(notes);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_change_fiscal_year_end_when_month_changes() {
        let old = FiscalYearEnd::from_str("1231")
            .expect("Given a hardcoded month and day, the parse should always succeed");
        let new = FiscalYearEnd::from_str("0630")
            .expect("Given a hardcoded month and day, the parse should always succeed");

        let expected_result = true;

        let result = old.differs_from(new);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_change_fiscal_year_end_when_end_drifts_across_new_year() {
        let old = FiscalYearEnd::from_str("1231")
            .expect("Given a hardcoded month and day, the parse should always succeed");
        let new = FiscalYearEnd::from_str("0102")
            .expect("Given a hardcoded month and day, the parse should always succeed");

        let expected_result = false;

        let result = old.differs_from(new);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_round_to_january_when_day_is_february_14() {
        let text = "0214";

        let expected_result = Some(1);

        let result = month(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_month_of_day_when_day_is_as_far_from_both_month_ends() {
        let text = "0415";

        let expected_result = Some(4);

        let result = month(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_round_to_previous_month_when_day_is_april_14() {
        let text = "0414";

        let expected_result = Some(3);

        let result = month(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_february_when_day_is_february_29() {
        let text = "0229";

        let expected_result = Some(2);

        let result = month(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_round_to_august_when_costco_ends_on_september_1() {
        let text = "--09-01";

        let expected_result = Some(8);

        let result = month(text);

        assert_eq!(result, expected_result);
    }
}
