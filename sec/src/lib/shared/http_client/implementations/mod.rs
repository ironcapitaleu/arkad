//! # HTTP Client Implementations
//!
//! Concrete clients implementing the [`traits`](super::traits) contracts.
//!
//! ## Modules
//!
//! - [`filing_document_set`]: The [`HasResponse`](super::traits::HasResponse) and
//!   [`Executes`](super::traits::Executes) impls of the filing document set, which fetch the
//!   instance and the label file.
//! - [`reqwest_client`]: Implements [`InnerClient`](super::traits::InnerClient) for [`reqwest::Client`] and provides the [`ReqwestClient`](reqwest_client::ReqwestClient), which keeps the [`UserAgent`](crate::shared::user_agent::UserAgent) it sends.
//! - [`responds_with`]: The [`RespondsWith`](super::traits::RespondsWith) impl of each SEC endpoint.
//! - [`sec_client`]: The [`SecClient`](sec_client::SecClient), the domain-level client orchestrating request execution.

pub mod filing_document_set;
pub mod reqwest_client;
pub mod responds_with;
pub mod sec_client;
