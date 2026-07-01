//! # Request Traits
//!
//! The two-layer request contract: an [`InnerRequest`] transport and the domain-level [`SecRequest`].
//!
//! ## Modules
//!
//! - [`inner`]: The [`InnerRequest`] trait abstracting a concrete request type.
//! - [`sec_request`]: The [`SecRequest`] trait adding SEC request semantics on top.

pub mod inner;
pub mod sec_request;

pub use inner::InnerRequest;
pub use sec_request::SecRequest;
