use crate::shared::request::implementations::sec_request::{CompanyFacts, NotesFile, RequestKind};
use crate::shared::response::implementations::sec_response::{JsonBody, ResponseBody, ZipBody};

/// An SEC endpoint that responds with a body of type [`RespondsWith::Body`].
///
/// [`SecClient::execute_sec_request`](super::SecClient::execute_sec_request) reads this body
/// type to choose the response type. The mapping lives on the client side, so `shared::request`
/// does not depend on `shared::response`.
pub trait RespondsWith: RequestKind {
    /// The body of the response from this endpoint.
    type Body: ResponseBody;
}

impl RespondsWith for CompanyFacts {
    type Body = JsonBody;
}

impl RespondsWith for NotesFile {
    type Body = ZipBody;
}
