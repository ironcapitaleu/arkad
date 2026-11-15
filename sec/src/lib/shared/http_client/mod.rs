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
//! - [`traits`]: The [`InnerClient`], [`SecClient`], [`RespondsWith`], [`HasResponse`], and
//!   [`Executes`] contracts.
//! - [`retry_policy`]: The [`RetryPolicy`] that sets the attempts and the waits of one request.
//! - [`implementations`]: The `reqwest` binding, the default [`SecClient`], and the
//!   [`RespondsWith`] impl of each SEC endpoint.

pub mod implementations;
pub mod retry_policy;
pub mod traits;

pub use retry_policy::RetryPolicy;
pub use traits::{Executes, HasResponse, InnerClient, RespondsWith, SecClient};
