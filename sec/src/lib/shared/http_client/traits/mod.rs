//! # HTTP Client Traits
//!
//! The two-layer client contract: a raw [`InnerClient`] and the domain-level [`SecClient`].
//!
//! ## Modules
//!
//! - [`expects_body`]: The [`ExpectsBody`] trait that wires each request kind to its body kind.
//! - [`inner`]: The [`InnerClient`] trait abstracting a concrete HTTP client.
//! - [`sec_client`]: The [`SecClient`] trait adding SEC request/response semantics on top.

pub mod expects_body;
pub mod inner;
pub mod sec_client;

pub use expects_body::ExpectsBody;
pub use inner::InnerClient;
pub use sec_client::SecClient;
