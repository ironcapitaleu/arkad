//! # Response Implementations
//!
//! Concrete responses implementing the [`traits`](super::traits) contracts.
//!
//! ## Modules
//!
//! - [`reqwest`]: Implements [`InnerResponse`](super::traits::InnerResponse) for `reqwest::Response`.
//! - [`sec_notes_file_response`]: The [`SecNotesFileResponse`](sec_notes_file_response::SecNotesFileResponse), a validated response to a Notes file request.
//! - [`sec_response`]: The [`SecResponse`](sec_response::SecResponse), a validated response guaranteeing success status, JSON content type, and parseable body.

pub mod reqwest;
pub mod sec_notes_file_response;
pub mod sec_response;
