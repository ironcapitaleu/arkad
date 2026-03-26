pub mod implementations;
pub mod traits;

pub use crate::shared::request::traits::sec_request::SecRequestType;
pub use traits::{InnerRequest, SecRequest}; // TODO: this is not a trait, move somehwere else
