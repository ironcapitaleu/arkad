//! # Raw Repository Traits
//!
//! The contract on a repository of
//! [`RawDocument`](crate::shared::raw_document::RawDocument)s that a state context can hold.
//!
//! ## Modules
//!
//! - [`raw_repository`]: The [`RawRepository`] trait.

pub mod raw_repository;

pub use raw_repository::RawRepository;
