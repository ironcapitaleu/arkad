//! # Periods
//!
//! Provides [`notes_periods`], which lists the Notes periods that the backfill run asks for.

use chrono::{Months, NaiveDate};
use sec::shared::notes_period::NotesPeriod;

use crate::constants::PUBLICATION_LAG_MONTHS;

/// Returns every Notes period from 2009q1 to the month [`PUBLICATION_LAG_MONTHS`] before `today`,
/// in increasing order.
pub fn notes_periods(today: NaiveDate) -> Vec<String> {
    today
        .checked_sub_months(Months::new(PUBLICATION_LAG_MONTHS))
        .map_or_else(Vec::new, NotesPeriod::all_through)
        .iter()
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, building it should always succeed")
    }

    #[test]
    fn should_list_every_period_from_2009q1_to_2026_09_when_today_is_2026_10_31() {
        let today = date(2026, 10, 31);
        let periods_from_2009q1_to_2026_09 = vec![
            "2009q1", "2009q2", "2009q3", "2009q4", "2010q1", "2010q2", "2010q3", "2010q4",
            "2011q1", "2011q2", "2011q3", "2011q4", "2012q1", "2012q2", "2012q3", "2012q4",
            "2013q1", "2013q2", "2013q3", "2013q4", "2014q1", "2014q2", "2014q3", "2014q4",
            "2015q1", "2015q2", "2015q3", "2015q4", "2016q1", "2016q2", "2016q3", "2016q4",
            "2017q1", "2017q2", "2017q3", "2017q4", "2018q1", "2018q2", "2018q3", "2018q4",
            "2019q1", "2019q2", "2019q3", "2019q4", "2020q1", "2020q2", "2020q3", "2020q4",
            "2021q1", "2021q2", "2021q3", "2021q4", "2022q1", "2022q2", "2022q3", "2022q4",
            "2023q1", "2023q2", "2023q3", "2023q4", "2024q1", "2024q2", "2024q3", "2024q4",
            "2025q1", "2025q2", "2025q3", "2025_10", "2025_11", "2025_12", "2026_01", "2026_02",
            "2026_03", "2026_04", "2026_05", "2026_06", "2026_07", "2026_08", "2026_09",
        ];

        let expected_result = periods_from_2009q1_to_2026_09;

        let result = notes_periods(today);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_end_in_the_previous_year_when_today_is_in_january() {
        let today = date(2027, 1, 31);

        let expected_result = Some("2026_12".to_string());

        let result = notes_periods(today).pop();

        assert_eq!(result, expected_result);
    }
}
