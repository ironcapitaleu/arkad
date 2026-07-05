//! # Response
//!
//! Provides the HTTP response abstraction returned by the SEC API.
//!
//! Split into two trait layers so the library is not bound to any one HTTP crate: an
//! [`InnerResponse`] wraps a raw HTTP response, and an [`SecResponse`] validates it and exposes
//! typed parts (URL, headers, status, content type, JSON body).
//!
//! ## Modules
//!
//! - [`traits`]: The [`InnerResponse`] and [`SecResponse`] contracts.
//! - [`implementations`]: Concrete responses — the `reqwest` binding and the [`SecResponse`].

pub mod implementations;
pub mod traits;

pub use traits::InnerResponse;
pub use traits::SecResponse;
