//! # Constants
//!
//! The Notes periods that the backfill run downloads.

/// Every published Notes period, from 2009q1 to `2026_09`, in publication order.
///
/// The SEC publishes each month some days into the next month. Add a month to the list only after
/// the SEC publishes its file.
pub const NOTES_PERIODS: [&str; 79] = [
    "2009q1", "2009q2", "2009q3", "2009q4", "2010q1", "2010q2", "2010q3", "2010q4", "2011q1",
    "2011q2", "2011q3", "2011q4", "2012q1", "2012q2", "2012q3", "2012q4", "2013q1", "2013q2",
    "2013q3", "2013q4", "2014q1", "2014q2", "2014q3", "2014q4", "2015q1", "2015q2", "2015q3",
    "2015q4", "2016q1", "2016q2", "2016q3", "2016q4", "2017q1", "2017q2", "2017q3", "2017q4",
    "2018q1", "2018q2", "2018q3", "2018q4", "2019q1", "2019q2", "2019q3", "2019q4", "2020q1",
    "2020q2", "2020q3", "2020q4", "2021q1", "2021q2", "2021q3", "2021q4", "2022q1", "2022q2",
    "2022q3", "2022q4", "2023q1", "2023q2", "2023q3", "2023q4", "2024q1", "2024q2", "2024q3",
    "2024q4", "2025q1", "2025q2", "2025q3", "2025_10", "2025_11", "2025_12", "2026_01", "2026_02",
    "2026_03", "2026_04", "2026_05", "2026_06", "2026_07", "2026_08", "2026_09",
];

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use sec::shared::notes_period::{NotesPeriod, NotesPeriodError};

    use super::*;

    #[test]
    fn should_hold_valid_periods_in_increasing_order_for_notes_periods() {
        let expected_result = Ok(true);

        let result: Result<bool, NotesPeriodError> = NOTES_PERIODS
            .iter()
            .map(|period| NotesPeriod::new(period))
            .collect::<Result<Vec<_>, _>>()
            .map(|periods| periods.windows(2).all(|pair| pair[0] < pair[1]));

        assert_eq!(result, expected_result);
    }
}
