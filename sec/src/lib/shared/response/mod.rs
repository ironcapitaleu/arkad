//! # Response
//!
//! Provides the HTTP response abstraction returned by the SEC API.
//!
//! Mirroring the client and request, the response has two trait layers to stay
//! transport-agnostic: an [`InnerResponse`] wraps a concrete response type (e.g. `reqwest`), and a
//! [`SecResponse`] adds the domain layer that exposes validated, typed parts (URL, headers, status,
//! content type, JSON body) and is built by reading the inner response.
//!
//! ## Modules
//!
//! - [`traits`]: The [`InnerResponse`] and [`SecResponse`] contracts.
//! - [`implementations`]: Concrete responses — the `reqwest` transport and the [`SecResponse`].

pub mod implementations;
pub mod traits;

pub use traits::InnerResponse;
pub use traits::SecResponse;
