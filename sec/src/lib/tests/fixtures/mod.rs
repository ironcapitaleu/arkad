//! # Common Test Fixtures
//!
//! Reusable sample implementations and data for building tests without live I/O.
//!
//! ## Modules
//!
//! - [`sample_http_client`]: Fake HTTP and SEC clients.
//! - [`sample_input_document`]: Sample input lists for the metadata `inputs` of stored documents.
//! - [`sample_notes_archive`]: A writer of small Notes ZIP files, and the members of a valid one.
//! - [`sample_rate_limiter`]: Fake rate limiters that never block, count turns, or hold callers.
//! - [`sample_request`]: Fake inner and SEC requests.
//! - [`sample_response`]: Fake inner and SEC responses.
//! - [`sample_server`]: Local TCP servers that stop sending, close the connection, or send a gzip
//!   body.
//! - [`sample_filing_context`]: A filing context on the registry of the `xbrl` fixture filing.
//! - [`sample_source_document`]: The raw documents of one sample Notes file and one sample
//!   instance.
//! - [`sample_sec_state`]: A minimal sample [`State`](crate::traits::state_machine::state::State).
//! - [`sample_sec_super_state`]: A sample super-state wrapping [`sample_sec_state`].
//! - [`sample_streaming_super_state`]: A multi-state super-state for streaming tests.
//! - [`sample_write_repository`]: A fake repository that records or rejects raw documents.
//! - [`data`]: Raw file fixtures (e.g. captured SEC responses).

pub mod sample_http_client;
pub mod sample_input_document;
pub mod sample_notes_archive;
pub mod sample_rate_limiter;
pub mod sample_request;
pub mod sample_response;
pub mod sample_server;
pub mod sample_source_document;

// Deviation: only unit tests build this module, since the `test-fixtures` feature of `sec` does
// not turn on the `test-fixtures` feature of `xbrl` that it reads.
#[cfg(test)]
pub mod sample_filing_context;

pub mod sample_sec_state;
pub mod sample_sec_super_state;
pub mod sample_streaming_super_state;
pub mod sample_write_repository;

pub mod data;
