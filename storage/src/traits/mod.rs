//! # Traits
//!
//! Provides the backend-agnostic traits every storage implementation satisfies.
//!
//! ## Modules
//!
//! - [`document`]: The trait every storable document kind implements.
//! - [`document_metadata`]: The trait every metadata kind implements, and the check of its shape.
//! - [`repository`]: The traits for reading and writing records.

pub mod document;
pub mod document_metadata;
pub mod repository;
