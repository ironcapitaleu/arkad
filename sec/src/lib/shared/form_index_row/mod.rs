//! # Form Index Row
//!
//! Provides [`FormIndexRow`], one row of an EDGAR form index, and [`parse_form_index`], which
//! reads the text of a form index into its rows. A form index lists filings sorted by form. The
//! SEC publishes one per day, the daily index `form.YYYYMMDD.idx`, and one per quarter, the full
//! index `form.idx`.
//!
//! A form index is fixed-width text. Header lines come first. The last header line starts with
//! `---`. Each row after it holds a form, a company name, a CIK, a date filed, and a file name. The
//! parser returns every row. [`select_filings`](crate::shared::filing_ref::select_filings) picks
//! the forms and keeps one row per accession number.
//!
//! Both files have the same layout. The date filed has the form `YYYYMMDD` in the daily index and
//! `YYYY-MM-DD` in the full index. The parser reads both. [`last_data_received`] reads the last
//! date that a form index covers from its header.
//!
//! ## Modules
//!
//! - [`form_index_error`]: The [`FormIndexError`] returned when the text or one of its lines is
//!   malformed.

use chrono::NaiveDate;
use serde::Serialize;

use crate::shared::cik::Cik;
use crate::shared::financial::accession_number::AccessionNumber;

pub mod form_index_error;

pub use form_index_error::{FormIndexError, InvalidFormIndexLineReason};

/// The start of the last header line, a line of dashes.
const HEADER_END_PREFIX: &str = "---";

/// The width of the form column. Form text can hold spaces, as in `DEF 14A`.
const FORM_COLUMN_WIDTH: usize = 17;

/// The format of the date filed field of a daily index, as in `20261027`.
const DAILY_FILED_DATE_FORMAT: &str = "%Y%m%d";

/// The length of the date filed field of a daily index.
const DAILY_FILED_DATE_LENGTH: usize = 8;

/// The format of the date filed field of a full index, as in `2026-10-27`.
const FULL_FILED_DATE_FORMAT: &str = "%Y-%m-%d";

/// The length of the date filed field of a full index.
const FULL_FILED_DATE_LENGTH: usize = 10;

/// The dash between the parts of the date filed field of a full index.
const FULL_FILED_DATE_SEPARATOR: u8 = b'-';

/// The positions of the two dashes in the date filed field of a full index.
const FULL_FILED_DATE_SEPARATOR_POSITIONS: [usize; 2] = [4, 7];

/// The extension of the file name field, as in `edgar/data/320193/0000320193-26-000106.txt`.
const FILE_NAME_EXTENSION: &str = ".txt";

/// The label of the header line that names the last date the index covers.
const LAST_DATA_RECEIVED_LABEL: &str = "Last Data Received:";

/// The format of the date of the header line `Last Data Received`. When parsing, `%B` also
/// accepts the short month name of the daily index.
const LAST_DATA_RECEIVED_FORMAT: &str = "%B %d, %Y";

/// One row of an EDGAR form index, daily or full: the form text, the CIK, the date filed, and the
/// accession number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct FormIndexRow {
    form_text: String,
    cik: Cik,
    filed_date: NaiveDate,
    accession_number: AccessionNumber,
}

/// Parses the text of an EDGAR daily index, or of the full index `form.idx` of a quarter, into
/// one [`FormIndexRow`] per row line.
///
/// Skips every line up to the first line that starts with `---`, and skips blank lines after it. Accepts `\n` and
/// `\r\n` line endings.
///
/// # Errors
///
/// Returns [`FormIndexError::MissingFormIndexHeader`] if no line starts with `---`.
/// Returns [`FormIndexError::InvalidFormIndexCik`] with the line number and the
/// [`CikError`](crate::shared::cik::CikError) for the first row line whose CIK fails validation.
/// Returns [`FormIndexError::InvalidFormIndexLine`] with the line number for the first row line
/// that is malformed in any other way.
///
/// # Examples
///
/// ```
/// use sec::shared::form_index_row::parse_form_index;
///
/// let text = "Form Type   Company Name\n---\n\
///     10-K             APPLE INC                                                     320193      20261030    edgar/data/320193/0000320193-26-000106.txt";
///
/// let expected_result = 1;
///
/// let result = parse_form_index(text)
///     .expect("Given a hardcoded well-formed row, parsing should always succeed")
///     .len();
///
/// assert_eq!(result, expected_result);
/// ```
pub fn parse_form_index(text: &str) -> Result<Vec<FormIndexRow>, FormIndexError> {
    let mut lines = text.lines().enumerate();
    lines
        .by_ref()
        .find(|(_, line)| line.starts_with(HEADER_END_PREFIX))
        .ok_or(FormIndexError::MissingFormIndexHeader)?;

    lines
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| FormIndexRow::from_line(index + 1, line))
        .collect()
}

