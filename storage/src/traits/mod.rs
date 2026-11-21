//! # Traits
//!
//! Provides the backend-agnostic traits every storage implementation satisfies.
//!
//! ## Modules
//!
//! - [`lease`]: The port through which the runs of a job kind take turns to write.
//! - [`repository`]: The traits for reading and writing records, and for listing documents.
//! - [`resolver`]: The ports that return the internal key of a subject for one of its
//!   identifiers.

pub mod lease;
pub mod repository;
pub mod resolver;
