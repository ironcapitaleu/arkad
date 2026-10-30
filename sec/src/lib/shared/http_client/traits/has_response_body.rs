use crate::shared::request::implementations::sec_request::{CompanyFacts, NotesFile, RequestKind};
use crate::shared::response::implementations::sec_response::{JsonBody, ResponseBody, ZipBody};

/// A [`RequestKind`] whose response has the body kind [`HasResponseBody::Body`].
///
/// [`SecClient::execute_sec_request`](super::SecClient::execute_sec_request) reads this body
/// kind to choose the response type. The mapping lives on the client side, so `shared::request`
/// does not depend on `shared::response`.
pub trait HasResponseBody: RequestKind {
    /// The body kind of the response to a request of this kind.
    type Body: ResponseBody;
}

impl HasResponseBody for CompanyFacts {
    type Body = JsonBody;
}

impl HasResponseBody for NotesFile {
    type Body = ZipBody;
}
