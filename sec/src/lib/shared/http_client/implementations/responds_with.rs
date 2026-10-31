//! # Endpoint Bodies
//!
//! Implements [`RespondsWith`] for each SEC endpoint, so each endpoint names the body kind of
//! its response.

use crate::shared::http_client::RespondsWith;
use crate::shared::request::implementations::sec_request::{
    CompanyFacts, DailyIndex, DailyIndexListing, NotesFile,
};
use crate::shared::response::implementations::sec_response::{JsonBody, ZipBody};

impl RespondsWith for CompanyFacts {
    type Body = JsonBody;

    const EXECUTE_STATE_NAME: &'static str = "Execute SEC Request";
}

impl RespondsWith for NotesFile {
    type Body = ZipBody;

    const EXECUTE_STATE_NAME: &'static str = "Download Notes Archive";
}

// Deviation: the daily index is fixed-width text, not a ZIP. `ZipBody` is reused because it runs
// the checks the index needs: status 200, `application/octet-stream`, and `Content-Length`. It
// keeps the body as raw bytes for the store.
impl RespondsWith for DailyIndex {
    type Body = ZipBody;

    const EXECUTE_STATE_NAME: &'static str = "Execute SEC Request";
}

impl RespondsWith for DailyIndexListing {
    type Body = JsonBody;

    const EXECUTE_STATE_NAME: &'static str = "Execute SEC Request";
}
