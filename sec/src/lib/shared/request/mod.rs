//! # Request
//!
//! Provides the HTTP request abstraction sent to the SEC API.
//!
//! Split into two trait layers so the library is not bound to any one HTTP crate: an
//! [`InnerRequest`] wraps a raw HTTP request, and an [`SecRequest`] adds the domain layer that
//! builds requests from SEC concepts like a CIK and an endpoint.
//!
//! ## Modules
//!
//! - [`traits`]: The [`InnerRequest`] and [`SecRequest`] contracts.
//! - [`implementations`]: Concrete requests — the `reqwest` binding and the [`SecRequest`] builder.

pub mod implementations;
pub mod traits;

pub use traits::{InnerRequest, SecRequest};
