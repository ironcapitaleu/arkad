//! # Response Implementations
//!
//! Concrete responses implementing the [`traits`](super::traits) contracts.
//!
//! ## Modules
//!
//! - [`reqwest`]: Implements [`InnerResponse`](super::traits::InnerResponse) for `reqwest::Response`.
//! - [`sec_response`]: The [`SecResponse`](sec_response::SecResponse), a validated response, generic over the body kind that sets its checks.

pub mod reqwest;
pub mod sec_response;
