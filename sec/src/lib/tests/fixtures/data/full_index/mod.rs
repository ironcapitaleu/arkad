/// The full index `form.idx` of 2026 Q3, cut to its 10 header lines and 6 of its 302,525 rows.
///
/// Downloaded from the SEC on 2026-11-13. The rows keep their order and their bytes. Five rows
/// hold a selected form: four with the date filed 2026-09-30, the rows of
/// [`SAMPLE_DAILY_INDEX_2026_09_30`](super::daily_index::SAMPLE_DAILY_INDEX_2026_09_30), and one
/// with 2026-09-29.
pub const SAMPLE_FULL_INDEX_2026_Q3: &str = include_str!("2026_QTR3_form.idx");

/// The full index `form.idx` of 2026 Q4, cut to its 10 header lines and 5 of its rows.
///
/// Downloaded from the SEC on 2026-11-13, when the file held the dates through 2026-11-12. The
/// rows keep their order and their bytes. Four rows hold a selected form: three with the date
/// filed 2026-10-01, the rows of
/// [`SAMPLE_DAILY_INDEX_2026_10_01`](super::daily_index::SAMPLE_DAILY_INDEX_2026_10_01), and one
/// with 2026-10-02.
pub const SAMPLE_FULL_INDEX_2026_Q4: &str = include_str!("2026_QTR4_form.idx");
