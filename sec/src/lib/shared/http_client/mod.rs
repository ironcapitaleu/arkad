//! # HTTP Client
//!
//! Provides the HTTP client abstraction used to execute SEC API requests.
//!
//! The client is split into two trait layers so the library is not bound to any one HTTP crate:
//! an [`InnerClient`] wraps a raw HTTP client, and an [`SecClient`]
//! adds the domain layer that turns an `SecRequest` into an `SecResponse`. This decoupling allows
//! swapping the HTTP library or substituting a fake for unit testing.
//!
//! ## Modules
//!
//! - [`traits`]: The [`InnerClient`], [`SecClient`], and [`RespondsWith`] contracts.
//! - [`implementations`]: The `reqwest` binding, the default [`SecClient`], and the
//!   [`RespondsWith`] impl of each SEC endpoint.

pub mod implementations;
pub mod traits;

pub use traits::{InnerClient, RespondsWith, SecClient};
