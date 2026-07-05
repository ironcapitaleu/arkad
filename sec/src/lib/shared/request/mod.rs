//! # Request
//!
//! Provides the HTTP request abstraction sent to the SEC API.
//!
//! As with the client, the request is split into two trait layers to stay HTTP-library-agnostic: an
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
