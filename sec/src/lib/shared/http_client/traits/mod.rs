//! # HTTP Client Traits
//!
//! The two-layer client contract: a raw [`InnerClient`] and the domain-level [`SecClient`].
//!
//! ## Modules
//!
//! - [`inner`]: The [`InnerClient`] trait abstracting a concrete HTTP client.
//! - [`responds_with`]: The [`RespondsWith`] trait that wires each SEC endpoint to its response body.
//! - [`sec_client`]: The [`SecClient`] trait adding SEC request/response semantics on top.

pub mod inner;
pub mod responds_with;
pub mod sec_client;

pub use inner::InnerClient;
pub use responds_with::RespondsWith;
pub use sec_client::SecClient;
