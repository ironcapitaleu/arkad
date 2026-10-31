//! # Response of a Request Kind
//!
//! Provides the [`HasResponse`] trait, which names the response that a request of each kind
//! gives.

use std::fmt::{Debug, Display};
use std::hash::Hash;

use serde::Serialize;

use crate::shared::request::implementations::sec_request::RequestKind;
use crate::shared::response::implementations::sec_response::SecResponse;

use super::RespondsWith;

/// A [`RequestKind`] with the response type that an [`Executes`](super::Executes) client returns
/// for it.
///
/// The bounds are the traits of a state output, because
/// [`ExecuteSecRequestOutput`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequestOutput)
/// holds the response.
///
/// A kind with its own impl must not implement [`RespondsWith`], or its impl overlaps the blanket
/// impl for `RespondsWith` kinds.
pub trait HasResponse: RequestKind {
    /// The response to a request of this kind.
    type Response: Debug + Display + Eq + Ord + Hash + Serialize + Send + Sync;
}

/// A kind that makes one HTTP call responds with one [`SecResponse`] of its body kind.
impl<K: RespondsWith> HasResponse for K {
    type Response = SecResponse<K::Body>;
}

#[cfg(test)]
mod tests {
    use crate::shared::request::implementations::sec_request::{
        CompanyFacts, FilingDocument, NotesFile,
    };
    use crate::shared::response::implementations::sec_response::{
        JsonBody, SecResponse, XmlBody, ZipBody,
    };

    use super::HasResponse;

    const fn responds_with<K: HasResponse<Response = R>, R>() {}

    #[test]
    const fn should_respond_with_a_json_sec_response_for_company_facts() {
        responds_with::<CompanyFacts, SecResponse<JsonBody>>();
    }

    #[test]
    const fn should_respond_with_a_zip_sec_response_for_notes_file() {
        responds_with::<NotesFile, SecResponse<ZipBody>>();
    }

    #[test]
    const fn should_respond_with_an_xml_sec_response_for_filing_document() {
        responds_with::<FilingDocument, SecResponse<XmlBody>>();
    }
}
