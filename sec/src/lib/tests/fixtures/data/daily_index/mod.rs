/// The daily index of 2026-10-27, cut to its 11 header lines and 19 of its 5,071 rows.
///
/// Downloaded from the SEC on 2026-10-30. The rows keep their order and their bytes. Row 10 is
/// the second filer of a filing with two filers. The last row has a date filed of 2026-09-29.
pub const SAMPLE_DAILY_INDEX: &str = include_str!("form.20261027.idx");
