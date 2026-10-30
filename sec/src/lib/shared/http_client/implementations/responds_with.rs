//! # Endpoint Bodies
//!
//! Implements [`RespondsWith`] for each SEC endpoint, so each request names the body kind of its
//! response.

use crate::shared::http_client::RespondsWith;
use crate::shared::request::implementations::sec_request::{CompanyFacts, NotesFile};
use crate::shared::response::implementations::sec_response::{JsonBody, ZipBody};

impl RespondsWith for CompanyFacts {
    type Body = JsonBody;
}

impl RespondsWith for NotesFile {
    type Body = ZipBody;
}
