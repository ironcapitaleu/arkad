//! # HTTP Client Traits
//!
//! The two-layer client contract: an [`InnerClient`] transport and the domain-level [`SecClient`].
//!
//! ## Modules
//!
//! - [`inner`]: The [`InnerClient`] trait abstracting a concrete HTTP transport.
//! - [`sec_client`]: The [`SecClient`] trait adding SEC request/response semantics on top.

pub mod inner;
pub mod sec_client;

pub use inner::InnerClient;
pub use sec_client::SecClient;
