//! # Daily-Index Listing
//!
//! Provides [`listed_index_dates`], the parser for the dates in the listing of a daily-index
//! quarter folder, `daily-index/YYYY/QTRn/index.json`.
//!
//! The listing has the shape `{"directory":{"item":[{"name":"form.20261027.idx",…},…]}}`. The
//! parser reads the date of each name of the form `form.YYYYMMDD.idx`. It ignores every other
//! name, for example `company.20261027.idx` and the 1998 name `form.981027.idx`.

use chrono::NaiveDate;
use serde_json::Value;

/// Returns the dates of the daily indexes of the form `form.YYYYMMDD.idx` in a listing.
///
/// A listing without the path `directory.item` gives no dates.
///
/// # Examples
///
/// ```
/// use chrono::NaiveDate;
/// use sec::shared::index_date::listing::listed_index_dates;
///
/// let listing = serde_json::json!({ "directory": { "item": [
///     { "name": "company.20261027.idx" },
///     { "name": "form.20261027.idx" },
/// ] } });
///
/// let expected_result = vec![NaiveDate::from_ymd_opt(2026, 10, 27).expect("A hardcoded date should be valid")];
///
/// let result = listed_index_dates(&listing);
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn listed_index_dates(listing: &Value) -> Vec<NaiveDate> {
    listing["directory"]["item"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item["name"].as_str())
        .filter_map(index_date_of_file_name)
        .collect()
}

fn index_date_of_file_name(name: &str) -> Option<NaiveDate> {
    let date = name.strip_prefix("form.")?.strip_suffix(".idx")?;
    if date.len() != 8 || !date.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    NaiveDate::parse_from_str(date, "%Y%m%d").ok()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded valid date, construction should always succeed")
    }

    fn listing(names: &[&str]) -> Value {
        let items: Vec<Value> = names.iter().map(|name| json!({ "name": name })).collect();
        json!({ "directory": { "item": items } })
    }

    #[test]
    fn should_return_the_dates_of_all_form_indexes_in_listing_order() {
        let expected_result = vec![date(2026, 10, 27), date(2026, 10, 28)];

        let result = listed_index_dates(&listing(&["form.20261027.idx", "form.20261028.idx"]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_ignore_the_names_of_other_index_types() {
        let expected_result: Vec<NaiveDate> = Vec::new();

        let result = listed_index_dates(&listing(&[
            "company.20261027.idx",
            "master.20261027.idx",
            "crawler.20261027.idx",
            "sitemap.20261027.xml",
        ]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_ignore_the_two_digit_year_name_of_1998() {
        let expected_result = vec![date(1998, 10, 27)];

        let result = listed_index_dates(&listing(&["form.981027.idx", "form.19981027.idx"]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_ignore_a_name_whose_date_does_not_exist() {
        let expected_result: Vec<NaiveDate> = Vec::new();

        let result = listed_index_dates(&listing(&["form.20260230.idx"]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_dates_when_the_listing_has_no_item_array() {
        let expected_result: Vec<NaiveDate> = Vec::new();

        let result = listed_index_dates(&json!({ "directory": {} }));

        assert_eq!(result, expected_result);
    }
}
