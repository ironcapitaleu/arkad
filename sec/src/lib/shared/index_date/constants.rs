//! # Index Date Constants
//!
//! The first date with a daily index of the form `form.YYYYMMDD.idx`, and the quarter folders of
//! the daily index.

use chrono::NaiveDate;

/// The first date whose daily index has the name `form.YYYYMMDD.idx`.
///
/// The daily index starts on 1994-07-01. Until 1998-05-17, the SEC named the files
/// `form.YYMMDD.idx`. From 1998-05-18 to 1998-12-31, both names exist. From 1999 on, only the
/// `YYYYMMDD` name exists. The SEC quarter listings showed this on 2026-10-30.
pub const FIRST_INDEX_DATE: NaiveDate = NaiveDate::from_ymd_opt(1998, 5, 18)
    .expect("Given a hardcoded valid date, construction should always succeed");

/// The quarter folders of the daily index, in the order of the quarters.
pub(crate) const QUARTER_FOLDERS: [&str; 4] = ["QTR1", "QTR2", "QTR3", "QTR4"];
