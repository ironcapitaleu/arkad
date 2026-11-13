//! # Published Periods
//!
//! Provides [`published_periods`], which lists the Notes periods whose files the SEC publishes now,
//! and its error [`FailedNotesProbe`].
//!
//! About a year after a quarter ends, the SEC merges the three monthly files of the quarter into
//! one quarterly file and removes the monthly files (assumption A4 in
//! `sec/design/pipeline/ingestion_paths.md`). The merge came 367 to 458 days after the quarter end
//! in the last 10 quarters. Within 24 months, the date does not tell whether a quarter is merged,
//! so the run asks the SEC with a `HEAD` request.

use std::time::Duration;

use chrono::{Datelike, Months, NaiveDate};
use thiserror::Error;

use super::super::retry::retry_while;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::notes_period::constants::FIRST_YEAR;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::response::implementations::sec_response::{SecResponse, ZipHeadBody};

/// The last quarter whose quarterly file the SEC had published on 2026-11-07.
///
/// A merged quarter stays merged, so the run does not ask for this quarter or an earlier one.
pub const LAST_MERGED_QUARTER: &str = "2025q3";

/// The number of months after the end of a quarter from which the run counts the quarter as merged
/// without a request.
///
/// The longest measured wait for a merge was 458 days, about 15 months. So at most 8 quarters need
/// a request.
pub const MERGE_WAIT_MONTHS: u32 = 24;

/// Error indicating that the run could not tell whether the quarterly Notes file of a period exists.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error(
    "[FailedNotesProbe] Failed to ask whether the Notes file of '{period}' exists, Caused by: {source}"
)]
pub struct FailedNotesProbe {
    /// The quarter that the request asked for.
    pub period: NotesPeriod,
    /// The error of the request. Its status is not 404.
    #[source]
    pub source: FailedSecRequest,
}

/// Returns every Notes period that ends in or before the month of `through`, in increasing order.
///
/// A quarter up to [`LAST_MERGED_QUARTER`], or one that ended [`MERGE_WAIT_MONTHS`] or more before
/// `through`, counts as merged. For every later quarter, the run sends one `HEAD` request for the
/// quarterly file. If the SEC answers 200, the list holds the quarter. If it answers 404, the list
/// holds the three months of the quarter instead. The months of a quarter that has not ended by
/// `through` follow, up to the month of `through`.
///
/// A request that fails with a transient SEC failure runs again after each wait of
/// `retry_delays`, as the downloads do, and `on_retry` gets the quarter and the error. The list is
/// empty if `through` comes before [`FIRST_YEAR`].
///
/// # Errors
///
/// Returns a [`FailedNotesProbe`] if a request fails with any answer other than 200 or 404, after
/// the retries for a transient failure. A 403 can mean that the SEC refused the request, so it does
/// not count as a missing file.
pub async fn published_periods<C>(
    sec_client: &C,
    through: NaiveDate,
    retry_delays: &[Duration],
    on_retry: impl Fn(NotesPeriod, &FailedSecRequest) + Send + Sync,
) -> Result<Vec<NotesPeriod>, FailedNotesProbe>
where
    C: SecClientTrait<Response<ZipHeadBody> = SecResponse<ZipHeadBody>, Error = FailedSecRequest>,
{
    let through_month = (through.year(), through.month());
    let mut periods = Vec::new();
    for year in i32::from(FIRST_YEAR.number())..=through.year() {
        for quarter in 1..=4 {
            let months = (quarter * 3 - 2..=quarter * 3).map(|month| (year, month));
            if (year, quarter * 3) > through_month {
                periods.extend(
                    months
                        .filter(|month| *month <= through_month)
                        .map(|(year, month)| period(&format!("{year}_{month:02}"))),
                );
                continue;
            }
            let quarter = period(&format!("{year}q{quarter}"));
            if is_merged(quarter, through)
                || is_published(sec_client, quarter, retry_delays, &on_retry).await?
            {
                periods.push(quarter);
            } else {
                periods.extend(months.map(|(year, month)| period(&format!("{year}_{month:02}"))));
            }
        }
    }
    Ok(periods)
}

/// Returns whether the run counts `quarter` as merged without a request.
fn is_merged(quarter: NotesPeriod, through: NaiveDate) -> bool {
    quarter <= period(LAST_MERGED_QUARTER)
        || quarter
            .last_day()
            .checked_add_months(Months::new(MERGE_WAIT_MONTHS))
            .is_some_and(|merged| merged <= through)
}

