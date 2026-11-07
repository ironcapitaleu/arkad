//! # Notes Periods
//!
//! Provides [`notes_periods`], which lists the Notes periods that the backfill run asks for, and
//! [`renamed_files`], which finds the links whose file name the run does not request.

use chrono::{Datelike, Months, NaiveDate};
use sec::shared::notes_file_list::{ListedNotesFile, NotesFileList};

/// The number of months between today and the last Notes period that the run asks for.
///
/// The SEC posts the file of month M during month M+1. On 2026-10-29, the SEC had posted
/// `2026_09`.
pub const PUBLICATION_LAG_MONTHS: u32 = 1;

/// Returns the period of every file in `files` that ends in or before the month
/// [`PUBLICATION_LAG_MONTHS`] before `today`, in increasing order.
///
/// The page lists only the files that exist now. So a monthly file that the SEC merged into a
/// quarterly file is not in the list, and the run does not ask for it.
pub fn notes_periods(files: &NotesFileList, today: NaiveDate) -> Vec<String> {
    let Some(last_month) = today.checked_sub_months(Months::new(PUBLICATION_LAG_MONTHS)) else {
        return Vec::new();
    };
    files
        .files()
        .iter()
        .map(ListedNotesFile::period)
        .filter(|period| {
            let last_day = period.last_day();
            (last_day.year(), last_day.month()) <= (last_month.year(), last_month.month())
        })
        .map(|period| period.to_string())
        .collect()
}

/// Returns the files in `files` whose linked file name differs from the file name that the run
/// requests for the period.
pub fn renamed_files(files: &NotesFileList) -> Vec<&ListedNotesFile> {
    files
        .files()
        .iter()
        .filter(|file| file.file_name() != file.period().file_name())
        .collect()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use sec::tests::fixtures::data::notes_data_sets_page::NOTES_DATA_SETS_PAGE;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, building it should always succeed")
    }

    fn stored_page() -> NotesFileList {
        NotesFileList::parse(NOTES_DATA_SETS_PAGE)
            .expect("Given the stored page of 2026-11-07, parsing should always succeed")
    }

    fn page_with(file_names: &[&str]) -> NotesFileList {
        let page = file_names
            .iter()
            .map(|file_name| {
                format!(
                    r#"<a href="/files/dera/data/financial-statement-notes-data-sets/{file_name}">"#
                )
            })
            .collect::<Vec<_>>()
            .concat();
        NotesFileList::parse(&page)
            .expect("Given links with valid periods, parsing should always succeed")
    }

    #[test]
    fn should_list_every_period_of_the_page_when_today_is_2026_11_07() {
        let files = stored_page();
        let today = date(2026, 11, 7);
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

        let result = notes_periods(&files, today);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_out_the_current_month_when_the_page_already_lists_it() {
        let files = page_with(&["2026_09_notes.zip", "2026_10_notes.zip"]);
        let today = date(2026, 10, 31);

        let expected_result = vec!["2026_09"];

        let result = notes_periods(&files, today);

        assert_eq!(result, expected_result);
    }

    // The fixed bounds listed `2025_10` to `2025_12` and never `2025q4`. After the SEC merges the
    // three months, it removes them, and a run without them failed.
    #[test]
    fn should_ask_for_the_merged_quarter_and_not_its_months_when_the_page_lists_only_the_quarter() {
        let files = page_with(&["2025q3_notes.zip", "2025q4_notes.zip", "2026_01_notes.zip"]);
        let today = date(2027, 2, 10);

        let expected_result = vec!["2025q3", "2025q4", "2026_01"];

        let result = notes_periods(&files, today);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_end_in_the_previous_year_when_today_is_in_january() {
        let files = page_with(&["2026_12_notes.zip", "2027_01_notes.zip"]);
        let today = date(2027, 1, 31);

        let expected_result = vec!["2026_12"];

        let result = notes_periods(&files, today);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_out_a_quarter_when_it_ends_after_last_month() {
        let files = page_with(&["2027q1_notes.zip"]);
        let today = date(2027, 3, 15);

        let expected_result: Vec<String> = Vec::new();

        let result = notes_periods(&files, today);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_find_no_renamed_file_for_the_stored_page_of_2026_11_07() {
        let files = stored_page();

        let expected_result: Vec<&ListedNotesFile> = Vec::new();

        let result = renamed_files(&files);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_file_when_its_linked_name_breaks_the_pattern() {
        let files = page_with(&["2026_09_notes.zip", "2026_10_notes_1.zip"]);

        let expected_result = vec!["2026_10_notes_1.zip"];

        let result: Vec<&str> = renamed_files(&files)
            .into_iter()
            .map(ListedNotesFile::file_name)
            .collect();

        assert_eq!(result, expected_result);
    }
}
