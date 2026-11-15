//! # Traits
//!
//! Provides the backend-agnostic traits every storage implementation satisfies.
//!
//! ## Modules
//!
//! - [`repository`]: The traits for reading and writing records, and for listing documents.
//! - [`resolver`]: The ports that return the internal key of a subject for one of its
//!   identifiers.

pub mod repository;
pub mod resolver;
