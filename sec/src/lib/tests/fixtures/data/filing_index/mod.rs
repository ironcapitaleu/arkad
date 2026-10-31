use std::collections::HashMap;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::shared::status_code::StatusCode;

/// The folder index of filing `0000003545-26-000047`, an 8-K with inline XBRL.
///
/// Downloaded from the SEC on 2026-10-31. It names `alco-20260908_htm.xml` and
/// `alco-20260908_lab.xml`.
pub const INLINE_XBRL_FOLDER_INDEX: &[u8] = include_bytes!("0000003545-26-000047-index.json");

/// The URL of [`INLINE_XBRL_FOLDER_INDEX`].
pub const INLINE_XBRL_FOLDER_INDEX_URL: &str =
    "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json";

/// The folder index of filing `0000320193-17-000070`, a 10-K of 2017 without inline XBRL.
///
/// Downloaded from the SEC on 2026-10-31. It names `aapl-20170930.xml` and the four linkbases.
pub const PLAIN_XBRL_FOLDER_INDEX: &[u8] = include_bytes!("0000320193-17-000070-index.json");

/// The URL of [`PLAIN_XBRL_FOLDER_INDEX`].
pub const PLAIN_XBRL_FOLDER_INDEX_URL: &str =
    "https://www.sec.gov/Archives/edgar/data/320193/000032019317000070/index.json";

/// Returns a `200 OK` JSON response from `url` with `body`, as the SEC sends a folder index.
///
/// # Panics
///
/// Panics if `url` is not a valid URL or `body` is not JSON.
#[must_use]
pub fn folder_index_response(url: &str, body: &[u8]) -> SecResponse {
    SecResponse::from_parts(
        url.parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed"),
        Headers::new(HashMap::new()),
        ContentType::Json,
        StatusCode::Ok,
        body.to_vec(),
    )
    .expect("Given a JSON body, building the response should always succeed")
}
