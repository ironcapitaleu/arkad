/// The folder index of filing `0000003545-26-000047`, an 8-K with inline XBRL.
///
/// Downloaded from the SEC on 2026-10-31. It names `alco-20260908_htm.xml` and
/// `alco-20260908_lab.xml`.
pub const INLINE_XBRL_FOLDER_INDEX: &[u8] = include_bytes!("0000003545-26-000047-index.json");

/// The folder index of filing `0000320193-17-000070`, a 10-K of 2017 without inline XBRL.
///
/// Downloaded from the SEC on 2026-10-31. It names `aapl-20170930.xml` and the four linkbases.
pub const PLAIN_XBRL_FOLDER_INDEX: &[u8] = include_bytes!("0000320193-17-000070-index.json");
