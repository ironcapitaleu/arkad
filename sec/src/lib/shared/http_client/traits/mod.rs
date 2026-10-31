//! # HTTP Client Traits
//!
//! The two-layer client contract: a raw [`InnerClient`] and the domain-level [`SecClient`]. A
//! client runs a request of each kind through [`Executes`].
//!
//! ## Modules
//!
//! - [`executes`]: The [`Executes`] trait that runs a request of one kind.
//! - [`has_response`]: The [`HasResponse`] trait that names the response of each request kind.
//! - [`inner`]: The [`InnerClient`] trait abstracting a concrete HTTP client.
//! - [`responds_with`]: The [`RespondsWith`] trait that maps the endpoint `E` of each
//!   `SecRequest<E>` to the body kind of the response it expects.
//! - [`sec_client`]: The [`SecClient`] trait adding SEC request/response semantics on top.

pub mod executes;
pub mod has_response;
pub mod inner;
pub mod responds_with;
pub mod sec_client;

pub use executes::Executes;
pub use has_response::HasResponse;
pub use inner::InnerClient;
pub use responds_with::RespondsWith;
pub use sec_client::SecClient;
