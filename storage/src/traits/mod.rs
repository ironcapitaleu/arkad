//! # Traits
//!
//! Provides the backend-agnostic traits every storage implementation satisfies.
//!
//! ## Modules
//!
//! - [`metadata_context`]: The trait every document kind implements for its metadata.
//! - [`repository`]: The traits for reading and writing records.

pub mod metadata_context;
pub mod repository;
