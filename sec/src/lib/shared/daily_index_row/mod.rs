//! # Daily Index Row
//!
//! Provides [`DailyIndexRow`], one row of the EDGAR daily index `form.YYYYMMDD.idx`, and
//! [`parse_daily_index`], which reads the text of a daily index into its rows.
//!
//! The daily index is fixed-width text. Header lines come first and end with a line of dashes.
//! Each row after it holds a form, a company name, a CIK, a date filed, and a file name. The
//! parser returns every row. `SelectFilings` picks the forms and keeps one row per accession
//! number.
//!
//! ## Modules
//!
//! - [`daily_index_error`]: The [`DailyIndexError`] returned when the text or one of its lines is
//!   malformed.

use chrono::NaiveDate;
use serde::Serialize;

use crate::shared::cik::Cik;
use crate::shared::financial::accession_number::AccessionNumber;

pub mod daily_index_error;

pub use daily_index_error::{DailyIndexError, InvalidDailyIndexLineReason};

/// The start of the line of dashes that ends the header.
const HEADER_END_PREFIX: &str = "---";

/// The width of the form column. Form text can hold spaces, as in `DEF 14A`.
const FORM_COLUMN_WIDTH: usize = 17;

/// The format of the date filed field, as in `20261027`.
const FILED_DATE_FORMAT: &str = "%Y%m%d";

/// The extension of the file name field, as in `edgar/data/320193/0000320193-26-000106.txt`.
const FILE_NAME_EXTENSION: &str = ".txt";

/// One row of an EDGAR daily index: the form text, the CIK, the date filed, and the accession
/// number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct DailyIndexRow {
    form_text: String,
    cik: Cik,
    filed_date: NaiveDate,
    accession_number: AccessionNumber,
}

/// Parses the text of an EDGAR daily index into one [`DailyIndexRow`] per row line.
///
/// Skips every line up to the line of dashes, and skips blank lines after it. Accepts `\n` and
/// `\r\n` line endings.
///
/// # Errors
///
/// Returns [`DailyIndexError::MissingDailyIndexHeader`] if no line starts with `---`.
/// Returns [`DailyIndexError::InvalidDailyIndexLine`] with the line number for the first row line
/// that is malformed.
///
/// # Examples
///
/// ```
/// use sec::shared::daily_index_row::parse_daily_index;
///
/// let text = "Form Type   Company Name\n---\n\
///     10-K             APPLE INC                                                     320193      20261030    edgar/data/320193/0000320193-26-000106.txt";
///
/// let expected_result = 1;
///
/// let result = parse_daily_index(text)
///     .expect("Given a hardcoded well-formed row, parsing should always succeed")
///     .len();
///
/// assert_eq!(result, expected_result);
/// ```
pub fn parse_daily_index(text: &str) -> Result<Vec<DailyIndexRow>, DailyIndexError> {
    let mut lines = text.lines().enumerate();
    lines
        .by_ref()
        .find(|(_, line)| line.starts_with(HEADER_END_PREFIX))
        .ok_or(DailyIndexError::MissingDailyIndexHeader)?;

    lines
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            DailyIndexRow::from_line(line).map_err(|reason| {
                DailyIndexError::InvalidDailyIndexLine {
                    line_number: index + 1,
                    reason,
                }
            })
        })
        .collect()
}

impl DailyIndexRow {
    /// Creates a new [`DailyIndexRow`].
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
    fn from_line(line: &str) -> Result<Self, InvalidDailyIndexLineReason> {
        let mut fields = line.split_whitespace().rev();
        let (Some(file_name), Some(filed_date), Some(cik), Some(_)) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(InvalidDailyIndexLineReason::MissingField);
        };
        let form_text = line
            .get(..FORM_COLUMN_WIDTH)
            .map(str::trim)
            .filter(|form_text| !form_text.is_empty())
            .ok_or(InvalidDailyIndexLineReason::MissingField)?;

        let cik = Cik::new(cik).map_err(|_| InvalidDailyIndexLineReason::InvalidCik {
            cik: cik.to_string(),
        })?;
        let filed_date =
            NaiveDate::parse_from_str(filed_date, FILED_DATE_FORMAT).map_err(|_| {
                InvalidDailyIndexLineReason::InvalidFiledDate {
                    filed_date: filed_date.to_string(),
                }
            })?;
        let accession_number = accession_number_of(file_name).ok_or_else(|| {
            InvalidDailyIndexLineReason::InvalidFileName {
                file_name: file_name.to_string(),
            }
        })?;

        Ok(Self::new(
            form_text.to_string(),
            cik,
            filed_date,
            accession_number,
        ))
    }
}