/// Returns the date of the header line `Last Data Received` of a form index, or `None` if the
/// header has no such line or its date does not parse.
///
/// The full index of a quarter lists the dates filed up to this date. A later date has no rows
/// yet. The date has the form `November 12, 2026` in the full index and `Sep 30, 2026` in the
/// daily index. Reads the header lines only, up to the first line that starts with `---`.
///
/// # Examples
///
/// ```
/// use chrono::NaiveDate;
/// use sec::shared::form_index_row::last_data_received;
///
/// let text = b"Last Data Received:    November 12, 2026\nForm Type   Company Name\n---\n";
///
/// let expected_result = NaiveDate::from_ymd_opt(2026, 11, 12);
///
/// let result = last_data_received(text);
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn last_data_received(text: &[u8]) -> Option<NaiveDate> {
    text.split(|byte| *byte == b'\n')
        .map(String::from_utf8_lossy)
        .take_while(|line| !line.starts_with(HEADER_END_PREFIX))
        .find_map(|line| {
            let date = line.strip_prefix(LAST_DATA_RECEIVED_LABEL)?.trim();
            NaiveDate::parse_from_str(date, LAST_DATA_RECEIVED_FORMAT).ok()
        })
}

impl FormIndexRow {
    /// Creates a new [`FormIndexRow`].
    #[must_use]
    pub const fn new(
        form_text: String,
        cik: Cik,
        filed_date: NaiveDate,
        accession_number: AccessionNumber,
    ) -> Self {
        Self {
            form_text,
            cik,
            filed_date,
            accession_number,
        }
    }

    /// Returns the form text as the SEC writes it, without padding, for example `10-K/A`.
    #[must_use]
    pub fn form_text(&self) -> &str {
        &self.form_text
    }

    /// Returns the CIK of the filer.
    #[must_use]
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }

    /// Returns the date filed.
    #[must_use]
    pub const fn filed_date(&self) -> NaiveDate {
        self.filed_date
    }

    /// Returns the accession number, taken from the file name.
    #[must_use]
    pub const fn accession_number(&self) -> &AccessionNumber {
        &self.accession_number
    }

    /// Parses one row line. Reads the form from its column, and the last three fields from the
    /// end of the line, so a company name that overflows its column still parses.
    fn from_line(line_number: usize, line: &str) -> Result<Self, FormIndexError> {
        let invalid_line = |reason| FormIndexError::InvalidFormIndexLine {
            line_number,
            reason,
        };
        let mut fields = line.split_whitespace().rev();
        let (Some(file_name), Some(filed_date), Some(cik), Some(_)) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(invalid_line(InvalidFormIndexLineReason::MissingField));
        };
        let form_text = line
            .get(..FORM_COLUMN_WIDTH)
            .map(str::trim)
            .filter(|form_text| !form_text.is_empty())
            .ok_or_else(|| invalid_line(InvalidFormIndexLineReason::MissingField))?;

        let cik = Cik::new(cik).map_err(|source| FormIndexError::InvalidFormIndexCik {
            line_number,
            source,
        })?;
        let filed_date = filed_date_of(filed_date).ok_or_else(|| {
            invalid_line(InvalidFormIndexLineReason::InvalidFiledDate {
                filed_date: filed_date.to_string(),
            })
        })?;
        let accession_number = accession_number_of(file_name).ok_or_else(|| {
            invalid_line(InvalidFormIndexLineReason::InvalidFileName {
                file_name: file_name.to_string(),
            })
        })?;

        Ok(Self::new(
            form_text.to_string(),
            cik,
            filed_date,
            accession_number,
        ))
    }
}

/// Returns the date of a text such as `20261027` or `2026-10-27`, or `None` if it has another
/// shape.
///
/// Checks the length and the characters first, because both formats also accept a 1-digit month
/// or day, as in `2026102`, and a sign before the year.
fn filed_date_of(text: &str) -> Option<NaiveDate> {
    let format = match text.len() {
        DAILY_FILED_DATE_LENGTH if text.bytes().all(|byte| byte.is_ascii_digit()) => {
            DAILY_FILED_DATE_FORMAT
        }
        FULL_FILED_DATE_LENGTH if has_full_filed_date_shape(text) => FULL_FILED_DATE_FORMAT,
        _ => return None,
    };

    NaiveDate::parse_from_str(text, format).ok()
}

/// Returns `true` if `text` has a `-` at the two positions of `2026-10-27` and a digit at every
/// other position.
fn has_full_filed_date_shape(text: &str) -> bool {
    text.bytes().enumerate().all(|(position, byte)| {
        if FULL_FILED_DATE_SEPARATOR_POSITIONS.contains(&position) {
            byte == FULL_FILED_DATE_SEPARATOR
        } else {
            byte.is_ascii_digit()
        }
    })
}

