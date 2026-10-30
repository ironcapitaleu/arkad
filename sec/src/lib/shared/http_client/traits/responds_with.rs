use crate::shared::request::implementations::sec_request::RequestKind;
use crate::shared::response::implementations::sec_response::ResponseBody;

/// An SEC endpoint that responds with a body of kind [`RespondsWith::Body`].
///
/// The endpoint is the only link between a request and its response:
///
/// 1. A request carries its endpoint `E` as a type: [`SecRequest<E>`].
/// 2. `E` names its body kind here. For example, `NotesFile` responds with a [`ZipBody`].
/// 3. [`SecClient::execute_sec_request`](super::SecClient::execute_sec_request) returns a
///    `Response<E::Body>`, so the client builds and checks the body kind that `E` names.
///
/// This trait lives on the client side, so `shared::request` does not depend on
/// `shared::response`.
///
/// The impls for each endpoint live in
/// [`implementations::responds_with`](crate::shared::http_client::implementations::responds_with).
///
/// [`SecRequest<E>`]: crate::shared::request::implementations::sec_request::SecRequest
/// [`ZipBody`]: crate::shared::response::implementations::sec_response::ZipBody
pub trait RespondsWith: RequestKind {
    /// The body kind of the response from this endpoint.
    type Body: ResponseBody;
}
