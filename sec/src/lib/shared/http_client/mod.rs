//! # HTTP Client
//!
//! Provides the HTTP client abstraction used to execute SEC API requests.
//!
//! The client is split into two trait layers so the library is not bound to any one HTTP crate:
//! an [`InnerClient`] wraps a concrete transport (e.g. `reqwest`), and a [`SecClient`] adds the
//! domain layer that turns a `SecRequest` into a `SecResponse`. States depend on the traits, so a
//! fake client can stand in for the network in tests.
//!
//! ## Modules
//!
//! - [`traits`]: The [`InnerClient`] and [`SecClient`] contracts.
//! - [`implementations`]: Concrete clients — the `reqwest` transport and the default [`SecClient`].

pub mod implementations;
pub mod traits;

pub use traits::{InnerClient, SecClient};
