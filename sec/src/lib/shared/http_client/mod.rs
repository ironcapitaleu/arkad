//! # HTTP Client
//!
//! Provides the HTTP client abstraction used to execute SEC API requests.
//!
//! The client is split into two trait layers so the library is not bound to any one HTTP crate:
//! an [`InnerClient`] wraps a raw HTTP client, and a [`SecClient`]
//! adds the domain layer that turns a `SecRequest` into a `SecResponse`. This decoupling allows
//! swapping the transport layer implementation or substituting a fake for unit testing.
//!
//! ## Modules
//!
//! - [`traits`]: The [`InnerClient`] and [`SecClient`] contracts.
//! - [`implementations`]: Concrete clients — the `reqwest` transport and the default [`SecClient`].

pub mod implementations;
pub mod traits;

pub use traits::{InnerClient, SecClient};
