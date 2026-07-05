//! # HTTP Client Implementations
//!
//! Concrete clients implementing the [`traits`](super::traits) contracts.
//!
//! ## Modules
//!
//! - [`reqwest_client`]: Implements [`InnerClient`](super::traits::InnerClient) for [`reqwest::Client`].
//! - [`sec_client`]: The [`SecClient`](sec_client::SecClient), the domain-level client orchestrating request execution.

pub mod reqwest_client;
pub mod sec_client;