/// Returns the accession number at the end of a file name such as
/// `edgar/data/320193/0000320193-26-000106.txt`, or `None` if it has another shape.
fn accession_number_of(file_name: &str) -> Option<AccessionNumber> {
    let accession = file_name
        .rsplit('/')
        .next()?
        .strip_suffix(FILE_NAME_EXTENSION)?;
    let is_accession_shape = accession.len() == 20
        && accession
            .char_indices()
            .all(|(index, character)| match index {
                10 | 13 => character == '-',
                _ => character.is_ascii_digit(),
            });

    is_accession_shape.then(|| AccessionNumber::new(accession))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::data::daily_index::SAMPLE_DAILY_INDEX;

    const FILE_NAME: &str = "edgar/data/320193/0000320193-26-000106.txt";

    /// Builds a row line with the column widths of a real daily index.
    fn row_line(form_text: &str, cik: &str, filed_date: &str, file_name: &str) -> String {
        format!(
            "{form_text:<17}{:<62}{cik:<12}{filed_date:<12}{file_name}",
            "ACME CORP"
        )
    }

    /// Parses an index with one header line of dashes and the given row line.
    fn parse_row_line(line: &str) -> Result<Vec<DailyIndexRow>, DailyIndexError> {
        parse_daily_index(&format!("---\n{line}"))
    }

    fn row(form_text: &str, cik: &str, accession_number: &str) -> DailyIndexRow {
        DailyIndexRow::new(
            form_text.to_string(),
            Cik::new(cik).expect("Given a hardcoded valid CIK, parsing should always succeed"),
            NaiveDate::from_ymd_opt(2026, 10, 27)
                .expect("Given a hardcoded calendar date, the date should always build"),
            AccessionNumber::new(accession_number),
        )
    }

    fn invalid_second_line(
        reason: InvalidDailyIndexLineReason,
    ) -> Result<Vec<DailyIndexRow>, DailyIndexError> {
        Err(DailyIndexError::InvalidDailyIndexLine {
            line_number: 2,
            reason,
        })
    }

    fn parse_sample_daily_index() -> Vec<DailyIndexRow> {
        parse_daily_index(SAMPLE_DAILY_INDEX)
            .expect("Given the stored fixture of a real index, parsing should always succeed")
    }

    #[test]
    fn should_return_one_row_per_row_line_for_sample_daily_index() {
        let expected_result = 19;

        let result = parse_sample_daily_index().len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_second_filer_of_multi_filer_filing_for_sample_daily_index() {
        let expected_result = row("10-Q", "201533", "0000811156-26-000032");

        let result = parse_sample_daily_index().swap_remove(9);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_rows_when_index_has_only_header() {
        let expected_result = Ok(vec![]);

        let result = parse_daily_index("Form Type   Company Name\n---\n");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_inner_space_of_form_text_when_form_text_has_two_words() {
        let expected_result = Ok(vec![row("DEF 14A", "320193", "0000320193-26-000106")]);

        let result = parse_row_line(&row_line("DEF 14A", "320193", "20261027", FILE_NAME));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_row_when_line_ends_with_crlf() {
        let line = row_line("10-K/A", "320193", "20261027", FILE_NAME);
        let expected_result = Ok(vec![row("10-K/A", "320193", "0000320193-26-000106")]);

        let result = parse_daily_index(&format!("---\r\n{line}\r\n"));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_blank_line_when_blank_line_follows_header() {
        let line = row_line("8-K", "320193", "20261027", FILE_NAME);
        let expected_result = Ok(vec![row("8-K", "320193", "0000320193-26-000106")]);

        let result = parse_daily_index(&format!("---\n \n{line}"));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_header_when_index_has_no_line_of_dashes() {
        let expected_result = Err(DailyIndexError::MissingDailyIndexHeader);

        let result = parse_daily_index(&row_line("10-K", "320193", "20261027", FILE_NAME));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_field_with_line_number_when_line_has_too_few_fields() {
        let expected_result = Err(DailyIndexError::InvalidDailyIndexLine {
            line_number: 3,
            reason: InvalidDailyIndexLineReason::MissingField,
        });

        let result = parse_daily_index("Description: Daily Index\n---\n10-K  320193  20261027");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_field_when_form_column_is_blank() {
        let expected_result = invalid_second_line(InvalidDailyIndexLineReason::MissingField);

        let result = parse_row_line(&row_line("", "320193", "20261027", FILE_NAME));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_cik_when_cik_is_not_a_number() {
        let expected_result = invalid_second_line(InvalidDailyIndexLineReason::InvalidCik {
            cik: "32O193".to_string(),
        });

        let result = parse_row_line(&row_line("10-K", "32O193", "20261027", FILE_NAME));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_filed_date_when_filed_date_is_not_a_calendar_date() {
        let expected_result = invalid_second_line(InvalidDailyIndexLineReason::InvalidFiledDate {
            filed_date: "20261332".to_string(),
        });

        let result = parse_row_line(&row_line("10-K", "320193", "20261332", FILE_NAME));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_file_name_when_file_name_has_no_txt_extension() {
        let file_name = "edgar/data/320193/0000320193-26-000106.htm";
        let expected_result = invalid_second_line(InvalidDailyIndexLineReason::InvalidFileName {
            file_name: file_name.to_string(),
        });

        let result = parse_row_line(&row_line("10-K", "320193", "20261027", file_name));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_file_name_when_accession_number_has_wrong_shape() {
        let file_name = "edgar/data/320193/000032019326000106.txt";
        let expected_result = invalid_second_line(InvalidDailyIndexLineReason::InvalidFileName {
            file_name: file_name.to_string(),
        });

        let result = parse_row_line(&row_line("10-K", "320193", "20261027", file_name));

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_daily_index_row() {
        implements_auto_traits::<DailyIndexRow>();
    }
}
