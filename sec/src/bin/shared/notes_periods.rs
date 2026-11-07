//! # Notes Periods
//!
//! Provides [`notes_periods`], which lists the Notes periods that the backfill run asks for.

use chrono::{Months, NaiveDate};
use sec::implementations::extraction::{FailedNotesProbe, published_periods};
use sec::shared::http_client::SecClient as SecClientTrait;
use sec::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use sec::shared::response::implementations::sec_response::{SecResponse, ZipHeadBody};

/// The number of months between today and the last Notes period that the run asks for.
///
/// The SEC posts the file of month M during month M+1. On 2026-10-29, the SEC had posted
/// `2026_09`.
pub const PUBLICATION_LAG_MONTHS: u32 = 1;

/// Returns every published Notes period from 2009q1 to the month [`PUBLICATION_LAG_MONTHS`] before
/// `today`, in increasing order. See [`published_periods`].
///
/// # Errors
///
/// Returns the [`FailedNotesProbe`] of [`published_periods`].
pub async fn notes_periods<C>(
    sec_client: &C,
    today: NaiveDate,
) -> Result<Vec<String>, FailedNotesProbe>
where
    C: SecClientTrait<Response<ZipHeadBody> = SecResponse<ZipHeadBody>, Error = FailedSecRequest>,
{
    let Some(last_month) = today.checked_sub_months(Months::new(PUBLICATION_LAG_MONTHS)) else {
        return Ok(Vec::new());
    };
    Ok(published_periods(sec_client, last_month)
        .await?
        .iter()
        .map(ToString::to_string)
        .collect())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use sec::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, building it should always succeed")
    }

    #[tokio::test]
    async fn should_end_at_last_month_when_today_is_2026_11_07() {
        let sec_client = RoutingSecClient::new();
        let today = date(2026, 11, 7);

        let expected_result = Some("2026_10".to_string());

        let result = notes_periods(&sec_client, today)
            .await
            .expect("Given a client that answers 404 to each request, the list should always build")
            .pop();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_end_in_the_previous_year_when_today_is_in_january() {
        let sec_client = RoutingSecClient::new();
        let today = date(2027, 1, 31);

        let expected_result = Some("2026_12".to_string());

        let result = notes_periods(&sec_client, today)
            .await
            .expect("Given a client that answers 404 to each request, the list should always build")
            .pop();

        assert_eq!(result, expected_result);
    }
}
