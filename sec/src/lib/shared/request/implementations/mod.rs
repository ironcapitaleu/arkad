//! # Request Implementations
//!
//! Concrete requests implementing the [`traits`](super::traits) contracts.
//!
//! ## Modules
//!
//! - [`reqwest`]: Implements [`InnerRequest`](super::traits::InnerRequest) for the `reqwest` request type.
//! - [`sec_request`]: The [`SecRequest`](sec_request::SecRequest) and its builder.

pub mod reqwest;
pub mod sec_request;
