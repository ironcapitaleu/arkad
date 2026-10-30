use crate::shared::request::implementations::sec_request::{CompanyFacts, NotesFile, RequestKind};
use crate::shared::response::implementations::sec_response::{JsonBody, ResponseBody, ZipBody};

/// A [`RequestKind`] that responds with a body of kind [`RespondsWith::Body`].
///
/// [`SecClient::execute_sec_request`](super::SecClient::execute_sec_request) reads this body
/// kind to choose the response type. The mapping lives on the client side, so `shared::request`
/// does not depend on `shared::response`.
pub trait RespondsWith: RequestKind {
    /// The body kind of the response to a request of this kind.
    type Body: ResponseBody;
}

impl RespondsWith for CompanyFacts {
    type Body = JsonBody;
}

impl RespondsWith for NotesFile {
    type Body = ZipBody;
}
