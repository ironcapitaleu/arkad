//! # Raw Repository
//!
//! Provides the [`RawRepository`] trait, the bound on a repository of
//! [`RawDocument`](crate::shared::raw_document::RawDocument)s that a state context can hold.
//!
//! ## Modules
//!
//! - [`traits`]: The [`RawRepository`] contract.

pub mod traits;

pub use traits::RawRepository;
