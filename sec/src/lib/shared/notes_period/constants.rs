//! # Notes Period Constants
//!
//! Range limits of the SEC Financial Statement and Notes files, and the file names that break the
//! naming pattern.

use super::month::Month;
use super::quarter::Quarter;
use super::year::Year;

/// The first year with a Notes file.
pub const FIRST_YEAR: Year = Year(2009);

/// The year and quarter of the last quarterly Notes file.
///
/// After this quarter, the SEC switched to monthly Notes files. The date does not change, so it is
/// a constant.
pub const LAST_QUARTER: (Year, Quarter) = (Year(2025), Quarter::Q3);

/// The year and month of the first monthly Notes file.
pub const FIRST_MONTH: (Year, Month) = (Year(2025), Month(chrono::Month::October));

/// The text after the period in every Notes file name that follows the pattern.
pub const FILE_NAME_SUFFIX: &str = "_notes.zip";

/// The periods whose linked file name breaks the pattern, and the linked file name.
///
/// For these periods, the file name of the pattern also returns HTTP 200. It serves an older
/// version of the file, last modified on 2020-12-22.
pub const FILE_NAME_OVERRIDES: [(&str, &str); 3] = [
    ("2010q1", "2010q1_notes_1.zip"),
    ("2010q2", "2010q2_notes_0.zip"),
    ("2010q3", "2010q3_notes_0.zip"),
];
