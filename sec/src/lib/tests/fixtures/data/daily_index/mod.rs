/// The daily index of 2026-10-27, cut to its 11 header lines and 19 of its 5,071 rows.
///
/// Downloaded from the SEC on 2026-10-30. The rows keep their order and their bytes. Row 10 is
/// the second filer of a filing with two filers. The last row has a date filed of 2026-09-29.
pub const SAMPLE_DAILY_INDEX: &str = include_str!("form.20261027.idx");

/// The daily index of 2026-09-30, the last day of a quarter, cut to its 11 header lines and 5 of
/// its 3,594 rows.
///
/// Downloaded from the SEC on 2026-11-13. The rows keep their order and their bytes. Four rows
/// hold a selected form.
pub const SAMPLE_DAILY_INDEX_2026_09_30: &str = include_str!("form.20260930.idx");

/// The daily index of 2026-10-01, the first day of a quarter, cut to its 11 header lines and 4 of
/// its 5,527 rows.
///
/// Downloaded from the SEC on 2026-11-13. The rows keep their order and their bytes. Three rows
/// hold a selected form.
pub const SAMPLE_DAILY_INDEX_2026_10_01: &str = include_str!("form.20261001.idx");
