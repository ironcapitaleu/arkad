//! # Endpoint Bodies
//!
//! Implements [`RespondsWith`] for each SEC endpoint, so each endpoint names the body kind of
//! its response.

use crate::shared::http_client::RespondsWith;
use crate::shared::request::implementations::sec_request::{
    CompanyFacts, DailyIndex, DailyIndexListing, FilingDocument, FilingIndex, NotesFile,
    Submissions,
};
use crate::shared::response::implementations::sec_response::{JsonBody, XmlBody, ZipBody};

impl RespondsWith for CompanyFacts {
    type Body = JsonBody;
}

impl RespondsWith for NotesFile {
    type Body = ZipBody;
}

// Deviation: the daily index is fixed-width text, not a ZIP. `ZipBody` is reused because it runs
// the checks the index needs: status 200, `application/octet-stream`, and `Content-Length`. It
// keeps the body as raw bytes for the store.
impl RespondsWith for DailyIndex {
    type Body = ZipBody;
}

impl RespondsWith for DailyIndexListing {
    type Body = JsonBody;
}

impl RespondsWith for Submissions {
    type Body = JsonBody;
}

impl RespondsWith for FilingIndex {
    type Body = JsonBody;
}

impl RespondsWith for FilingDocument {
    type Body = XmlBody;
}
