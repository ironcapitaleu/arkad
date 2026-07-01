//! # Common Test Fixtures
//!
//! Reusable sample implementations of the crate's traits, used to build tests without live I/O.
//!
//! ## Modules
//!
//! - [`sample_http_client`]: Fake HTTP and SEC clients.
//! - [`sample_request`]: Fake inner and SEC requests.
//! - [`sample_response`]: Fake inner and SEC responses.
//! - [`sample_sec_state`]: A minimal sample [`State`](crate::traits::state_machine::state::State).
//! - [`sample_sec_super_state`]: A sample super-state wrapping [`sample_sec_state`].
//! - [`sample_streaming_super_state`]: A multi-state super-state for streaming tests.
//! - [`data`]: Raw JSON fixtures (e.g. captured SEC responses).

pub mod sample_http_client;
pub mod sample_request;
pub mod sample_response;

pub mod sample_sec_state;
pub mod sample_sec_super_state;
pub mod sample_streaming_super_state;

pub mod data;