/// Asks the SEC whether the Notes file of `quarter` exists. A request that fails with a transient
/// SEC failure runs again after each wait of `retry_delays`, and `on_retry` gets its error.
///
/// # Errors
///
/// Returns a [`FailedNotesProbe`] if the last request fails with any answer other than 404.
pub(in crate::implementations::extraction) async fn is_published<C>(
    sec_client: &C,
    quarter: NotesPeriod,
    retry_delays: &[Duration],
    on_retry: &(impl Fn(NotesPeriod, &FailedSecRequest) + Sync),
) -> Result<bool, FailedNotesProbe>
where
    C: SecClientTrait<Response<ZipHeadBody> = SecResponse<ZipHeadBody>, Error = FailedSecRequest>,
{
    let answer = retry_while(
        retry_delays,
        || {
            let request = SecRequest::builder()
                .notes_file_probe()
                .period(quarter)
                .build();
            sec_client.execute_sec_request(request)
        },
        FailedSecRequest::is_transient,
        |error| on_retry(quarter, error),
    )
    .await;
    match answer {
        Ok(_) => Ok(true),
        Err(error) if error.is_not_found() => Ok(false),
        Err(source) => Err(FailedNotesProbe {
            period: quarter,
            source,
        }),
    }
}

/// Parses a period that this module builds.
fn period(period: &str) -> NotesPeriod {
    NotesPeriod::new(period).expect(
        "Given a quarter or a zero-padded month from 2009 on, parsing should always succeed",
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;

    const NOTES_URL: &str =
        "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/";

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, building it should always succeed")
    }

    /// Returns a client that answers 200 for the quarterly file of each of `quarters`, and 404 for
    /// every other file.
    fn sec_with_quarters(quarters: &[&str]) -> RoutingSecClient {
        quarters
            .iter()
            .fold(RoutingSecClient::new(), |sec_client, quarter| {
                sec_client.route(
                    &format!("{NOTES_URL}{quarter}_notes.zip"),
                    200,
                    "application/octet-stream",
                    b"",
                )
            })
    }

    /// No wait between two attempts, so a test with a retry takes no time.
    const NO_WAIT: [Duration; 1] = [Duration::ZERO];

    async fn periods_from(sec_client: &RoutingSecClient, through: NaiveDate) -> Vec<String> {
        published_periods(sec_client, through, &NO_WAIT, |_, _| {})
            .await
            .expect("Given a client that answers only 200 or 404, the list should always build")
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[tokio::test]
    async fn should_list_the_80_files_of_2026_11_07_when_no_later_quarter_is_merged() {
        let sec_client = sec_with_quarters(&[]);
        let through = date(2026, 10, 7);
        let periods_from_2009q1_to_2026_10 = vec![
            "2009q1", "2009q2", "2009q3", "2009q4", "2010q1", "2010q2", "2010q3", "2010q4",
            "2011q1", "2011q2", "2011q3", "2011q4", "2012q1", "2012q2", "2012q3", "2012q4",
            "2013q1", "2013q2", "2013q3", "2013q4", "2014q1", "2014q2", "2014q3", "2014q4",
            "2015q1", "2015q2", "2015q3", "2015q4", "2016q1", "2016q2", "2016q3", "2016q4",
            "2017q1", "2017q2", "2017q3", "2017q4", "2018q1", "2018q2", "2018q3", "2018q4",
            "2019q1", "2019q2", "2019q3", "2019q4", "2020q1", "2020q2", "2020q3", "2020q4",
            "2021q1", "2021q2", "2021q3", "2021q4", "2022q1", "2022q2", "2022q3", "2022q4",
            "2023q1", "2023q2", "2023q3", "2023q4", "2024q1", "2024q2", "2024q3", "2024q4",
            "2025q1", "2025q2", "2025q3", "2025_10", "2025_11", "2025_12", "2026_01", "2026_02",
            "2026_03", "2026_04", "2026_05", "2026_06", "2026_07", "2026_08", "2026_09", "2026_10",
        ];

        let expected_result = periods_from_2009q1_to_2026_10;

        let result = periods_from(&sec_client, through).await;

        assert_eq!(result, expected_result);
    }

    // The fixed bounds listed `2025_10` to `2025_12` and never `2025q4`. After the SEC merges the
    // three months, it removes them, and a run without them failed.
    #[tokio::test]
    async fn should_list_the_merged_quarter_and_not_its_months_when_the_quarterly_file_exists() {
        let sec_client = sec_with_quarters(&["2025q4"]);
        let through = date(2027, 2, 10);

        let expected_result = vec!["2025q3", "2025q4", "2026_01", "2026_02", "2026_03"];

        let result: Vec<String> = periods_from(&sec_client, through)
            .await
            .into_iter()
            .skip(66)
            .take(5)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_list_the_months_of_a_quarter_when_through_is_its_last_month() {
        let sec_client = sec_with_quarters(&[]);
        let through = date(2026, 12, 31);

        let expected_result = vec!["2026_10", "2026_11", "2026_12"];

        let result: Vec<String> = periods_from(&sec_client, through).await.split_off(79);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_count_a_quarter_as_merged_without_a_request_when_it_ended_24_months_before() {
        let sec_client = sec_with_quarters(&[]);
        let through = date(2027, 12, 31);

        let expected_result = vec!["2025q4", "2026_01", "2026_02", "2026_03"];

        let result: Vec<String> = periods_from(&sec_client, through)
            .await
            .into_iter()
            .skip(67)
            .take(4)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_notes_probe_when_the_sec_answers_403() {
        let sec_client = RoutingSecClient::new().route(
            &format!("{NOTES_URL}2025q4_notes.zip"),
            403,
            "application/xml",
            b"",
        );
        let through = date(2026, 1, 31);

        let expected_result = Some("[FailedNotesProbe] Failed to ask whether the Notes file of '2025q4' exists, Caused by: [InvalidSecResponse] Invalid SEC Response, Reason: 'Status code '403' is not accepted'".to_string());

        let result = published_periods(&sec_client, through, &NO_WAIT, |_, _| {})
            .await
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_list_the_quarter_when_the_sec_answers_503_and_then_200() {
        let sec_client = RoutingSecClient::new().route_sequence(
            &format!("{NOTES_URL}2025q4_notes.zip"),
            &[
                (503, "text/html", b""),
                (200, "application/octet-stream", b""),
            ],
        );
        let through = date(2025, 12, 31);

        let expected_result = vec!["2025q3".to_string(), "2025q4".to_string()];

        let result = periods_from(&sec_client, through).await.split_off(66);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_notes_probe_without_a_retry_when_the_sec_answers_403_and_then_200()
     {
        let sec_client = RoutingSecClient::new().route_sequence(
            &format!("{NOTES_URL}2025q4_notes.zip"),
            &[
                (403, "text/html", b""),
                (200, "application/octet-stream", b""),
            ],
        );
        let through = date(2025, 12, 31);

        let expected_result = Some("[FailedNotesProbe] Failed to ask whether the Notes file of '2025q4' exists, Caused by: [InvalidSecResponse] Invalid SEC Response, Reason: 'Status code '403' is not accepted'".to_string());

        let result = published_periods(&sec_client, through, &NO_WAIT, |_, _| {})
            .await
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_notes_probe_when_the_sec_answers_503_after_every_retry() {
        let sec_client = RoutingSecClient::new().route(
            &format!("{NOTES_URL}2025q4_notes.zip"),
            503,
            "text/html",
            b"",
        );
        let through = date(2025, 12, 31);

        let expected_result = Some("[FailedNotesProbe] Failed to ask whether the Notes file of '2025q4' exists, Caused by: [InvalidSecResponse] Invalid SEC Response, Reason: 'Status code '503' is not accepted'".to_string());

        let result = published_periods(&sec_client, through, &NO_WAIT, |_, _| {})
            .await
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_report_the_quarter_of_each_retry_when_the_sec_answers_503_and_then_200() {
        let sec_client = RoutingSecClient::new().route_sequence(
            &format!("{NOTES_URL}2025q4_notes.zip"),
            &[
                (503, "text/html", b""),
                (200, "application/octet-stream", b""),
            ],
        );
        let through = date(2025, 12, 31);
        let retried = std::sync::Mutex::new(Vec::new());

        let expected_result = vec!["2025q4".to_string()];

        published_periods(&sec_client, through, &NO_WAIT, |quarter, _| {
            retried
                .lock()
                .expect("Given a callback that never panics, the lock should never be poisoned")
                .push(quarter.to_string());
        })
        .await
        .expect("Given a 503 and then a 200, the list should build after one retry");
        let result = retried
            .into_inner()
            .expect("Given a callback that never panics, the lock should never be poisoned");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_no_period_when_through_is_before_2009() {
        let sec_client = sec_with_quarters(&[]);
        let through = date(2008, 12, 31);

        let expected_result: Vec<String> = Vec::new();

        let result = periods_from(&sec_client, through).await;

        assert_eq!(result, expected_result);
    }
}