/// Returns the accession number at the end of a file name such as
/// `edgar/data/320193/0000320193-26-000106.txt`, or `None` if it has another shape.
fn accession_number_of(file_name: &str) -> Option<AccessionNumber> {
    let accession = file_name
        .rsplit('/')
        .next()?
        .strip_suffix(FILE_NAME_EXTENSION)?;

    AccessionNumber::new(accession).ok()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::cik::{CikError, InvalidCikReason};
    use crate::tests::fixtures::data::daily_index::SAMPLE_DAILY_INDEX;
    use crate::tests::fixtures::data::full_index::SAMPLE_FULL_INDEX_2026_Q3;

    const FILE_NAME: &str = "edgar/data/320193/0000320193-26-000106.txt";

    /// The index of row 10 of the fixture, the second filer of a filing with two filers.
    const SECOND_FILER_ROW_INDEX: usize = 9;

    /// Builds a row line with the column widths of a real daily index.
    fn row_line(form_text: &str, cik: &str, filed_date: &str, file_name: &str) -> String {
        format!(
            "{form_text:<17}{:<62}{cik:<12}{filed_date:<12}{file_name}",
            "ACME CORP"
        )
    }

    #[test]
    fn should_return_the_last_data_received_for_sample_full_index() {
        let text = SAMPLE_FULL_INDEX_2026_Q3.as_bytes();

        let expected_result = NaiveDate::from_ymd_opt(2026, 9, 30);

        let result = last_data_received(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_last_data_received_for_sample_daily_index() {
        let text = SAMPLE_DAILY_INDEX.as_bytes();

        let expected_result = NaiveDate::from_ymd_opt(2026, 10, 27);

        let result = last_data_received(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_the_header_has_no_last_data_received_line() {
        let text = b"Form Type   Company Name\n---\n";

        let expected_result = None;

        let result = last_data_received(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_the_last_data_received_line_comes_after_the_header() {
        let text = b"---\nLast Data Received:    November 12, 2026\n";

        let expected_result = None;

        let result = last_data_received(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_the_date_of_last_data_received_does_not_parse() {
        let text = b"Last Data Received:    2026-11-12\n---\n";

        let expected_result = None;

        let result = last_data_received(text);

        assert_eq!(result, expected_result);
    }

    /// Parses an index with one header line of dashes and the given row line.
    fn parse_row_line(line: &str) -> Result<Vec<FormIndexRow>, FormIndexError> {
        parse_form_index(&format!("---\n{line}"))
    }

    fn row(form_text: &str, cik: &str, accession_number: &str) -> FormIndexRow {
        FormIndexRow::new(
            form_text.to_string(),
            Cik::new(cik).expect("Given a hardcoded valid CIK, parsing should always succeed"),
            NaiveDate::from_ymd_opt(2026, 10, 27)
                .expect("Given a hardcoded calendar date, the date should always build"),
            AccessionNumber::new(accession_number).expect("Given a hardcoded accession number of the right shape, creation should always succeed"),
        )
    }

    fn invalid_second_line(
        reason: InvalidFormIndexLineReason,
    ) -> Result<Vec<FormIndexRow>, FormIndexError> {
        Err(FormIndexError::InvalidFormIndexLine {
            line_number: 2,
            reason,
        })
    }

    #[test]
    fn should_return_one_row_per_row_line_for_sample_daily_index() {
        let text = SAMPLE_DAILY_INDEX;

        let expected_result = 19;

        let result = parse_form_index(text)
            .expect("Given the stored fixture of a real index, parsing should always succeed")
            .len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_second_filer_of_multi_filer_filing_for_sample_daily_index() {
        let text = SAMPLE_DAILY_INDEX;

        let expected_result = row("10-Q", "201533", "0000811156-26-000032");

        let result = parse_form_index(text)
            .expect("Given the stored fixture of a real index, parsing should always succeed")
            .swap_remove(SECOND_FILER_ROW_INDEX);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_rows_when_index_has_only_header() {
        let text = "Form Type   Company Name\n---\n";

        let expected_result = Ok(vec![]);

        let result = parse_form_index(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_inner_space_of_form_text_when_form_text_has_two_words() {
        let line = row_line("DEF 14A", "320193", "20261027", FILE_NAME);

        let expected_result = Ok(vec![row("DEF 14A", "320193", "0000320193-26-000106")]);

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_row_when_line_ends_with_crlf() {
        let line = row_line("10-K/A", "320193", "20261027", FILE_NAME);
        let text = format!("---\r\n{line}\r\n");

        let expected_result = Ok(vec![row("10-K/A", "320193", "0000320193-26-000106")]);

        let result = parse_form_index(&text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_blank_line_when_blank_line_follows_header() {
        let line = row_line("8-K", "320193", "20261027", FILE_NAME);
        let text = format!("---\n \n{line}");

        let expected_result = Ok(vec![row("8-K", "320193", "0000320193-26-000106")]);

        let result = parse_form_index(&text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_header_when_index_has_no_line_of_dashes() {
        let line = row_line("10-K", "320193", "20261027", FILE_NAME);

        let expected_result = Err(FormIndexError::MissingFormIndexHeader);

        let result = parse_form_index(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_field_with_line_number_when_line_has_too_few_fields() {
        let text = "Description: Daily Index\n---\n10-K  320193  20261027";

        let expected_result = Err(FormIndexError::InvalidFormIndexLine {
            line_number: 3,
            reason: InvalidFormIndexLineReason::MissingField,
        });

        let result = parse_form_index(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_field_when_form_column_is_blank() {
        let line = row_line("", "320193", "20261027", FILE_NAME);

        let expected_result = invalid_second_line(InvalidFormIndexLineReason::MissingField);

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_cik_with_cik_error_when_cik_is_not_a_number() {
        let line = row_line("10-K", "32O193", "20261027", FILE_NAME);

        let expected_result = Err(FormIndexError::InvalidFormIndexCik {
            line_number: 2,
            source: CikError::new(InvalidCikReason::ContainsNonNumericCharacters, "32O193"),
        });

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_cik_with_cik_error_when_cik_has_eleven_digits() {
        let line = row_line("10-K", "12345678901", "20261027", FILE_NAME);

        let expected_result = Err(FormIndexError::InvalidFormIndexCik {
            line_number: 2,
            source: CikError::new(
                InvalidCikReason::MaxLengthExceeded { cik_length: 11 },
                "12345678901",
            ),
        });

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_filed_date_when_filed_date_is_not_a_calendar_date() {
        let line = row_line("10-K", "320193", "20261332", FILE_NAME);

        let expected_result = invalid_second_line(InvalidFormIndexLineReason::InvalidFiledDate {
            filed_date: "20261332".to_string(),
        });

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_filed_date_when_filed_date_has_seven_digits() {
        let line = row_line("10-K", "320193", "2026102", FILE_NAME);

        let expected_result = invalid_second_line(InvalidFormIndexLineReason::InvalidFiledDate {
            filed_date: "2026102".to_string(),
        });

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_filed_date_when_row_has_the_date_form_of_a_full_index() {
        let line = row_line("10-K", "320193", "2026-10-27", FILE_NAME);

        let expected_result = Ok(vec![row("10-K", "320193", "0000320193-26-000106")]);

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_filed_date_when_dashed_filed_date_has_a_one_digit_day() {
        let line = row_line("10-K", "320193", "2026-10-7", FILE_NAME);

        let expected_result = invalid_second_line(InvalidFormIndexLineReason::InvalidFiledDate {
            filed_date: "2026-10-7".to_string(),
        });

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_filed_date_when_dashed_filed_date_starts_with_a_sign() {
        let line = row_line("10-K", "320193", "-026-10-27", FILE_NAME);

        let expected_result = invalid_second_line(InvalidFormIndexLineReason::InvalidFiledDate {
            filed_date: "-026-10-27".to_string(),
        });

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_filed_date_when_ten_digits_hold_no_dash() {
        let line = row_line("10-K", "320193", "2026102700", FILE_NAME);

        let expected_result = invalid_second_line(InvalidFormIndexLineReason::InvalidFiledDate {
            filed_date: "2026102700".to_string(),
        });

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_one_row_per_row_line_for_sample_full_index() {
        let text = SAMPLE_FULL_INDEX_2026_Q3;

        let expected_result = 6;

        let result = parse_form_index(text)
            .expect("Given the stored fixture of a real index, parsing should always succeed")
            .len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_row_when_company_name_overflows_its_column() {
        let line = format!(
            "{:<17}{} 320193 20261027 {FILE_NAME}",
            "10-K",
            "A".repeat(70)
        );

        let expected_result = Ok(vec![row("10-K", "320193", "0000320193-26-000106")]);

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_file_name_when_file_name_has_no_txt_extension() {
        let file_name = "edgar/data/320193/0000320193-26-000106.htm";
        let line = row_line("10-K", "320193", "20261027", file_name);

        let expected_result = invalid_second_line(InvalidFormIndexLineReason::InvalidFileName {
            file_name: file_name.to_string(),
        });

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_file_name_when_accession_number_has_wrong_shape() {
        let file_name = "edgar/data/320193/000032019326000106.txt";
        let line = row_line("10-K", "320193", "20261027", file_name);

        let expected_result = invalid_second_line(InvalidFormIndexLineReason::InvalidFileName {
            file_name: file_name.to_string(),
        });

        let result = parse_row_line(&line);

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_form_index_row() {
        implements_auto_traits::<FormIndexRow>();
    }
}
